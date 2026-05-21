pub type Location = usize; //But actually should only be in u16 range
pub type u4 = u8; // But actually should only be in u4 range

#[derive(Debug)]
pub enum Operation {
    Copy,
    Or,
    And,
    Xor,
    Add,
    Sub,
    SubN,
    ShiftLeft,
    ShiftRight,
}
#[derive(Debug)]
pub enum Instruction {
    NoOperation,
    ClearScreen,
    Return,

    Jump { loc: Location },
    Do { loc: Location },
    Skip { eq: bool, reg: u4, value: u8 },
    SkipReg { eq: bool, reg1: u4, reg2: u4 },
    SetRegister { reg: u4, value: u8 },
    AddRegister { reg: u4, value: u8 },

    LogicOp { op: Operation, x: u4, y: u4 },

    SetIndex { value: Location },
    JumpAdvance { loc: Location },
    RandomAnd { reg: u4, val: u8 },
    Display { x_reg: u4, y_reg: u4, num_bytes: u4 },

    SkipKey { eq: bool, reg: u4 },

    Stop,

    StoreTimer { reg: u4 },
    StoreKey { reg: u4 },

    SetTimer { reg: u4 },
    SetPitch { reg: u4 },
    SetTone { reg: u4 },

    AddIndex { reg: u4 },

    FontPoint { reg: u4 },
    StoreDec { reg: u4 },

    StoreVars { reg: u4 },
    LoadVars { reg: u4 },
}

impl TryFrom<&[u8]> for Instruction {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let value = u16::from_be_bytes([data[0], data[1]]);
        // println!("{:?}", value);
        match (value & 0xF000) >> 12 {
            0 => match value & 0x00FF {
                0x00 => Ok(Self::NoOperation),
                0xE0 => Ok(Self::ClearScreen),
                0xEE => Ok(Self::Return),
                _ => Err("Could not parse instruction starting with 0".into()),
            },
            0x1 => Ok(Self::Jump {
                loc: (value & 0x0FFF) as Location,
            }),
            0x2 => Ok(Self::Do {
                loc: (value & 0x0FFF) as Location,
            }),
            0x3 => Ok(Self::Skip {
                eq: true,
                reg: ((value & 0x0F00) >> 8) as u4,
                value: (value & 0x00FF) as u8,
            }),
            0x4 => Ok(Self::Skip {
                eq: false,
                reg: ((value & 0x0F00) >> 8) as u4,
                value: (value & 0x00FF) as u8,
            }),
            0x5 => Ok(Self::SkipReg {
                eq: true,
                reg1: ((value & 0x0F00) >> 8) as u4,
                reg2: ((value & 0x00F0) >> 4) as u4,
            }),
            0x6 => Ok(Self::SetRegister {
                reg: ((value & 0x0F00) >> 8) as u4,
                value: (value & 0x00FF) as u8,
            }),
            0x7 => Ok(Self::AddRegister {
                reg: ((value & 0x0F00) >> 8) as u4,
                value: (value & 0x00FF) as u8,
            }),
            0x8 => {
                let op = match value & 0x000F {
                    0 => Operation::Copy,
                    1 => Operation::Or,
                    2 => Operation::And,
                    3 => Operation::Xor,
                    4 => Operation::Add,
                    5 => Operation::Sub,
                    6 => Operation::ShiftRight,
                    7 => Operation::SubN,
                    0xE => Operation::ShiftLeft,
                    _ => return Err("Could not parse instruction starting with 8".into()),
                };
                Ok(Self::LogicOp {
                    op,
                    x: ((value & 0x0F00) >> 8) as u4,
                    y: ((value & 0x00F0) >> 4) as u4,
                })
            }
            0x9 => match value & 0x000F {
                0 => Ok(Self::SkipReg {
                    eq: false,
                    reg1: ((value & 0x0F00) >> 8) as u4,
                    reg2: ((value & 0x00F0) >> 4) as u4,
                }),
                _ => Err("Could not parse instruction starting with 9".into()),
            },
            0xA => Ok(Self::SetIndex {
                value: (value & 0x0FFF) as Location,
            }),
            0xB => Ok(Self::JumpAdvance {
                loc: (value & 0x0FFF) as Location,
            }),
            0xC => Ok(Self::RandomAnd {
                reg: ((value & 0x0F00) >> 8) as u4,
                val: (value & 0x00FF) as u8,
            }),
            0xD => Ok(Self::Display {
                x_reg: ((value & 0x0F00) >> 8) as u4,
                y_reg: ((value & 0x00F0) >> 4) as u4,
                num_bytes: (value & 0x000F) as u4,
            }),
            0xE => {
                let eq = match value & 0x00FF {
                    0x9E => true,
                    0xA1 => false,
                    _ => return Err("Could not parse instruction starting with E".into()),
                };
                Ok(Self::SkipKey {
                    eq,
                    reg: ((value & 0x0F00) >> 8) as u4,
                })
            }
            0xF => {
                let x = ((value & 0x0F00) >> 8) as u4;
                match value & 0x00FF {
                    0x00 => Ok(Self::Stop),                  // F000
                    0x07 => Ok(Self::StoreTimer { reg: x }), // FX07
                    0x0A => Ok(Self::StoreKey { reg: x }),   // FX0A
                    0x15 => Ok(Self::SetTimer { reg: x }),   // FX15
                    0x17 => Ok(Self::SetPitch { reg: x }),   // FX17
                    0x18 => Ok(Self::SetTone { reg: x }),    // FX18
                    0x1E => Ok(Self::AddIndex { reg: x }),   // FX1E
                    0x29 => Ok(Self::FontPoint { reg: x }),
                    0x33 => Ok(Self::StoreDec { reg: x }),
                    0x55 => Ok(Self::StoreVars { reg: x }),
                    0x65 => Ok(Self::LoadVars { reg: x }),

                    _ => Err(format!("Could not parse instruction {:x}", value)),
                }
            }
            _ => Err("Could not parse instruction".into()),
        }
    }
}
//Stored Code Mnemonic Description 0000 NOP No Operation. 00E0 ERASE Clear the Screen. 00EE RETURN Return from Subroutine. 1MMM GOTO MMM Jump to location MMM. 2MMM DO MMM Call Subroutine. 3XKK SKF VX=KK Skip next Instruction if VX=KK. 4XKK SKF VX≠KK Skip next Instruction if VX≠KK. 5XY0 SKF VX=VY Skip next Instruction if VX=VY. 6XKK VX=KK Assign Hex value KK to Register VX. 7XKK VX=VX+KK Add KK to VX. 8XY0 VX=VY Copy VY to VX. 8XY1 VX=VX│VY Logical OR VX with VY. 8XY2 VX=VX.VY Logical AND VX with VY. 8XY3 VX=VX XOR VY Logical XOR VX with VY. 8XY4 VX=VX+VY Add VY to VX.If result >FF, then VF=1. 8XY5 VX=VX-VY Subtract VY. If VX<VY, then VF=0. 9XY0 SKF VX≠VY Skip next Instruction if VX≠VY. AMMM I=MMM Set memory Index Pointer to MMM. BMMM GOTO MMM+V0 Jump to location MMM+V0. CXKK VX=RND.KK Get random byte, then AND with KK. DXYN SHOW N@VX,VY Display N-byte pattern at (VX,VY). EX9E SKF VX=KEY Skip if key down =VX. No wait. EXA1 SKF VX≠KEY Skip if key down ≠VX. No wait. F000 STOP Jump to Monitor (CHIPOS). FX07 VX=TIME Get current timer value. FX0A VX=KEY Input Hex key code. Wait for key down. FX15 TIME=VX Initialize Timer. 01=20 mS. FX17 PITCH=VX Set the Pitch of the Tone Generator to VX. FX18 TONE=VX Sound Tone for 20 timesVX milliseconds. FX1E I=I+VX Add VX to Memory Pointer. FX29 I=DSP,VX Set Pointer to show VX (LS digit). FX33 MI=DEQ,VX Store 3 digit decimal equivalent of VX. FX55 MI=VO:VX Store V0 through VX at I. I=I+X+1. FX65 V0:VX=MI Load V0 through VX at I. I=I+X+1. FX70 RS485=VX Send data in VX to RS485 Port. FX71 VX=RS485 Waits for received RS485 data. Place in VX. FX72 BAUD=VX Set RS485 Baud rate.
