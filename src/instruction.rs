pub type Location = usize; //But actually should only be in u16 range
pub type u4 = u8; // But actually should only be in u4 range
#[derive(Debug)]
pub enum Instruction {
    ClearScreen,
    Jump { loc: Location },
    SetRegister { register: u4, value: u8 },
    AddRegister { register: u4, value: u8 },
    SetIndex { value: Location },
    Display { x_reg: u4, y_reg: u4, num_bytes: u4 },
}

impl TryFrom<&[u8]> for Instruction {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let value = u16::from_be_bytes([data[0], data[1]]);
        // println!("{:?}", value);
        match value & 0xF000 {
            0x0000 => return Ok(Self::ClearScreen),
            0x1000 => {
                return Ok(Self::Jump {
                    loc: (value & 0x0FFF) as Location,
                });
            }
            0x6000 => {
                return Ok(Self::SetRegister {
                    register: ((value & 0x0F00) >> 8) as u4,
                    value: (value & 0x00FF) as u8,
                });
            }
            0x7000 => {
                return Ok(Self::AddRegister {
                    register: ((value & 0x0F00) >> 8) as u4,
                    value: (value & 0x00FF) as u8,
                });
            }
            0xA000 => {
                return Ok(Self::SetIndex {
                    value: (value & 0x0FFF) as Location,
                });
            }
            0xD000 => {
                return Ok(Self::Display {
                    x_reg: ((value & 0x0F00) >> 8) as u4,
                    y_reg: ((value & 0x00F0) >> 4) as u4,
                    num_bytes: (value & 0x000F) as u4,
                });
            }
            _ => return Err("Could not parse instruction".into()),
        }
    }
}
