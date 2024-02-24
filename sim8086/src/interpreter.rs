use crate::ast::{
    EffectiveAddress, Encoding, Inst, InstType, MemoryEncoding, OperandEncoding, Register,
    RegisterAddress,
};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
struct Flags(u16);

macro_rules! bit_field_is {
    ($name:ident, $shift:literal) => {
        fn $name(self) -> bool {
            let shift: u8 = $shift;
            ((self.0 >> shift) & 1) == 1
        }
    };
}

macro_rules! bit_field_unset {
    ($name:ident, $shift:literal) => {
        fn $name(self) -> Self {
            let shift: u8 = $shift;
            Self(self.0 & !(1u16 << shift))
        }
    };
}

macro_rules! bit_field_set {
    ($name:ident, $shift:literal) => {
        fn $name(self) -> Self {
            let shift: u8 = $shift;
            Self(self.0 | (1u16 << shift))
        }
    };
}

impl Flags {
    // https://en.wikipedia.org/wiki/FLAGS_register
    bit_field_is!(is_cf, 0);
    bit_field_set!(set_cf, 0);
    bit_field_unset!(unset_cf, 0);

    bit_field_is!(is_pf, 2);
    bit_field_set!(set_pf, 2);
    bit_field_unset!(unset_pf, 2);

    bit_field_is!(is_af, 4);
    bit_field_set!(set_af, 4);
    bit_field_unset!(unset_af, 4);

    bit_field_is!(is_zf, 6);
    bit_field_set!(set_zf, 6);
    bit_field_unset!(unset_zf, 6);

    bit_field_is!(is_sf, 7);
    bit_field_set!(set_sf, 7);
    bit_field_unset!(unset_sf, 7);
}

impl std::fmt::Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            if self.is_cf() { "C" } else { "" },
            if self.is_pf() { "P" } else { "" },
            if self.is_af() { "A" } else { "" },
            if self.is_zf() { "Z" } else { "" },
            if self.is_sf() { "S" } else { "" },
        )
    }
}

#[derive(Debug, Default)]
pub struct Code {
    insts: Vec<Inst>,
    ip_insts_idx: HashMap<usize, usize>,
}

impl Code {
    fn get_inst(&self, ip: usize) -> Option<Inst> {
        self.ip_insts_idx
            .get(&ip)
            .map(|idx| self.insts[*idx].clone())
    }
}

impl From<Vec<crate::decoder::Asm>> for Code {
    fn from(value: Vec<crate::decoder::Asm>) -> Self {
        Self {
            ip_insts_idx: value
                .iter()
                .enumerate()
                .map(|iasm| (iasm.1.ip, iasm.0))
                .collect(),
            insts: value.into_iter().map(|x| x.decode()).collect(),
        }
    }
}

impl FromIterator<crate::decoder::Asm> for Code {
    fn from_iter<T: IntoIterator<Item = crate::decoder::Asm>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<Vec<crate::decoder::Asm>>())
    }
}

#[derive(Debug)]
struct Step {
    inst: Inst,
    ip: (u16, u16),
    register: Option<(Register, i16, i16)>,
    flags: Option<(Flags, Flags)>,
}

#[derive(Debug, Default)]
pub struct Processor {
    // I didn't bother with cascade behavior of registers,
    // so only 16-bit registers are supported
    ip: u16,
    flags: Flags,
    registers: [i16; 16],
    code: Code,
    memory: Vec<u8>,
    // stack: Vec<u8>,
}

impl Processor {
    pub fn new(code: Code) -> Self {
        Self {
            code: Code::from(code),
            memory: vec![0; 1024 * 1024],
            ..Self::default()
        }
    }
    fn register_value(&self, reg: Register) -> i16 {
        self.registers[reg.to_idx()]
    }

    fn step(&mut self) -> Option<Step> {
        let inst = self.code.get_inst(self.ip as usize)?;
        let from_ip = self.ip;
        let from_flags = self.flags;
        let mut register_update = None;

        match (&inst.t, &inst.lhs, &inst.rhs) {
            (
                InstType::MOV,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                self.registers[reg1.to_idx()] = val;
                let to_reg = self.registers[reg1.to_idx()];

                register_update = Some((reg1, from_reg, to_reg));
            }
            (
                InstType::MOV,
                &Encoding::Word(MemoryEncoding::Memory(address)),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                self.memory[address as usize] = (val & 0xFF) as u8;
                self.memory[address as usize + 1] = ((val >> 8) & 0xFF) as u8;
            }
            (
                InstType::MOV,
                &Encoding::Word(MemoryEncoding::EffectiveAddress(EffectiveAddress {
                    register: RegisterAddress::BX,
                    disp,
                })),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                let address = self.registers[Register::BX.to_idx()] + disp;
                self.memory[address as usize] = (val & 0xFF) as u8;
                self.memory[address as usize + 1] = ((val >> 8) & 0xFF) as u8;
            }
            (
                InstType::MOV,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                self.registers[reg1.to_idx()] = self.registers[reg2.to_idx()];
                let to_reg = self.registers[reg1.to_idx()];

                register_update = Some((reg1, from_reg, to_reg));
            }
            (
                InstType::MOV,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Word(MemoryEncoding::Memory(address)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                let val = ((self.memory[address as usize + 1] as u16) << 8)
                    | self.memory[address as usize] as u16;
                self.registers[reg1.to_idx()] = val as i16;
                let to_reg = self.registers[reg1.to_idx()];

                register_update = Some((reg1, from_reg, to_reg));
            }
            (
                InstType::ADD,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                self.registers[reg1.to_idx()] += val;
                let to_reg = self.registers[reg1.to_idx()];

                register_update = Some((reg1, from_reg, to_reg));
                self.update_flags(from_reg, to_reg);
                self.update_add_flags(from_reg, val);
            }
            (
                InstType::SUB,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                self.registers[reg1.to_idx()] -= self.registers[reg2.to_idx()];
                let to_reg = self.registers[reg1.to_idx()];

                register_update = Some((reg1, from_reg, to_reg));

                self.update_flags(from_reg, to_reg);
                self.update_sub_flags(from_reg, self.registers[reg2.to_idx()]);
            }
            (
                InstType::SUB,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                self.registers[reg1.to_idx()] -= val;
                let to_reg = self.registers[reg1.to_idx()];

                register_update = Some((reg1, from_reg, to_reg));

                self.update_flags(from_reg, to_reg);
                self.update_sub_flags(from_reg, val);
            }
            (
                InstType::CMP,
                Encoding::Operand(OperandEncoding::Register(reg1)),
                Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                let to_reg = self.registers[reg1.to_idx()] - self.registers[reg2.to_idx()];
                self.update_flags(from_reg, to_reg);
            }
            (
                InstType::JNZ,
                &Encoding::Operand(OperandEncoding::Jmp(offset, _)),
                Encoding::Empty,
            ) => {
                if !self.flags.is_zf() {
                    self.ip = (self.ip as i16 + offset as i16) as u16;
                }
            }
            _ => {}
        };

        self.ip += inst.length as u16;

        let mut flag_update = None;
        if self.flags != from_flags {
            flag_update = Some((from_flags, self.flags));
        }

        Some(Step {
            inst,
            ip: (from_ip, self.ip),
            flags: flag_update,
            register: register_update,
        })
    }

    fn update_flags(&mut self, from_val: i16, to_val: i16) {
        if to_val < 0 {
            self.flags = self.flags.set_sf();
        } else {
            self.flags = self.flags.unset_sf();
        }
        if to_val == 0 {
            self.flags = self.flags.set_zf();
        } else {
            self.flags = self.flags.unset_zf();
        }
        if (to_val & 0xFF).count_ones() % 2 == 0 {
            self.flags = self.flags.set_pf();
        } else {
            self.flags = self.flags.unset_pf();
        }
        if (to_val > 0 && from_val < 0) || (to_val < 0 && from_val > 0) {
            self.flags = self.flags.set_cf();
        } else {
            self.flags = self.flags.unset_cf();
        }
    }
    fn update_add_flags(&mut self, from_val: i16, val: i16) {
        if (from_val & 0xF) + (val & 0xF) > 0xF {
            self.flags = self.flags.set_af();
        } else {
            self.flags = self.flags.unset_af();
        }
    }
    fn update_sub_flags(&mut self, from_val: i16, val: i16) {
        if (from_val & 0xF) - (val & 0xF) < 0 {
            self.flags = self.flags.set_af();
        } else {
            self.flags = self.flags.unset_af();
        }
    }
}

#[derive(Default)]
pub struct TracerOptions {
    pub with_ip: bool,
}

#[derive(Default)]
pub struct Tracer {
    opt: TracerOptions,
    registers: HashSet<Register>,
}

impl Tracer {
    pub fn with_options(opt: TracerOptions) -> Self {
        Self {
            opt,
            ..Tracer::default()
        }
    }

    pub fn run(&mut self, processor: &mut Processor) {
        while let Some(step) = processor.step() {
            self.trace(step);
        }
        self.state(processor);
    }

    fn trace(&mut self, step: Step) {
        let mut trace = step.inst.to_string();
        let mut write_trace = |msg| write!(trace, "{}", msg).unwrap();
        let fmt_flags = |from, to| format!(" flags:{}->{}", from, to);
        let fmt_reg = |reg, from, to| format!(" {}:{:#x}->{:#x}", reg, from, to);
        let fmt_ip = |from, to| format!(" ip:{:#x}->{:#x}", from, to);

        write_trace(" ;".to_string());
        match step.register {
            Some((reg, from, to)) if from != to => {
                self.registers.insert(reg);
                write_trace(fmt_reg(reg, from, to));
            }
            _ => {}
        }
        if self.opt.with_ip {
            write_trace(fmt_ip(step.ip.0, step.ip.1));
        }
        if let Some((from, to)) = step.flags {
            write_trace(fmt_flags(from, to));
        }

        println!("{}", trace);
    }

    fn state(&mut self, processor: &Processor) {
        let mut registers = self.registers.iter().map(|x| *x).collect::<Vec<Register>>();
        registers.sort();

        let mut trace = "Final registers:\n".to_string();
        let mut write_trace = |msg| write!(trace, "{}", msg).unwrap();
        for reg in registers {
            write_trace(format!(
                "{:>8}: {:#06x} ({})\n",
                reg.to_string(),
                processor.register_value(reg) as u16,
                processor.register_value(reg) as u16,
            ));
        }

        if self.opt.with_ip {
            write_trace(format!(
                "      ip: {:#06x} ({})\n",
                processor.ip, processor.ip
            ));
        }

        if processor.flags != Flags(0) {
            write_trace(format!("   flags: {}", processor.flags));
        }

        print!("{}", trace);
    }
}
