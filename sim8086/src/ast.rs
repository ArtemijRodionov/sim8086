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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) enum OperandType {
    Explicit,
    Implicit,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) enum OperandSize {
    Byte,
    Word,
}

impl OperandSize {
    pub(crate) fn size(self) -> u8 {
        match self {
            Self::Byte => 8,
            Self::Word => 16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Register {
    AX,
    BX,
    CX,
    DX,
    SP,
    BP,
    SI,
    DI,
    AL,
    BL,
    CL,
    DL,
    AH,
    BH,
    CH,
    DH,
}

impl Register {
    pub(crate) fn from(reg: u8, w: u8) -> Self {
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

    pub(crate) fn size(self) -> OperandSize {
        match self {
            Self::AL
            | Self::BL
            | Self::CL
            | Self::DL
            | Self::AH
            | Self::BH
            | Self::CH
            | Self::DH => OperandSize::Byte,
            Self::AX
            | Self::BX
            | Self::CX
            | Self::DX
            | Self::SP
            | Self::BP
            | Self::SI
            | Self::DI => OperandSize::Word,
        }
    }

    pub(crate) fn to_idx(self) -> usize {
        match self {
            Self::AX => 0,
            Self::BX => 1,
            Self::CX => 2,
            Self::DX => 3,
            Self::SP => 4,
            Self::BP => 5,
            Self::SI => 6,
            Self::DI => 7,

            Self::AH => 0,
            Self::AL => 1,
            Self::BH => 2,
            Self::BL => 3,
            Self::CH => 4,
            Self::CL => 5,
            Self::DH => 6,
            Self::DL => 7,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterAddress {
    Empty,
    BXSI,
    BXDI,
    BPSI,
    BPDI,
    SI,
    DI,
    BX,
    DirectBP,
}

impl RegisterAddress {
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
    pub(crate) register: RegisterAddress,
    pub(crate) disp: i16,
}

impl EffectiveAddress {
    pub(crate) fn new(register: RegisterAddress, disp: i16) -> Self {
        Self { register, disp }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum OperandEncoding {
    Accumulator(OperandSize),
    // TODO I don't like strings in ast, but it was fast to implement
    // Remove strings and move Display logic to decoder?
    Jmp { offset: i8, label: String },
    Immediate(i16),
    Register(Register),
}

#[derive(Debug, Clone)]
pub(crate) enum Encoding {
    Empty,
    Operand(OperandEncoding),
    // TODO OperandType is surplus, because it can be infered from operand types
    // If there is a register operand then it's explicit.
    // TODO OperandType can be safely removed in favor of using only explicit size identifiers
    Memory(EffectiveAddress, OperandSize, OperandType),
}

#[derive(Debug, Clone)]
pub enum InstType {
    MOV,
    ADD,
    SUB,
    CMP,
    JNZ,
    JE,
    JL,
    JLE,
    JB,
    JBE,
    JP,
    JO,
    JS,
    JNL,
    JG,
    JNB,
    JA,
    JNP,
    JNO,
    JNS,
    LOOP,
    LOOPZ,
    LOOPNZ,
    JCXZ,
    Label(String),
}

#[derive(Debug, Clone)]
pub struct Inst {
    pub(crate) t: InstType,
    pub(crate) lhs: Encoding,
    pub(crate) rhs: Encoding,
    pub(crate) length: usize,
}

impl Inst {
    pub(crate) fn new(name: InstType, lhs: Encoding, rhs: Encoding, length: usize) -> Self {
        Self {
            t: name,
            lhs,
            rhs,
            length,
        }
    }
}

impl std::fmt::Display for EffectiveAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            if self.disp != 0 && matches!(self.register, RegisterAddress::Empty) {
                self.disp.to_string()
            } else {
                let reg = match self.register {
                    RegisterAddress::Empty => "",
                    RegisterAddress::BXSI => "bx + si",
                    RegisterAddress::BXDI => "bx + di",
                    RegisterAddress::BPSI => "bp + si",
                    RegisterAddress::BPDI => "bp + di",
                    RegisterAddress::SI => "si",
                    RegisterAddress::DI => "di",
                    RegisterAddress::BX => "bx",
                    RegisterAddress::DirectBP => "bp",
                };
                if self.disp == 0 {
                    reg.to_string()
                } else {
                    let sign = match self.disp {
                        1.. => "+",
                        _ => "-",
                    };
                    format!("{} {} {}", reg, sign, self.disp.abs())
                }
            }
        )
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

impl std::fmt::Display for OperandEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Jmp { label: s, .. } => s.to_string(),
                Self::Accumulator(OperandSize::Byte) => "al".to_string(),
                Self::Accumulator(OperandSize::Word) => "ax".to_string(),
                Self::Immediate(e) => e.to_string(),
                Self::Register(r) => r.to_string(),
            }
        )
    }
}

impl std::fmt::Display for OperandSize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Byte => "byte",
                Self::Word => "word",
            }
        )
    }
}

impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Operand(o) => o.to_string(),
                Self::Memory(o, size, OperandType::Explicit) => format!("{} {}", size, o),
                Self::Memory(o, _, OperandType::Implicit) => o.to_string(),
                Self::Empty => "".to_string(),
            }
        )
    }
}

impl std::fmt::Display for InstType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MOV => "mov",
                Self::ADD => "add",
                Self::SUB => "sub",
                Self::CMP => "cmp",
                Self::JNZ => "jnz",
                Self::JE => "je",
                Self::JL => "jl",
                Self::JLE => "jle",
                Self::JB => "jb",
                Self::JBE => "jbe",
                Self::JP => "jp",
                Self::JO => "jo",
                Self::JS => "js",
                Self::JNL => "jnl",
                Self::JG => "jg",
                Self::JNB => "jnb",
                Self::JA => "ja",
                Self::JNP => "jnp",
                Self::JNO => "jno",
                Self::JNS => "jns",
                Self::LOOP => "loop",
                Self::LOOPZ => "loopz",
                Self::LOOPNZ => "loopnz",
                Self::JCXZ => "jcxz",
                Self::Label(s) => s,
            }
        )
    }
}

impl std::fmt::Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if matches!(self.lhs, Encoding::Empty) {
            write!(f, "{}", self.t)
        } else if matches!(self.rhs, Encoding::Empty) {
            write!(f, "{} {}", self.t, self.lhs)
        } else {
            write!(f, "{} {}, {}", self.t, self.lhs, self.rhs)
        }
    }
}
