use std::env::args;

use sim8086;

fn mode_to_write(rm: u8, mode: u8) -> usize {
    use sim8086::{Address, Mode};
    match Mode::from(mode) {
        Mode::Mem0Disp => {
            if let Address::DirectBP = Address::from(rm) {
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

fn mode_encode(data: &Vec<u8>, mode: u8, rm: u8, w: u8) -> sim8086::OperandEncoding {
    use sim8086::{Address, Mode, OperandEncoding};
    match Mode::from(mode) {
        Mode::Reg => OperandEncoding::register(rm, w),
        Mode::Mem0Disp => {
            let address = Address::from(rm);
            if matches!(address, Address::DirectBP) {
                let direct = (data[3] as u16) << 8 | (data[2] as u16);
                OperandEncoding::Memory(direct)
            } else {
                OperandEncoding::effective_address(address, 0)
            }
        }
        Mode::Mem1Disp => {
            let address = Address::from(rm);
            OperandEncoding::effective_address(address, (data[2] as i8) as i16)
        }
        Mode::Mem2Disp => {
            let address = Address::from(rm);
            let disp = (data[3] as i16) << 8 | (data[2] as i16);
            OperandEncoding::effective_address(address, disp)
        }
    }
}

#[derive(Debug)]
struct RM(Vec<u8>);

impl RM {
    const PREFIX: [(&'static str, u8); 4] = [
        ("add", 0b000000),
        ("sub", 0b001010),
        ("mov", 0b100010),
        ("cmp", 0b001110),
    ];

    fn find_name(op: u8) -> Option<&'static str> {
        let op_prefix = op >> 2;
        for (name, prefix) in Self::PREFIX {
            if (op_prefix ^ prefix) == 0 {
                return Some(name);
            }
        }

        None
    }

    fn match_op(op: u8) -> bool {
        Self::find_name(op).is_some()
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

    fn decode(&self) -> sim8086::Inst {
        use sim8086::{Encoding, Inst, OperandEncoding};

        let mut src = OperandEncoding::register(self.reg(), self.w());
        let mut dst = mode_encode(&self.0, self.mode(), self.rm(), self.w());

        if self.d() == 0b1 {
            (src, dst) = (dst, src);
        };

        let name = Self::find_name(self.0[0]).unwrap().to_string();
        Inst::new(name, Encoding::Operand(dst), Encoding::Operand(src))
    }
}

#[derive(Debug)]
enum IRMCommonOpCode {
    TBD,
    Cmp,
    Add,
    Sub,
}

impl IRMCommonOpCode {
    fn with_reg(reg: u8) -> Self {
        if reg ^ 0b000 == 0 {
            IRMCommonOpCode::Add
        } else if reg ^ 0b101 == 0 {
            IRMCommonOpCode::Sub
        } else if reg ^ 0b111 == 0 {
            IRMCommonOpCode::Cmp
        } else {
            unreachable!()
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Cmp => "cmp",
            Self::TBD => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct IRM(Vec<u8>);
enum IRMOpCode {
    Mov,
    Common(IRMCommonOpCode),
}

impl IRMOpCode {
    fn match_op(op: u8) -> bool {
        Self::get(op).is_some()
    }

    fn get(op: u8) -> Option<Self> {
        if (op >> 1) ^ 0b1100011 == 0 {
            Some(Self::Mov)
        } else if (op >> 2) ^ 0b100000 == 0 {
            Some(Self::Common(IRMCommonOpCode::TBD))
        } else {
            None
        }
    }

    fn with_reg(op: u8, reg: u8) -> Self {
        let op = Self::get(op).unwrap();
        match op {
            Self::Mov => op,
            Self::Common(IRMCommonOpCode::TBD) => Self::Common(IRMCommonOpCode::with_reg(reg)),
            _ => unreachable!(),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Mov => "mov",
            Self::Common(common) => common.name(),
        }
    }
}

impl IRM {
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
            self.w(),
            self.s(),
        ) {
            (IRMOpCode::Mov, 1, _) => 2,
            (IRMOpCode::Common(IRMCommonOpCode::Add | IRMCommonOpCode::Sub), 1, 0) => 2,
            (IRMOpCode::Common(IRMCommonOpCode::Cmp), 1, 1) => 2,
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
    fn decode(&self) -> sim8086::Inst {
        use sim8086::{Encoding, Inst, Mode, OperandEncoding};

        let mode = Mode::from(self.mode());
        let data_idx = match mode {
            Mode::Reg | Mode::Mem0Disp => 2,
            Mode::Mem1Disp => 3,
            Mode::Mem2Disp => 4,
        };

        let src = Encoding::Operand(OperandEncoding::Immediate(if self.data_len() == 2 {
            ((self.0[data_idx + 1] as u16) << 8) | self.0[data_idx] as u16
        } else {
            self.0[data_idx] as u16
        }));

        let rm = mode_encode(&self.0, self.mode(), self.rm(), self.w());
        let dst = match (mode, self.s(), self.w()) {
            (Mode::Reg, _, _) => Encoding::Operand(rm),
            (_, 1, 1) => Encoding::Word(rm),
            (_, _, _) => Encoding::Byte(rm),
        };

        if matches!(
            IRMOpCode::with_reg(self.0[0], self.reg()),
            IRMOpCode::Common(IRMCommonOpCode::Sub)
        ) {
            for x in self.0.clone() {
                // print!("{:b} ", x)
            }
        }

        let name = IRMOpCode::with_reg(self.0[0], self.reg())
            .name()
            .to_string();
        Inst::new(name, dst, src)
    }
}

#[derive(Debug)]
struct IR(Vec<u8>);

impl IR {
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
        if self.0.len() == 1 {
            1
        } else if self.w() == 1 && self.0.len() == 2 {
            1
        } else {
            0
        }
    }
    fn push(&mut self, data: u8) {
        self.0.push(data);
    }
    fn decode(&self) -> sim8086::Inst {
        use sim8086::{Encoding, Inst, OperandEncoding};

        let dst = OperandEncoding::register(self.reg(), self.w());
        let src = OperandEncoding::Immediate(if self.w() == 1 {
            ((self.0[2] as u16) << 8) | self.0[1] as u16
        } else {
            self.0[1] as u16
        });

        let name = "mov".to_string();
        Inst::new(name, Encoding::Operand(dst), Encoding::Operand(src))
    }
}

#[derive(Debug)]
struct MovMA(Vec<u8>);

impl MovMA {
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
        if self.0.len() == 1 {
            1
        } else if self.w() == 1 && self.0.len() == 2 {
            1
        } else {
            0
        }
    }
    fn push(&mut self, data: u8) {
        assert!(self.0.len() <= 3);
        self.0.push(data);
    }
    fn decode(&self) -> sim8086::Inst {
        use sim8086::{Encoding, Inst, OperandEncoding};

        let mut dst = OperandEncoding::Accumulator;
        let mut src = OperandEncoding::Memory(if self.w() == 1 {
            ((self.0[2] as u16) << 8) | self.0[1] as u16
        } else {
            self.0[1] as u16
        });

        if self.d() == 1 {
            (dst, src) = (src, dst);
        };

        let name = "mov".to_string();
        Inst::new(name, Encoding::Operand(dst), Encoding::Operand(src))
    }
}

#[derive(Debug)]
enum Mov {
    RM(RM),
    IRM(IRM),
    IR(IR),
    MA(MovMA),
}

impl Mov {
    fn get_mov(first: u8) -> Option<Self> {
        if RM::match_op(first) {
            Some(Self::RM(RM::new(first)))
        } else if ((first >> 4) ^ 0b1011) == 0 {
            Some(Self::IR(IR::new(first)))
        } else if IRM::match_op(first) {
            Some(Self::IRM(IRM::new(first)))
        } else if ((first >> 2) ^ 0b101000) == 0 {
            Some(Self::MA(MovMA::new(first)))
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::RM(r) => r.len(),
            Self::IR(r) => r.len(),
            Self::MA(r) => r.len(),
            Self::IRM(r) => r.len(),
        }
    }

    fn push(&mut self, data: u8) {
        match self {
            Self::RM(r) => r.push(data),
            Self::IR(r) => r.push(data),
            Self::MA(r) => r.push(data),
            Self::IRM(r) => r.push(data),
        }
    }

    fn decode(&self) -> sim8086::Inst {
        match self {
            Self::RM(r) => r.decode(),
            Self::IR(r) => r.decode(),
            Self::MA(r) => r.decode(),
            Self::IRM(r) => r.decode(),
        }
    }
}

fn parse(mut it: impl Iterator<Item = u8>) -> Vec<Result<Mov, sim8086::Error>> {
    let mut ops = vec![];

    while let Some(first) = it.next() {
        let Some(mut mov) = Mov::get_mov(first) else {
            ops.push(Err(sim8086::Error(format!("{:b}", first))));
            // println!("{:b}", first);
            continue;
        };

        loop {
            let w = mov.len();
            if w == 0 {
                break;
            }
            for _ in 0..w {
                let Some(data) = it.next() else {
                    // dbg!(mov);
                    panic!()
                };
                mov.push(data);
            }
        }
        ops.push(Ok(mov));
    }

    ops
}

fn main() {
    let path = args()
        .nth(1)
        .expect("Provide unix path to 8086 binary file");
    let data = std::fs::read(path).expect("Can't open given file");
    for op in parse(data.into_iter()) {
        match op.and_then(|x| Ok(x.decode())) {
            Ok(op) => println!("{}", op.to_string()),
            Err(e) => println!("{}", e.0),
        };
    }
}
