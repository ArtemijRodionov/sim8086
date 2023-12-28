use std::fmt;

#[derive(Debug)]
pub struct Error;

#[derive(Debug, Clone, Copy)]
enum Mode {
    Mem0disp = 0b00,
    Mem8disp = 0b01,
    Mem16dist = 0b10,
    Reg = 0b11,
}

#[derive(Debug, Clone, Copy)]
pub enum FieldEncoding {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

impl FieldEncoding {
    pub fn from(reg: u8, w: u8) -> Result<Self, Error> {
        match (reg & 0b111) << 1 | (w & 0b1) {
            0b0000 => Ok(Self::AL),
            0b0010 => Ok(Self::CL),
            0b0100 => Ok(Self::DL),
            0b0110 => Ok(Self::BL),
            0b1000 => Ok(Self::AH),
            0b1010 => Ok(Self::CH),
            0b1100 => Ok(Self::DH),
            0b1110 => Ok(Self::BH),
            0b0001 => Ok(Self::AX),
            0b0011 => Ok(Self::CX),
            0b0101 => Ok(Self::DX),
            0b0111 => Ok(Self::BX),
            0b1001 => Ok(Self::SP),
            0b1011 => Ok(Self::BP),
            0b1101 => Ok(Self::SI),
            0b1111 => Ok(Self::DI),
            _ => Err(Error),
        }
    }
}

impl fmt::Display for FieldEncoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AL => "al",
                Self::CL => "cl",
                Self::DL => "dl",
                Self::BL => "bl",
                Self::AH => "ah",
                Self::CH => "ch",
                Self::DH => "dh",
                Self::BH => "bh",
                Self::AX => "ax",
                Self::CX => "cx",
                Self::DX => "dx",
                Self::BX => "bx",
                Self::SP => "sp",
                Self::BP => "bp",
                Self::SI => "si",
                Self::DI => "di",
            }
        )
    }
}

pub struct Inst {
    name: String,
    lhs: FieldEncoding,
    rhs: FieldEncoding,
}

impl Inst {
    pub fn new(name: String, lhs: FieldEncoding, rhs: FieldEncoding) -> Self {
        Self { name, lhs, rhs }
    }
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}, {}", self.name, self.lhs, self.rhs)
    }
}
