use std::env::args;

#[derive(Debug, Clone, Copy)]
struct Mov(u8, u8);

impl Mov {
    fn w(&self) -> u8 {
        self.0 & 0b1
    }
    fn reg(&self) -> u8 {
        (self.1 >> 3) & 0b111
    }
    fn rm(&self) -> u8 {
        self.1 & 0b111
    }

    fn encode(&self) -> Result<sim8086::Inst, sim8086::Error> {
        let rhs = sim8086::FieldEncoding::from(self.reg(), self.w())?;
        let lhs = sim8086::FieldEncoding::from(self.rm(), self.w())?;
        let name = "mov".to_string();
        Ok(sim8086::Inst::new(name, lhs, rhs))
    }
}

fn is_mov(i: u8) -> bool {
    ((i >> 2) ^ 0b100010) == 0
}

fn parse(mut it: impl Iterator<Item = u8>) -> Vec<Result<Mov, sim8086::Error>> {
    let mut ops = vec![];
    while let Some(first) = it.next() {
        if is_mov(first) {
            let mov = it
                .next()
                .map(|second| Mov(first, second))
                .ok_or(sim8086::Error);
            ops.push(mov)
        } else {
            ops.push(Err(sim8086::Error))
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
        println!(
            "{}",
            op.expect("parse op")
                .encode()
                .expect("encoding")
                .to_string()
        );
    }
}
