use std::fmt;

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

    fn to_idx(self) -> usize {
        match self {
            Self::AL => 0,
            Self::CL => 1,
            Self::DL => 2,
            Self::BL => 3,
            Self::AH => 4,
            Self::CH => 5,
            Self::DH => 6,
            Self::BH => 7,
            Self::AX => 8,
            Self::CX => 9,
            Self::DX => 10,
            Self::BX => 11,
            Self::SP => 12,
            Self::BP => 13,
            Self::SI => 14,
            Self::DI => 15,
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
    disp: i16,
}

impl EffectiveAddress {
    fn new(address: Address, disp: i16) -> Self {
        Self { address, disp }
    }
}

#[derive(Debug, Clone)]
pub enum OperandEncoding {
    Accumulator8,
    Accumulator16,
    Jmp(String),
    Memory(u16),
    Immediate(i16),
    Register(Register),
    EffectiveAddress(EffectiveAddress),
}

impl OperandEncoding {
    pub fn direct(direct: u16) -> Self {
        Self::Memory(direct)
    }

    pub fn register(reg: u8, w: u8) -> Self {
        Self::Register(Register::from(reg, w))
    }

    pub fn effective_address(address: Address, disp: i16) -> Self {
        Self::EffectiveAddress(EffectiveAddress::new(address, disp))
    }
}

#[derive(Debug, Clone)]
pub enum Encoding {
    Empty,
    Operand(OperandEncoding),
    Byte(OperandEncoding),
    Word(OperandEncoding),
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

pub struct Inst {
    t: InstType,
    lhs: Encoding,
    rhs: Encoding,
}

impl Inst {
    pub fn new(name: InstType, lhs: Encoding, rhs: Encoding) -> Self {
        Self { t: name, lhs, rhs }
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
                1.. => format!(" + {}", self.disp),
                _ => format!(" - {}", -self.disp),
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

impl fmt::Display for OperandEncoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Jmp(e) => e.to_string(),
                Self::Accumulator8 => "al".to_string(),
                Self::Accumulator16 => "ax".to_string(),
                Self::Immediate(e) => e.to_string(),
                Self::Memory(e) => format!("[{}]", e),
                Self::Register(r) => r.to_string(),
                Self::EffectiveAddress(e) => e.to_string(),
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
                Self::Operand(o) => o.to_string(),
                Self::Byte(o) => format!("byte {}", o),
                Self::Word(o) => format!("word {}", o),
                Self::Empty => "".to_string(),
            }
        )
    }
}

impl fmt::Display for InstType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if matches!(self.lhs, Encoding::Empty) {
            write!(f, "{}", self.t)
        } else if matches!(self.rhs, Encoding::Empty) {
            write!(f, "{} {}", self.t, self.lhs)
        } else {
            write!(f, "{} {}, {}", self.t, self.lhs, self.rhs)
        }
    }
}

#[derive(Debug, Default)]
struct Flags {}

#[derive(Debug, Default)]
pub struct Machine {
    registers: [i16; 16],
    // flags: Flags,
    // stack: Vec<u8>,
    // memory: Vec<u8>,
}

impl Machine {
    fn exec(&mut self, inst: Inst) {
        match (inst.t, inst.lhs, inst.rhs) {
            (
                InstType::MOV,
                Encoding::Operand(OperandEncoding::Register(reg)),
                Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                println!(
                    " ; {}:{:#x}->{:#x}",
                    reg.to_string(),
                    self.registers[reg.to_idx()],
                    val
                );
                self.registers[reg.to_idx()] = val;
            }
            _ => {}
        };
    }
}

pub fn trace_exec(m: &mut Machine, inst: Inst) {
    m.exec(inst);
}
