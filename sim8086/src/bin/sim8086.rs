use std::{
    collections::{HashMap, HashSet},
    env::args,
};

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
struct JMP(Vec<u8>, String);
impl JMP {
    const PREFIX: [(&'static str, u8); 20] = [
        ("jnz", 0b01110101),
        ("je", 0b01110100),
        ("jl", 0b01111100),
        ("jle", 0b01111110),
        ("jb", 0b01110010),
        ("jbe", 0b01110110),
        ("jp", 0b01111010),
        ("jo", 0b01110000),
        ("js", 0b01111000),
        ("jnl", 0b01111101),
        ("jg", 0b01111111),
        ("jnb", 0b01110011),
        ("ja", 0b01110111),
        ("jnp", 0b01111011),
        ("jno", 0b01110001),
        ("jns", 0b01111001),
        ("loop", 0b11100010),
        ("loopz", 0b11100001),
        ("loopnz", 0b11100000),
        ("jcxz", 0b11100011),
    ];

    fn find_name(op: u8) -> Option<&'static str> {
        for (name, prefix) in Self::PREFIX {
            if (op ^ prefix) == 0 {
                return Some(name);
            }
        }

        None
    }

    fn match_op(op: u8) -> bool {
        Self::find_name(op).is_some()
    }

    fn new(op: u8) -> Self {
        let mut v = Vec::with_capacity(2);
        v.push(op);
        Self(v, "".to_string())
    }

    fn get_offset(&self) -> i8 {
        self.0[1] as i8
    }

    fn set_label(&mut self, label: String) {
        self.1 = label
    }

    fn len(&self) -> usize {
        if self.0.len() == 1 {
            return 1;
        }

        0
    }

    fn push(&mut self, data: u8) {
        assert!(self.0.len() < 2);
        self.0.push(data);
    }

    fn decode(&self) -> sim8086::Inst {
        use sim8086::{Encoding, Inst, OperandEncoding};

        let src = Encoding::Empty;
        let dst = Encoding::Operand(OperandEncoding::Jmp(self.1.clone()));

        let name = Self::find_name(self.0[0]).unwrap().to_string();
        Inst::new(name, dst, src)
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
struct IRM(Vec<u8>);
enum IRMOpCode {
    Mov,
    TBD,
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
            Some(Self::TBD)
        } else {
            None
        }
    }

    fn with_reg(op: u8, reg: u8) -> Self {
        let op = Self::get(op).unwrap();
        match op {
            Self::Mov => op,
            Self::TBD => {
                if reg ^ 0b000 == 0 {
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

    fn name(self) -> &'static str {
        match self {
            Self::Mov => "mov",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Cmp => "cmp",
            Self::TBD => unreachable!(),
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
    fn decode(&self) -> sim8086::Inst {
        use sim8086::{Encoding, Inst, Mode, OperandEncoding};

        let data_idx = 2 + mode_to_write(self.rm(), self.mode());
        let src = Encoding::Operand(OperandEncoding::Immediate(if self.data_len() == 2 {
            ((self.0[data_idx + 1] as i16) << 8) | self.0[data_idx] as i16
        } else {
            (self.0[data_idx] as i8) as i16
        }));

        let mode = Mode::from(self.mode());
        let rm = mode_encode(&self.0, self.mode(), self.rm(), self.w());
        let dst = match (mode, self.s(), self.w()) {
            (Mode::Reg, _, _) => Encoding::Operand(rm),
            (_, 1, 1) => Encoding::Word(rm),
            (_, _, _) => Encoding::Byte(rm),
        };

        let name = IRMOpCode::with_reg(self.0[0], self.reg())
            .name()
            .to_string();
        Inst::new(name, dst, src)
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
            ((self.0[2] as i16) << 8) | self.0[1] as i16
        } else {
            (self.0[1] as i8) as i16
        });

        let name = "mov".to_string();
        Inst::new(name, Encoding::Operand(dst), Encoding::Operand(src))
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
    fn name(self) -> &'static str {
        match self {
            Self::Mov => "mov",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Cmp => "cmp",
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
        let is_wide = self.w() == 1;

        let (mut dst, src_val) = if is_wide {
            let mem = ((self.0[2] as i16) << 8) | self.0[1] as i16;
            (OperandEncoding::Accumulator16, mem)
        } else {
            let mem = (self.0[1] as i8) as i16;
            (OperandEncoding::Accumulator8, mem)
        };

        let op_code = MAOpCode::from(self.0[0]).unwrap();
        let src = if matches!(op_code, MAOpCode::Mov) {
            let mut src = OperandEncoding::Memory(src_val as u16);
            if self.d() == 1 {
                (dst, src) = (src, dst);
            };

            src
        } else {
            OperandEncoding::Immediate(src_val)
        };

        let name = op_code.name().to_string();
        Inst::new(name, Encoding::Operand(dst), Encoding::Operand(src))
    }
}

#[derive(Debug)]
enum AsmOp {
    RM(RM),
    IRM(IRM),
    IR(IR),
    MA(MA),
    JMP(JMP),
    Label(usize),
}

impl AsmOp {
    fn len(&self) -> usize {
        match self {
            Self::RM(r) => r.len(),
            Self::IR(r) => r.len(),
            Self::MA(r) => r.len(),
            Self::IRM(r) => r.len(),
            Self::JMP(r) => r.len(),
            Self::Label(_) => 0,
        }
    }

    fn push(&mut self, data: u8) {
        match self {
            Self::RM(r) => r.push(data),
            Self::IR(r) => r.push(data),
            Self::MA(r) => r.push(data),
            Self::IRM(r) => r.push(data),
            Self::JMP(r) => r.push(data),
            Self::Label(_) => panic!("cant push"),
        }
    }

    fn decode(&self) -> sim8086::Inst {
        match self {
            Self::RM(r) => r.decode(),
            Self::IR(r) => r.decode(),
            Self::MA(r) => r.decode(),
            Self::IRM(r) => r.decode(),
            Self::JMP(r) => r.decode(),
            Self::Label(s) => sim8086::Inst::new_label(format!("label_{}:", s)),
        }
    }
}

#[derive(Debug)]
struct Asm {
    ip: usize,
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
            } else if IRM::match_op(op) {
                AsmOp::IRM(IRM::new(op))
            } else if MA::match_op(op) {
                AsmOp::MA(MA::new(op))
            } else if JMP::match_op(op) {
                AsmOp::JMP(JMP::new(op))
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

    fn decode(&self) -> sim8086::Inst {
        self.op.decode()
    }
}

fn parse(it: impl Iterator<Item = u8>) -> Vec<Result<Asm, String>> {
    let mut ops = vec![];
    let mut it = it.enumerate();
    let mut existed_labels = HashMap::new();

    while let Some((mut ip, first)) = it.next() {
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
                let Some((i, data)) = it.next() else {
                    dbg!(asm);
                    panic!()
                };
                ip = i;
                asm.push(data);
            }
        }

        // ugly label handling code
        if let AsmOp::JMP(jump) = &mut asm.op {
            let offset = jump.get_offset();
            let label_ip = (ip as i64 + offset as i64) as usize;

            let label_number = existed_labels.len() + 1;
            existed_labels.entry(label_ip).or_insert_with(|| {
                ops.push(Ok(Asm {
                    ip: label_ip,
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

#[derive(Debug, Default)]
struct Args {
    flags: HashSet<String>,
    path: String,
}

fn main() {
    let args = args()
        .into_iter()
        .skip(1)
        .try_fold(Args::default(), |mut args, s| {
            if s.starts_with("--") {
                args.flags.insert(s.trim_start_matches("--").to_string());
            } else if args.path == "" {
                args.path = s.to_string();
            } else {
                return Err("You can't have multiple paths");
            }

            Ok(args)
        })
        .expect("Provide unix path to 8086 binary file");

    let data = std::fs::read(args.path).expect("Can't open given file");
    let asm = parse(data.into_iter());

    if args.flags.is_empty() {
        for op in asm {
            match op.and_then(|x| Ok(x.decode())) {
                Ok(op) => println!("{}", op),
                Err(e) => println!("{}", e),
            };
        }
    }
}
