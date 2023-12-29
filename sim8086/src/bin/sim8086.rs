use std::env::args;

use sim8086;

#[derive(Debug, Clone)]
struct Mov(Vec<u8>);

impl Mov {
    fn new(first: u8, second: u8) -> Self {
        let mut v = Vec::with_capacity(6);
        v.push(first);
        v.push(second);
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
        use sim8086::{Address, Mode};
        match Mode::from(self.mode()) {
            Mode::Mem0Disp => {
                if let Address::DirectBP = Address::from(self.rm()) {
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

    fn write(&mut self, data: u8) {
        assert!(self.0.len() <= 6);
        self.0.push(data);
    }

    fn encode(&self) -> sim8086::Inst {
        use sim8086::*;

        let mut src = Encoding::register(self.reg(), self.w());
        let mut dst = match Mode::from(self.mode()) {
            Mode::Reg => Encoding::register(self.rm(), self.w()),
            Mode::Mem0Disp => {
                let address = Address::from(self.rm());
                let mut direct = None;
                if matches!(address, Address::DirectBP) {
                    direct = Some((self.0[3] as u16) << 8 | (self.0[2] as u16));
                }
                Encoding::effective_address(address, direct, None)
            }
            Mode::Mem1Disp => {
                let address = Address::from(self.rm());
                Encoding::effective_address(address, None, Some(self.0[2] as u16))
            }
            Mode::Mem2Disp => {
                let address = Address::from(self.rm());
                let disp = (self.0[3] as u16) << 8 | (self.0[2] as u16);
                Encoding::effective_address(address, None, Some(disp))
            }
        };

        if self.d() == 0b1 {
            (src, dst) = (dst, src);
        };

        let name = "mov".to_string();
        sim8086::Inst::new(name, dst, src)
    }
}

fn is_mov(i: u8) -> bool {
    ((i >> 2) ^ 0b100010) == 0
}

fn parse(mut it: impl Iterator<Item = u8>) -> Vec<Result<Mov, sim8086::Error>> {
    let mut ops = vec![];

    while let Some(first) = it.next() {
        if is_mov(first) {
            let second = it.next().unwrap();
            let mut mov = Mov::new(first, second);
            for _ in 0..mov.to_write() {
                let Some(data) = it.next() else { panic!() };
                mov.write(data);
            }
            ops.push(Ok(mov));
        } else {
            ops.push(Err(sim8086::Error));
        }
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
