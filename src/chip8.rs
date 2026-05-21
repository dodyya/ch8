use crate::instruction::{Instruction, Instruction::*, Operation::*, u4};

const FONT_START: usize = 0x50;
const FONT_WIDTH: usize = 5;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const RAM_BYTES: usize = 4096;
pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: u8 = 32;

pub struct Chip8 {
    ram: [u8; RAM_BYTES],
    var_registers: [u8; 16],
    index: usize,
    program_counter: usize,
    pub display_buffer: [bool; 2048],
    //Timers
    timer: u8,
    sound_timer: u8,

    stack: Vec<u16>,

    /// State of the 16 hex keys (true = pressed).
    pub keypad: [bool; 16],
    /// True while the CPU is blocked on an FX0A "wait for key" instruction.
    halted: bool,

    /// State for the inline xorshift PRNG used by the RND opcode.
    rng_state: u32,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut out = Chip8 {
            ram: [0; RAM_BYTES],
            var_registers: [0; 16],
            index: 0,
            program_counter: 0x200,
            display_buffer: [false; 2048],
            timer: 0,
            sound_timer: 0,
            stack: vec![],
            keypad: [false; 16],
            halted: false,
            rng_state: 0x1234_5678,
        };

        out.load_fontset();
        out
    }

    /// Tiny xorshift32 PRNG, used by the RND opcode. A fixed seed is fine for
    /// a demo and keeps the build free of the `getrandom`/`rand` dependency.
    fn next_random(&mut self) -> u8 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state = x;
        (x & 0xFF) as u8
    }

    pub fn set_key(&mut self, key: u8, pressed: bool) {
        if (key as usize) < self.keypad.len() {
            self.keypad[key as usize] = pressed;
            // A key press releases the CPU from an FX0A wait.
            if pressed {
                self.halted = false;
            }
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        assert!(data.len() < RAM_BYTES - 0x200);
        self.ram[0x200..0x200 + data.len()].copy_from_slice(data);
        // println!("{:#x?}", self.ram.iter().enumerate());
    }

    pub fn load_fontset(&mut self) {
        self.ram[FONT_START..FONT_START + FONTSET.len()].copy_from_slice(&FONTSET);
    }

    pub fn execute_cycle(&mut self) {
        // While halted (FX0A) the CPU does nothing until a key is pressed.
        if self.halted {
            return;
        }
        let instruction = match self.parse_instruction() {
            Ok(i) => i,
            // Skip bytes we cannot decode rather than aborting the whole demo.
            Err(_) => {
                self.program_counter += 2;
                return;
            }
        };
        self.program_counter += 2;
        self.execute(instruction);
    }

    fn parse_instruction(&mut self) -> Result<Instruction, String> {
        // println!(
        //     "{:x?}{:x?}",
        //     &self.ram[self.program_counter],
        //     &self.ram[self.program_counter + 1]
        // );
        let out = Instruction::try_from(&self.ram[self.program_counter..self.program_counter + 2]);
        return out;
    }

    fn execute(&mut self, i: Instruction) {
        match i {
            ClearScreen => self.display_buffer.fill(false),
            Jump { loc } => self.program_counter = loc,
            SetRegister { reg, value } => {
                self.var_registers[reg as usize] = value;
                // println!("Set register");
            }
            AddRegister { reg, value } => {
                // 7XKK does not affect the carry flag.
                let result = self.var_registers[reg as usize].wrapping_add(value);
                self.var_registers[reg as usize] = result;
            }
            SetIndex { value } => self.index = value,
            Display {
                x_reg,
                y_reg,
                num_bytes,
            } => self.display(x_reg, y_reg, num_bytes),
            Skip { eq, reg, value } => {
                let predicate: bool = self.var_registers[reg as usize] == value;
                if predicate == eq {
                    self.program_counter += 2;
                }
            }
            AddIndex { reg } => {
                self.index += self.var_registers[reg as usize] as usize;
            }
            StoreTimer { reg } => {
                self.var_registers[reg as usize] = self.timer;
            }
            Do { loc } => {
                self.stack.push((self.program_counter) as u16);
                self.program_counter = loc;
            }
            Return => {
                // Ignore an underflowing return rather than crashing the demo.
                if let Some(return_address) = self.stack.pop() {
                    self.program_counter = return_address as usize;
                }
            }
            SetTimer { reg } => {
                self.timer = self.var_registers[reg as usize];
            }
            RandomAnd { reg, val } => {
                let r = self.next_random();
                self.var_registers[reg as usize] = val & r;
            }

            LogicOp { op, x, y } => {
                let x_val = self.var_registers[x as usize];
                let y_val = self.var_registers[y as usize];
                // Compute result and the new VF before writing either, so the
                // flag is never clobbered when x or y happens to be register F.
                let (result, flag) = match op {
                    Copy => (y_val, None),
                    Or => (x_val | y_val, None),
                    And => (x_val & y_val, None),
                    Xor => (x_val ^ y_val, None),
                    Add => {
                        let (val, overflowed) = x_val.overflowing_add(y_val);
                        (val, Some(overflowed as u8))
                    }
                    // CHIP-8 subtract: VF = 1 when there is NO borrow.
                    Sub => {
                        let (val, borrow) = x_val.overflowing_sub(y_val);
                        (val, Some((!borrow) as u8))
                    }
                    SubN => {
                        let (val, borrow) = y_val.overflowing_sub(x_val);
                        (val, Some((!borrow) as u8))
                    }
                    ShiftLeft => (x_val << 1, Some((x_val >> 7) & 1)),
                    ShiftRight => (x_val >> 1, Some(x_val & 1)),
                };
                self.var_registers[x as usize] = result;
                if let Some(f) = flag {
                    self.var_registers[0xF] = f;
                }
            }

            StoreDec { reg } => {
                let num = self.var_registers[reg as usize];
                self.ram[self.index] = num / 100 % 10;
                self.ram[self.index + 1] = num / 10 % 10;
                self.ram[self.index + 2] = num % 10;
            }

            StoreVars { reg } => {
                self.ram[self.index..self.index + reg as usize + 1]
                    .copy_from_slice(&self.var_registers[0..=reg as usize]);
            }

            LoadVars { reg } => {
                self.var_registers[0..=reg as usize]
                    .copy_from_slice(&self.ram[self.index..self.index + reg as usize + 1]);
            }

            FontPoint { reg } => {
                self.index = FONT_START + (self.var_registers[reg as usize] as usize * FONT_WIDTH);
            }

            SkipReg { eq, reg1, reg2 } => {
                let predicate =
                    self.var_registers[reg1 as usize] == self.var_registers[reg2 as usize];
                if predicate == eq {
                    self.program_counter += 2;
                }
            }

            JumpAdvance { loc } => {
                self.program_counter = loc + self.var_registers[0] as usize;
            }

            SkipKey { eq, reg } => {
                let key = self.var_registers[reg as usize] as usize & 0x0F;
                if self.keypad[key] == eq {
                    self.program_counter += 2;
                }
            }

            StoreKey { reg } => {
                // FX0A: block until a key is pressed, then store its index.
                match self.keypad.iter().position(|&pressed| pressed) {
                    Some(key) => self.var_registers[reg as usize] = key as u8,
                    None => {
                        // Re-run this instruction next cycle until a key is hit.
                        self.program_counter -= 2;
                        self.halted = true;
                    }
                }
            }

            // FX18 sets the sound timer; no audio is produced in the demo,
            // but the timer is tracked so programs that poll it behave.
            SetTone { reg } => {
                self.sound_timer = self.var_registers[reg as usize];
            }
            SetPitch { .. } => {}

            NoOperation | Stop => {}
        }
    }

    fn display(&mut self, x_reg: u4, y_reg: u4, num_bytes: u8) {
        let start_x = (self.var_registers[x_reg as usize] % SCREEN_WIDTH) as usize;
        let start_y = (self.var_registers[y_reg as usize] % SCREEN_HEIGHT) as usize;
        self.var_registers[0xF] = 0;

        for row_num in 0..num_bytes as usize {
            let y = start_y + row_num;
            if y >= SCREEN_HEIGHT as usize {
                break;
            }

            let row_byte = self.ram[self.index + row_num];
            let mut x = start_x;

            for bit in (0..8).rev() {
                if x >= SCREEN_WIDTH as usize {
                    break;
                }
                let pix = ((row_byte >> bit) & 1) != 0;
                let idx = y * SCREEN_WIDTH as usize + x;
                if pix && self.display_buffer[idx] {
                    self.var_registers[0xF] = 1;
                }
                self.display_buffer[idx] ^= pix;
                x += 1;
            }
        }
    }

    pub fn update_timers(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}

#[allow(dead_code)]
fn print_buffer(buffer: &[bool; 2048]) {
    for i in 0..32 {
        for j in 0..64 {
            let c = if buffer[i * 64 + j] { '#' } else { '.' };
            print!("{}", c);
        }
        println!();
    }
}
