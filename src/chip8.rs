use crate::instruction::{Instruction, u4};

pub const RAM_BYTES: usize = 4096;
pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: u8 = 32;
pub struct Chip8 {
    ram: [u8; RAM_BYTES],
    var_registers: [u8; 16],
    index: usize,
    program_counter: usize,
    pub display_buffer: [bool; 2048],
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            ram: [0; RAM_BYTES],
            var_registers: [0; 16],
            index: 0,
            program_counter: 0x200,
            display_buffer: [false; 2048],
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        assert!(data.len() < RAM_BYTES - 0x200);
        self.ram[0x200..0x200 + data.len()].copy_from_slice(data);
        // println!("{:#x?}", self.ram.iter().enumerate());
    }

    pub fn execute_cycle(&mut self) {
        let instruction: Instruction = self.parse_instruction().expect("Failed to parse");
        // println!("{:?}", instruction);
        self.program_counter += 2;
        self.execute(instruction);
        // print_buffer(&self.display_buffer);
        // println!("{}", self.program_counter);
    }

    fn parse_instruction(&mut self) -> Result<Instruction, String> {
        // // println!(
        //     "{:?}",
        //     &self.ram[self.program_counter..self.program_counter + 2]
        // );
        let out = Instruction::try_from(&self.ram[self.program_counter..self.program_counter + 2]);
        return out;
    }

    fn execute(&mut self, i: Instruction) {
        match i {
            Instruction::ClearScreen => self.display_buffer.fill(false),
            Instruction::Jump { loc } => self.program_counter = loc,
            Instruction::SetRegister { register, value } => {
                self.var_registers[register as usize] = value;
                // println!("Set register");
            }
            Instruction::AddRegister { register, value } => {
                self.var_registers[register as usize] += value
            }
            Instruction::SetIndex { value } => self.index = value,
            Instruction::Display {
                x_reg,
                y_reg,
                num_bytes,
            } => self.display(x_reg, y_reg, num_bytes),
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
}

fn print_buffer(buffer: &[bool; 2048]) {
    for i in 0..32 {
        for j in 0..64 {
            let c = if buffer[i * 64 + j] { '#' } else { '.' };
            print!("{}", c);
        }
        println!();
    }
}
