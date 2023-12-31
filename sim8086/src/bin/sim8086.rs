use std::env::args;

use sim8086;

fn mov_mode_to_write(rm: u8, mode: u8) -> usize {
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

fn mov_mode_encode(data: &Vec<u8>, mode: u8, rm: u8, w: u8) -> sim8086::Encoding {
    use sim8086::{Address, Encoding, Mode};
    match Mode::from(mode) {
        Mode::Reg => Encoding::register(rm, w),
        Mode::Mem0Disp => {
            let address = Address::from(rm);
            if matches!(address, Address::DirectBP) {
                let direct = (data[3] as u16) << 8 | (data[2] as u16);
                Encoding::Direct(direct)
            } else {
                Encoding::effective_address(address, 0)
            }
        }
        Mode::Mem1Disp => {
            let address = Address::from(rm);
            Encoding::effective_address(address, data[2] as u16)
        }
        Mode::Mem2Disp => {
            let address = Address::from(rm);
            let disp = (data[3] as u16) << 8 | (data[2] as u16);
            Encoding::effective_address(address, disp)
        }
    }
}

#[derive(Debug)]
struct MovRM(Vec<u8>);

impl MovRM {
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

    fn to_write(&self) -> usize {
        if self.0.len() == 1 {
            return 1;
        }

        if 2 == self.0.len() {
            mov_mode_to_write(self.rm(), self.mode())
        } else {
            0
        }
    }

    fn write(&mut self, data: u8) {
        assert!(self.0.len() <= 6);
        self.0.push(data);
    }

    fn encode(&self) -> sim8086::Inst {
        use sim8086::{Encoding, Inst};

        let mut src = Encoding::register(self.reg(), self.w());
        let mut dst = mov_mode_encode(&self.0, self.mode(), self.rm(), self.w());

        if self.d() == 0b1 {
            (src, dst) = (dst, src);
        };

        let name = "mov".to_string();
        Inst::new(name, dst, src)
    }
}

#[derive(Debug)]
struct MovIRM(Vec<u8>);

impl MovIRM {
    fn new(first: u8) -> Self {
        let mut v = Vec::with_capacity(6);
        v.push(first);
        Self(v)
    }
    fn w(&self) -> u8 {
        self.0[0] & 0b1
    }
    fn rm(&self) -> u8 {
        self.0[1] & 0b111
    }
    fn mode(&self) -> u8 {
        (self.0[1] >> 6) & 0b11
    }
    fn to_write(&self) -> usize {
        if self.0.len() == 1 {
            1
        } else if self.0.len() == 2 {
            let rm_len = mov_mode_to_write(self.rm(), self.mode());
            let data_len = if self.w() == 1 { 2 } else { 1 };
            rm_len + data_len
        } else {
            0
        }
    }
    fn write(&mut self, data: u8) {
        assert!(self.0.len() < 6);
        self.0.push(data);
    }
    fn encode(&self) -> sim8086::Inst {
        use sim8086::{Encoding, Inst, Mode};
        let data_idx = match Mode::from(self.mode()) {
            Mode::Reg | Mode::Mem0Disp => 2,
            Mode::Mem1Disp => 3,
            Mode::Mem2Disp => 4,
        };
        let src = Encoding::immediate(if self.w() == 1 {
            ((self.0[data_idx + 1] as u16) << 8) | self.0[data_idx] as u16
        } else {
            self.0[data_idx] as u16
        });
        let dst = mov_mode_encode(&self.0, self.mode(), self.rm(), self.w());

        let name = "mov".to_string();
        Inst::new(name, dst, src)
    }
}

#[derive(Debug)]
struct MovIR(Vec<u8>);

impl MovIR {
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
    fn to_write(&self) -> usize {
        if self.0.len() == 1 {
            1
        } else if self.w() == 1 && self.0.len() == 2 {
            1
        } else {
            0
        }
    }
    fn write(&mut self, data: u8) {
        self.0.push(data);
    }
    fn encode(&self) -> sim8086::Inst {
        use sim8086::{Encoding, Inst};

        let dst = Encoding::register(self.reg(), self.w());
        let src = Encoding::immediate(if self.w() == 1 {
            ((self.0[2] as u16) << 8) | self.0[1] as u16
        } else {
            self.0[1] as u16
        });

        let name = "mov".to_string();
        Inst::new(name, dst, src)
    }
}

#[derive(Debug)]
enum Mov {
    RM(MovRM),
    IRM(MovIRM),
    IR(MovIR),
}

impl Mov {
    fn get_mov(first: u8) -> Option<Self> {
        if ((first >> 2) ^ 0b100010) == 0 {
            Some(Self::RM(MovRM::new(first)))
        } else if ((first >> 4) ^ 0b1011) == 0 {
            Some(Self::IR(MovIR::new(first)))
        } else if ((first >> 1) ^ 0b1100011) == 0 {
            Some(Self::IRM(MovIRM::new(first)))
        } else {
            None
        }
    }

    fn to_write(&self) -> usize {
        match self {
            Self::RM(r) => r.to_write(),
            Self::IR(r) => r.to_write(),
            Self::IRM(r) => r.to_write(),
        }
    }

    fn write(&mut self, data: u8) {
        match self {
            Self::RM(r) => r.write(data),
            Self::IR(r) => r.write(data),
            Self::IRM(r) => r.write(data),
        }
    }

    fn encode(&self) -> sim8086::Inst {
        match self {
            Self::RM(r) => r.encode(),
            Self::IR(r) => r.encode(),
            Self::IRM(r) => r.encode(),
        }
    }
}

fn parse(mut it: impl Iterator<Item = u8>) -> Vec<Result<Mov, sim8086::Error>> {
    let mut ops = vec![];

    while let Some(first) = it.next() {
        let Some(mut mov) = Mov::get_mov(first) else {
            ops.push(Err(sim8086::Error));
            println!("{:b}", first);
            continue;
        };

        loop {
            let w = mov.to_write();
            if w == 0 {
                break;
            }
            for _ in 0..w {
                let Some(data) = it.next() else {
                    dbg!(mov);
                    panic!()
                };
                mov.write(data);
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
        match op.map_err(|_| "").and_then(|x| Ok(x.encode())) {
            Ok(op) => println!("{}", op.to_string()),
            Err(_) => continue,
        };
    }
}
