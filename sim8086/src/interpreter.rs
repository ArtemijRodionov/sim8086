use crate::ast::{
    EffectiveAddress, Encoding, Inst, InstType, MemoryEncoding, OperandEncoding, OperandSize,
    Register, RegisterAddress,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
struct Registers(u128);
impl Registers {
    fn load(self, reg: Register) -> i16 {
        let reg_size = reg.size().size();
        let reg_idx = reg.to_idx() as u8;
        let reg_mask = (1 << reg_size) - 1;
        ((self.0 >> (reg_size * reg_idx)) & reg_mask) as i16
    }
    fn store(self, reg: Register, val: i16) -> Registers {
        let val = (val as u16) as u128;
        let reg_size = reg.size().size();
        let reg_idx = reg.to_idx() as u8;

        let left = self.0 & (((1 << ((7 - reg_idx) * reg_size)) - 1) << ((reg_idx + 1) * reg_size));
        let mid = val << (reg_idx * reg_size);
        let right = self.0 & ((1 << (reg_idx * reg_size)) - 1);
        Self(left | mid | right)
    }
}

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

#[derive(Debug, Default)]
struct Clock {
    value: u8,
    ea: u8,
}

#[derive(Debug)]
struct Step {
    inst: Inst,
    ip: (u16, u16),
    register: Option<(Register, i16, i16)>,
    flags: Option<(Flags, Flags)>,
    clock: Clock,
}

#[derive(Debug, Default)]
pub struct Processor {
    ip: u16,
    flags: Flags,
    registers: Registers,
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

    fn run(&mut self) {
        while let Some(_) = self.step() {}
    }

    fn step(&mut self) -> Option<Step> {
        let inst = self.code.get_inst(self.ip as usize)?;
        let from_ip = self.ip;
        let from_flags = self.flags;
        let from_registers = self.registers;
        let mut clock = 0;
        let mut clock_ea = 0;

        match (&inst.t, &inst.lhs, &inst.rhs) {
            (
                InstType::MOV,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                self.store_register(reg1, val);
                clock = 4;
            }
            (
                InstType::MOV,
                &Encoding::Memory(MemoryEncoding::Memory(address), size, _),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => self.store_memory(address, val, size),
            (
                InstType::MOV,
                &Encoding::Memory(MemoryEncoding::EffectiveAddress(ea), size, _),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => self.store_memory(self.translate_effective_address(ea), val, size),
            (
                InstType::MOV,
                &Encoding::Memory(MemoryEncoding::Memory(address), size, _),
                &Encoding::Operand(OperandEncoding::Register(reg1)),
            ) => self.store_memory(address, self.load_register(reg1), size),
            (
                InstType::MOV,
                &Encoding::Memory(MemoryEncoding::EffectiveAddress(ea), size, _),
                &Encoding::Operand(OperandEncoding::Register(reg1)),
            ) => self.store_memory(
                self.translate_effective_address(ea),
                self.load_register(reg1),
                size,
            ),
            (
                InstType::MOV,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => {
                self.store_register(reg1, self.load_register(reg2));
                clock = 2;
            }
            (
                InstType::MOV,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Memory(MemoryEncoding::Memory(address), size, _),
            ) => {
                self.store_register(reg1, self.load_memory(address, size));
                clock = 8;
                clock_ea = 6;
            }
            (
                InstType::MOV,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Memory(MemoryEncoding::EffectiveAddress(ea), size, _),
            ) => {
                self.store_register(
                    reg1,
                    self.load_memory(self.translate_effective_address(ea), size),
                );
                clock = 8;
                if ea.disp != 0 {
                    clock_ea += 6;
                }
                if ea.disp != 0 {
                    clock_ea += 6;
                }
            }
            (
                InstType::ADD,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => self.store_add_register(reg1, val),
            (
                InstType::ADD,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => self.store_add_register(reg1, self.load_register(reg2)),
            (
                InstType::ADD,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Memory(MemoryEncoding::EffectiveAddress(ea), size, _),
            ) => self.store_add_register(
                reg1,
                self.load_memory(self.translate_effective_address(ea), size),
            ),
            (
                InstType::ADD,
                &Encoding::Memory(MemoryEncoding::EffectiveAddress(ea), size, _),
                &Encoding::Operand(OperandEncoding::Register(reg1)),
            ) => self.store_add_memory(ea, size, reg1),
            (
                InstType::SUB,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => self.store_sub(reg1, self.load_register(reg2)),
            (
                InstType::SUB,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => self.store_sub(reg1, val),
            (
                InstType::CMP,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => {
                let from_reg = self.load_register(reg1);
                let to_reg = self.load_register(reg1) - self.load_register(reg2);
                self.update_flags(from_reg, to_reg);
                self.update_sub_flags(from_reg, self.load_register(reg2))
            }
            (
                InstType::CMP,
                &Encoding::Operand(OperandEncoding::Register(reg1)),
                &Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                let from_reg = self.load_register(reg1);
                let to_reg = self.load_register(reg1) - val;
                self.update_flags(from_reg, to_reg);
                self.update_sub_flags(from_reg, val)
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
            (
                InstType::LOOP,
                &Encoding::Operand(OperandEncoding::Jmp(offset, _)),
                Encoding::Empty,
            ) => {
                let new_cx = self.load_register(Register::CX) - 1;
                self.store_register(Register::CX, new_cx);
                if new_cx != 0 {
                    self.ip = (self.ip as i16 + offset as i16) as u16;
                }
            }
            _ => {
                unimplemented!("unimplemented instruction {:?}", inst);
            }
        };

        self.ip += inst.length as u16;

        let mut flag_update = None;
        if self.flags != from_flags {
            flag_update = Some((from_flags, self.flags));
        }

        let mut register_update = None;
        if self.registers != from_registers {
            register_update = [
                Register::AX,
                Register::BX,
                Register::CX,
                Register::DX,
                Register::SP,
                Register::BP,
                Register::SI,
                Register::DI,
            ]
            .into_iter()
            .find_map(|reg| {
                let to = self.load_register(reg);
                let from = from_registers.load(reg);
                if to != from {
                    Some((reg, from, to))
                } else {
                    None
                }
            });
        }

        Some(Step {
            inst,
            ip: (from_ip, self.ip),
            flags: flag_update,
            register: register_update,
            clock: Clock {
                value: clock,
                ea: clock_ea,
            },
        })
    }

    fn translate_effective_address(&self, ea: EffectiveAddress) -> u16 {
        let address = ea.disp
            + match ea.register {
                RegisterAddress::BXSI => {
                    self.load_register(Register::BX) + self.load_register(Register::SI)
                }
                RegisterAddress::BXDI => {
                    self.load_register(Register::BX) + self.load_register(Register::DI)
                }
                RegisterAddress::BPSI => {
                    self.load_register(Register::BP) + self.load_register(Register::SI)
                }
                RegisterAddress::BPDI => {
                    self.load_register(Register::BP) + self.load_register(Register::DI)
                }
                RegisterAddress::SI => self.load_register(Register::SI),
                RegisterAddress::DI => self.load_register(Register::DI),
                RegisterAddress::BX => self.load_register(Register::BX),
                RegisterAddress::DirectBP => self.load_register(Register::BP),
            };
        address as u16
    }

    fn load_register(&self, reg: Register) -> i16 {
        self.registers.load(reg)
    }

    fn store_register(&mut self, reg: Register, val: i16) {
        self.registers = self.registers.store(reg, val);
    }

    fn store_memory(&mut self, address: u16, val: i16, size: OperandSize) {
        self.memory[address as usize] = (val as u16 & 0xFF) as u8;
        if let OperandSize::Word = size {
            self.memory[address as usize + 1] = ((val as u16 >> 8) & 0xFF) as u8;
        };
    }

    fn load_memory(&self, address: u16, size: OperandSize) -> i16 {
        let mut val = self.memory[address as usize] as u16;
        if let OperandSize::Word = size {
            val |= (self.memory[address as usize + 1] as u16) << 8;
        };
        val as i16
    }

    fn store_add_register(&mut self, reg: Register, val: i16) {
        let from_reg = self.load_register(reg);
        self.store_register(reg, from_reg + val);
        let to_reg = self.load_register(reg);

        self.update_flags(from_reg, to_reg);
        self.update_add_flags(from_reg, val);
    }

    fn store_add_memory(&mut self, ea: EffectiveAddress, size: OperandSize, reg: Register) {
        let address = self.translate_effective_address(ea);
        let from = self.load_memory(address, size);
        let val = self.load_register(reg);
        let to = from + val;
        self.store_memory(address, to, size);

        self.update_flags(from, to);
        self.update_add_flags(from, val);
    }

    fn store_sub(&mut self, reg: Register, val: i16) {
        let from_reg = self.load_register(reg);
        self.store_register(reg, self.load_register(reg) - val);
        let to_reg = self.load_register(reg);

        self.update_flags(from_reg, to_reg);
        self.update_sub_flags(from_reg, val);
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

#[derive(Default, Clone)]
pub struct TracerOptions {
    pub with_ip: bool,
    pub with_print: bool,
    pub with_estimate: bool,
    pub dump_path: String,
}

#[derive(Default, Clone)]
pub struct Tracer {
    opt: TracerOptions,
    registers: HashSet<Register>,
    clocks: u32,
}

impl Tracer {
    pub fn with_options(opt: TracerOptions) -> Self {
        Self {
            opt,
            ..Default::default()
        }
    }

    pub fn run(&mut self, processor: &mut Processor) {
        if self.opt.with_print {
            while let Some(step) = processor.step() {
                self.trace(step);
            }
            self.print(processor);
        } else {
            processor.run();
        }

        if self.opt.dump_path != "" {
            self.dump(processor);
        }
    }

    fn trace(&mut self, step: Step) {
        use std::io::Write;
        let mut sink = std::io::stdout();
        let mut write_trace = |msg| write!(sink, "{}", msg).unwrap();
        let fmt_flags = |from, to| format!(" flags:{}->{}", from, to);
        let fmt_reg = |reg, from, to| format!(" {}:{:#x}->{:#x}", reg, from, to);
        let fmt_ip = |from, to| format!(" ip:{:#x}->{:#x}", from, to);
        let mut fmt_clock = |clock: Clock| {
            let inc = clock.value + clock.ea;
            self.clocks += inc as u32;
            let mut fmt = format!(" Clocks: +{} = {}", inc, self.clocks);
            if clock.ea != 0 {
                fmt = format!("{} ({} + {}ea)", fmt, clock.value, clock.ea);
            }
            format!("{} |", fmt)
        };

        write_trace(step.inst.to_string());
        write_trace(" ;".to_string());
        if self.opt.with_estimate {
            write_trace(fmt_clock(step.clock));
        }
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
        write_trace("\n".to_string());
    }

    fn print(&mut self, processor: &Processor) {
        let mut registers = self.registers.iter().map(|x| *x).collect::<Vec<Register>>();
        registers.sort();

        use std::io::Write;
        let mut sink = std::io::stdout();
        let mut write_trace = |msg| write!(sink, "{}", msg).unwrap();
        write_trace("Final registers:\n".to_string());
        for reg in registers {
            let val = processor.load_register(reg);
            write_trace(format!(
                "{:>8}: {:#06x} ({})\n",
                reg.to_string(),
                val as u16,
                val as u16,
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

        write_trace("\n".to_string());
    }

    fn dump(&mut self, processor: &Processor) {
        use std::io::Write;
        let mut sink =
            std::fs::File::create(self.opt.dump_path.clone()).expect("can't create file");
        sink.write_all(processor.memory.as_ref())
            .expect("can't dump");
    }
}
