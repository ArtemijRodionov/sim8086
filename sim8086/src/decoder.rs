use crate::ast::{
    EffectiveAddress, Encoding, Inst, InstType, Mode, OperandEncoding, OperandSize, OperandType,
    Register, RegisterAddress,
};
use std::collections::HashMap;

fn mode_to_write(rm: u8, mode: u8) -> usize {
    match Mode::from(mode) {
        Mode::Mem0Disp => {
            if let RegisterAddress::DirectBP = RegisterAddress::from(rm) {
                2
            } else {
                0
            }
        }
        Mode::Mem1Disp => 1,
        Mode::Mem2Disp => 2,
        _ => 0,
    }
}

fn mode_encode(
    data: &[u8],
    mode: u8,
    rm: u8,
    w: u8,
    size: OperandSize,
    t: OperandType,
) -> Encoding {
    let mode = Mode::from(mode);
    if let Mode::Reg = mode {
        return Encoding::Operand(OperandEncoding::Register(Register::from(rm, w)));
    }

    let memory = match mode {
        Mode::Mem0Disp => {
            let address = RegisterAddress::from(rm);
            if matches!(address, RegisterAddress::DirectBP) {
                let direct = (data[3] as u16) << 8 | (data[2] as u16);
                EffectiveAddress::new(RegisterAddress::Empty, direct as i16)
            } else {
                EffectiveAddress::new(address, 0)
            }
        }
        Mode::Mem1Disp => {
            let address = RegisterAddress::from(rm);
            let disp = (data[2] as i8) as i16;
            EffectiveAddress::new(address, disp)
        }
        Mode::Mem2Disp => {
            let address = RegisterAddress::from(rm);
            let disp = (data[3] as i16) << 8 | (data[2] as i16);
            EffectiveAddress::new(address, disp)
        }
        _ => unreachable!(),
    };

    Encoding::Memory(memory, size, t)
}

#[derive(Debug)]
struct JP {
    data: Vec<u8>,
    label: String,
}
impl JP {
    const PREFIX: [(InstType, u8); 20] = [
        (InstType::JNZ, 0b01110101),
        (InstType::JE, 0b01110100),
        (InstType::JL, 0b01111100),
        (InstType::JLE, 0b01111110),
        (InstType::JB, 0b01110010),
        (InstType::JBE, 0b01110110),
        (InstType::JP, 0b01111010),
        (InstType::JO, 0b01110000),
        (InstType::JS, 0b01111000),
        (InstType::JNL, 0b01111101),
        (InstType::JG, 0b01111111),
        (InstType::JNB, 0b01110011),
        (InstType::JA, 0b01110111),
        (InstType::JNP, 0b01111011),
        (InstType::JNO, 0b01110001),
        (InstType::JNS, 0b01111001),
        (InstType::LOOP, 0b11100010),
        (InstType::LOOPZ, 0b11100001),
        (InstType::LOOPNZ, 0b11100000),
        (InstType::JCXZ, 0b11100011),
    ];

    fn inst_type(op: u8) -> Option<InstType> {
        for (name, prefix) in Self::PREFIX {
            if (op ^ prefix) == 0 {
                return Some(name);
            }
        }

        None
    }

    fn match_op(op: u8) -> bool {
        Self::inst_type(op).is_some()
    }

    fn new(op: u8) -> Self {
        let mut v = Vec::with_capacity(2);
        v.push(op);
        Self {
            data: v,
            label: "".to_string(),
        }
    }

    fn get_offset(&self) -> i8 {
        self.data.len() as i8 + self.data[1] as i8
    }

    fn set_label(&mut self, label: String) {
        self.label = label
    }

    fn len(&self) -> usize {
        if self.data.len() == 1 {
            return 1;
        }

        0
    }

    fn push(&mut self, data: u8) {
        assert!(self.data.len() < 2);
        self.data.push(data);
    }

    fn decode(&self) -> Inst {
        let src = Encoding::Empty;
        let dst = Encoding::Operand(OperandEncoding::Jmp {
            offset: self.data[1] as i8,
            label: self.label.to_string(),
        });

        let name = Self::inst_type(self.data[0]).unwrap();
        Inst::new(name, dst, src, self.data.len())
    }
}

#[derive(Debug)]
struct RM(Vec<u8>);
impl RM {
    const PREFIX: [(InstType, u8); 4] = [
        (InstType::ADD, 0b000000),
        (InstType::SUB, 0b001010),
        (InstType::MOV, 0b100010),
        (InstType::CMP, 0b001110),
    ];

    fn inst_type(op: u8) -> Option<InstType> {
        let op_prefix = op >> 2;
        for (name, prefix) in Self::PREFIX {
            if (op_prefix ^ prefix) == 0 {
                return Some(name);
            }
        }

        None
    }

    fn match_op(op: u8) -> bool {
        Self::inst_type(op).is_some()
    }

    fn new(first: u8) -> Self {
        let mut v = Vec::with_capacity(4);
        v.push(first);
        Self(v)
    }
    fn w(&self) -> u8 {
        self.0[0] & 0b1
    }
    fn d(&self) -> u8 {
        (self.0[0] >> 1) & 0b1
    }
    fn mode(&self) -> u8 {
        (self.0[1] >> 6) & 0b11
    }
    fn reg(&self) -> u8 {
        (self.0[1] >> 3) & 0b111
    }
    fn rm(&self) -> u8 {
        self.0[1] & 0b111
    }

    fn len(&self) -> usize {
        if self.0.len() == 1 {
            return 1;
        }

        if 2 == self.0.len() {
            mode_to_write(self.rm(), self.mode())
        } else {
            0
        }
    }

    fn push(&mut self, data: u8) {
        assert!(self.0.len() <= 6);
        self.0.push(data);
    }

    fn decode(&self) -> Inst {
        let register = Register::from(self.reg(), self.w());
        let mut src = Encoding::Operand(OperandEncoding::Register(register));
        let mut dst = mode_encode(
            &self.0,
            self.mode(),
            self.rm(),
            self.w(),
            register.size(),
            OperandType::Implicit,
        );

        if self.d() == 0b1 {
            (src, dst) = (dst, src);
        };

        let name = Self::inst_type(self.0[0]).unwrap();
        Inst::new(name, dst, src, self.0.len())
    }
}

#[derive(Debug)]
struct IM(Vec<u8>);
enum IRMOpCode {
    Empty,
    Mov,
    Cmp,
    Add,
    Sub,
}

impl IRMOpCode {
    fn match_op(op: u8) -> bool {
        Self::get(op).is_some()
    }

    fn get(op: u8) -> Option<Self> {
        if (op >> 1) ^ 0b1100011 == 0 {
            Some(Self::Mov)
        } else if (op >> 2) ^ 0b100000 == 0 {
            Some(Self::Empty)
        } else {
            None
        }
    }

    fn with_reg(op: u8, reg: u8) -> Self {
        let op = Self::get(op).unwrap();
        match op {
            Self::Mov => op,
            Self::Empty => {
                if reg == 0 {
                    Self::Add
                } else if reg ^ 0b101 == 0 {
                    Self::Sub
                } else if reg ^ 0b111 == 0 {
                    Self::Cmp
                } else {
                    unreachable!()
                }
            }
            _ => unreachable!(),
        }
    }

    fn inst_type(self) -> InstType {
        match self {
            Self::Mov => InstType::MOV,
            Self::Add => InstType::ADD,
            Self::Sub => InstType::SUB,
            Self::Cmp => InstType::CMP,
            Self::Empty => unreachable!(),
        }
    }
}

impl IM {
    fn match_op(op: u8) -> bool {
        IRMOpCode::match_op(op)
    }
    fn new(first: u8) -> Self {
        let mut v = Vec::with_capacity(6);
        v.push(first);
        Self(v)
    }
    fn w(&self) -> u8 {
        self.0[0] & 0b1
    }
    fn reg(&self) -> u8 {
        (self.0[1] >> 3) & 0b111
    }
    fn s(&self) -> u8 {
        (self.0[0] >> 1) & 0b1
    }
    fn rm(&self) -> u8 {
        self.0[1] & 0b111
    }
    fn mode(&self) -> u8 {
        (self.0[1] >> 6) & 0b11
    }
    fn data_len(&self) -> usize {
        match (
            IRMOpCode::with_reg(self.0[0], self.reg()),
            self.s(),
            self.w(),
        ) {
            (IRMOpCode::Mov, _, 1) => 2,
            (IRMOpCode::Add | IRMOpCode::Sub | IRMOpCode::Cmp, 0, 1) => 2,
            (_, _, _) => 1,
        }
    }
    fn len(&self) -> usize {
        if self.0.len() == 1 {
            1
        } else if self.0.len() == 2 {
            let rm_len = mode_to_write(self.rm(), self.mode());
            rm_len + self.data_len()
        } else {
            0
        }
    }
    fn push(&mut self, data: u8) {
        assert!(self.0.len() < 6);
        self.0.push(data);
    }
    fn decode(&self) -> Inst {
        let data_idx = 2 + mode_to_write(self.rm(), self.mode());
        let src = Encoding::Operand(OperandEncoding::Immediate(if self.data_len() == 2 {
            ((self.0[data_idx + 1] as i16) << 8) | self.0[data_idx] as i16
        } else {
            (self.0[data_idx] as i8) as i16
        }));

        let mode = Mode::from(self.mode());
        let size = match (mode, self.s(), self.w()) {
            (_, 1, 1) => OperandSize::Word,
            (_, _, _) => OperandSize::Byte,
        };

        let dst = mode_encode(
            &self.0,
            self.mode(),
            self.rm(),
            self.w(),
            size,
            OperandType::Explicit,
        );

        let name = IRMOpCode::with_reg(self.0[0], self.reg()).inst_type();
        Inst::new(name, dst, src, self.0.len())
    }
}

#[derive(Debug)]
struct IR(Vec<u8>);
impl IR {
    fn match_op(op: u8) -> bool {
        ((op >> 4) ^ 0b1011) == 0
    }
    fn new(first: u8) -> Self {
        let mut v = Vec::with_capacity(3);
        v.push(first);
        Self(v)
    }
    fn w(&self) -> u8 {
        (self.0[0] >> 3) & 0b1
    }
    fn reg(&self) -> u8 {
        self.0[0] & 0b111
    }
    fn len(&self) -> usize {
        if self.0.len() == 1 || self.w() == 1 && self.0.len() == 2 {
            1
        } else {
            0
        }
    }
    fn push(&mut self, data: u8) {
        self.0.push(data);
    }
    fn decode(&self) -> Inst {
        let dst = OperandEncoding::Register(Register::from(self.reg(), self.w()));
        let src = OperandEncoding::Immediate(if self.w() == 1 {
            ((self.0[2] as i16) << 8) | self.0[1] as i16
        } else {
            (self.0[1] as i8) as i16
        });

        let inst_type = InstType::MOV;
        Inst::new(
            inst_type,
            Encoding::Operand(dst),
            Encoding::Operand(src),
            self.0.len(),
        )
    }
}

enum MAOpCode {
    Mov,
    Add,
    Sub,
    Cmp,
}
impl MAOpCode {
    fn from(op: u8) -> Option<Self> {
        if ((op >> 2) ^ 0b101000) == 0 {
            Some(Self::Mov)
        } else if ((op >> 1) ^ 0b0000010) == 0 {
            Some(Self::Add)
        } else if ((op >> 1) ^ 0b0010110) == 0 {
            Some(Self::Sub)
        } else if ((op >> 1) ^ 0b0011110) == 0 {
            Some(Self::Cmp)
        } else {
            None
        }
    }
    fn inst_type(self) -> InstType {
        match self {
            Self::Mov => InstType::MOV,
            Self::Add => InstType::ADD,
            Self::Sub => InstType::SUB,
            Self::Cmp => InstType::CMP,
        }
    }
}

#[derive(Debug)]
struct MA(Vec<u8>);

impl MA {
    fn match_op(op: u8) -> bool {
        MAOpCode::from(op).is_some()
    }
    fn new(first: u8) -> Self {
        let mut v = Vec::with_capacity(3);
        v.push(first);
        Self(v)
    }
    fn w(&self) -> u8 {
        self.0[0] & 0b1
    }
    fn d(&self) -> u8 {
        (self.0[0] >> 1) & 0b1
    }
    fn len(&self) -> usize {
        if self.0.len() == 1 || self.w() == 1 && self.0.len() == 2 {
            1
        } else {
            0
        }
    }
    fn push(&mut self, data: u8) {
        assert!(self.0.len() <= 3);
        self.0.push(data);
    }
    fn decode(&self) -> Inst {
        let is_wide = self.w() == 1;

        let (dst, src_val) = if is_wide {
            let mem = ((self.0[2] as i16) << 8) | self.0[1] as i16;
            (OperandEncoding::Accumulator(OperandSize::Word), mem)
        } else {
            let mem = (self.0[1] as i8) as i16;
            (OperandEncoding::Accumulator(OperandSize::Byte), mem)
        };
        let mut dst = Encoding::Operand(dst);

        let op_code = MAOpCode::from(self.0[0]).unwrap();
        let src = if matches!(op_code, MAOpCode::Mov) {
            let mut src = Encoding::Memory(
                EffectiveAddress::new(RegisterAddress::Empty, src_val),
                OperandSize::Word,
                OperandType::Implicit,
            );
            if self.d() == 1 {
                (dst, src) = (src, dst);
            };

            src
        } else {
            Encoding::Operand(OperandEncoding::Immediate(src_val))
        };

        let inst_type = op_code.inst_type();
        Inst::new(inst_type, dst, src, self.0.len())
    }
}

#[derive(Debug)]
enum AsmOp {
    RM(RM),
    IM(IM),
    IR(IR),
    MA(MA),
    JP(JP),
    Label(usize),
}

impl AsmOp {
    fn len(&self) -> usize {
        match self {
            Self::RM(r) => r.len(),
            Self::IR(r) => r.len(),
            Self::MA(r) => r.len(),
            Self::IM(r) => r.len(),
            Self::JP(r) => r.len(),
            Self::Label(_) => 0,
        }
    }

    fn push(&mut self, data: u8) {
        match self {
            Self::RM(r) => r.push(data),
            Self::IR(r) => r.push(data),
            Self::MA(r) => r.push(data),
            Self::IM(r) => r.push(data),
            Self::JP(r) => r.push(data),
            Self::Label(_) => panic!("cant push"),
        }
    }

    fn decode(&self) -> Inst {
        match self {
            Self::RM(r) => r.decode(),
            Self::IR(r) => r.decode(),
            Self::MA(r) => r.decode(),
            Self::IM(r) => r.decode(),
            Self::JP(r) => r.decode(),
            Self::Label(s) => Inst::new(
                InstType::Label(format!("label_{}:", s)),
                Encoding::Empty,
                Encoding::Empty,
                0,
            ),
        }
    }
}

#[derive(Debug)]
pub struct Asm {
    pub ip: usize,
    op: AsmOp,
}

impl Asm {
    fn new(ip: usize, op: u8) -> Option<Self> {
        Some(Asm {
            ip,
            op: if RM::match_op(op) {
                AsmOp::RM(RM::new(op))
            } else if IR::match_op(op) {
                AsmOp::IR(IR::new(op))
            } else if IM::match_op(op) {
                AsmOp::IM(IM::new(op))
            } else if MA::match_op(op) {
                AsmOp::MA(MA::new(op))
            } else if JP::match_op(op) {
                AsmOp::JP(JP::new(op))
            } else {
                return None;
            },
        })
    }

    fn len(&self) -> usize {
        self.op.len()
    }

    fn push(&mut self, data: u8) {
        self.op.push(data)
    }

    pub fn decode(&self) -> Inst {
        self.op.decode()
    }
}

pub fn decode(it: impl Iterator<Item = u8>) -> Vec<Result<Asm, String>> {
    let mut ops = vec![];
    let mut it = it.enumerate();
    let mut existed_labels = HashMap::new();

    while let Some((ip, first)) = it.next() {
        let Some(mut asm) = Asm::new(ip, first) else {
            ops.push(Err(format!("{:b}", first)));
            continue;
        };

        loop {
            let w = asm.len();
            if w == 0 {
                break;
            }
            for _ in 0..w {
                let Some((_, data)) = it.next() else {
                    dbg!(asm);
                    panic!()
                };
                asm.push(data);
            }
        }

        // ugly label handling code
        if let AsmOp::JP(jump) = &mut asm.op {
            let offset = jump.get_offset();
            let label_ip = (ip as i64 + offset as i64) as usize;

            let label_number = existed_labels.len() + 1;
            existed_labels.entry(label_ip).or_insert_with(|| {
                ops.push(Ok(Asm {
                    ip: label_ip - 1,
                    op: AsmOp::Label(label_number),
                }));
                label_number
            });

            let label_name = format!("label_{}", existed_labels[&label_ip]);
            jump.set_label(label_name.clone());
        };
        ops.push(Ok(asm));
    }

    ops.sort_by(|a, b| match (a, b) {
        (Ok(a), Ok(b)) => a.ip.cmp(&b.ip),
        _ => std::cmp::Ordering::Equal,
    });

    ops
}
