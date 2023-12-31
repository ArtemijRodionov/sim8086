use std::fmt;

#[derive(Debug)]
pub struct Error;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Mem0Disp,
    Mem1Disp,
    Mem2Disp,
    Reg,
}

impl Mode {
    pub fn from(mode: u8) -> Self {
        match mode & 0b11 {
            0b00 => Self::Mem0Disp,
            0b01 => Self::Mem1Disp,
            0b10 => Self::Mem2Disp,
            0b11 => Self::Reg,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Register {
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

impl Register {
    fn from(reg: u8, w: u8) -> Self {
        match (reg & 0b111, w & 0b1) {
            (0b000, 0b0) => Self::AL,
            (0b001, 0b0) => Self::CL,
            (0b010, 0b0) => Self::DL,
            (0b011, 0b0) => Self::BL,
            (0b100, 0b0) => Self::AH,
            (0b101, 0b0) => Self::CH,
            (0b110, 0b0) => Self::DH,
            (0b111, 0b0) => Self::BH,
            (0b000, 0b1) => Self::AX,
            (0b001, 0b1) => Self::CX,
            (0b010, 0b1) => Self::DX,
            (0b011, 0b1) => Self::BX,
            (0b100, 0b1) => Self::SP,
            (0b101, 0b1) => Self::BP,
            (0b110, 0b1) => Self::SI,
            (0b111, 0b1) => Self::DI,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Address {
    BXSI,
    BXDI,
    BPSI,
    BPDI,
    SI,
    DI,
    BX,
    DirectBP,
}

impl Address {
    pub fn from(address: u8) -> Self {
        match address & 0b111 {
            0b000 => Self::BXSI,
            0b001 => Self::BXDI,
            0b010 => Self::BPSI,
            0b011 => Self::BPDI,
            0b100 => Self::SI,
            0b101 => Self::DI,
            0b110 => Self::DirectBP,
            0b111 => Self::BX,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EffectiveAddress {
    address: Address,
    disp: u16,
}

impl EffectiveAddress {
    fn new(address: Address, disp: u16) -> Self {
        Self { address, disp }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    Accumulator,
    Memory(u16),
    Immediate(u16),
    Immediate8ToMem(u8),
    Immediate16ToMem(u16),
    Register(Register),
    EffectiveAddress(EffectiveAddress),
}

impl Encoding {
    pub fn direct(direct: u16) -> Self {
        Self::Memory(direct)
    }

    pub fn register(reg: u8, w: u8) -> Self {
        Self::Register(Register::from(reg, w))
    }

    pub fn effective_address(address: Address, disp: u16) -> Self {
        Self::EffectiveAddress(EffectiveAddress::new(address, disp))
    }
}

pub struct Inst {
    name: String,
    lhs: Encoding,
    rhs: Encoding,
}

impl Inst {
    pub fn new(name: String, lhs: Encoding, rhs: Encoding) -> Self {
        Self { name, lhs, rhs }
    }
}

impl fmt::Display for EffectiveAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}{}]",
            match self.address {
                Address::BXSI => "bx + si",
                Address::BXDI => "bx + di",
                Address::BPSI => "bp + si",
                Address::BPDI => "bp + di",
                Address::SI => "si",
                Address::DI => "di",
                Address::BX => "bx",
                Address::DirectBP => "bp",
            },
            match self.disp {
                0 => "".to_string(),
                disp => format!(" + {}", disp),
            }
        )
    }
}

impl fmt::Display for Register {
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

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Accumulator => "ax".to_string(),
                Self::Immediate(e) => e.to_string(),
                Self::Immediate8ToMem(e) => format!("byte {}", e),
                Self::Immediate16ToMem(e) => format!("word {}", e),
                Self::Memory(e) => format!("[{}]", e),
                Self::Register(r) => r.to_string(),
                Self::EffectiveAddress(e) => e.to_string(),
            }
        )
    }
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}, {}", self.name, self.lhs, self.rhs)
    }
}
