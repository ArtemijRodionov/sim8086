use crate::decoder::{Encoding, Inst, InstType, OperandEncoding, Register};
use std::collections::HashSet;
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
    bit_field_is!(is_p, 2);
    bit_field_set!(set_p, 2);
    bit_field_unset!(unset_p, 2);

    bit_field_is!(is_z, 6);
    bit_field_set!(set_z, 6);
    bit_field_unset!(unset_z, 6);

    bit_field_is!(is_s, 7);
    bit_field_set!(set_s, 7);
    bit_field_unset!(unset_s, 7);
}

impl std::fmt::Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.is_p() { "P" } else { "" },
            if self.is_z() { "Z" } else { "" },
            if self.is_s() { "S" } else { "" },
        )
    }
}

#[derive(Debug)]
struct Step {
    register: Option<(Register, i16, i16)>,
    flags: Option<(Flags, Flags)>,
}

#[derive(Debug, Default)]
pub struct Machine {
    registers: [i16; 16],
    flags: Flags,
    // stack: Vec<u8>,
    // memory: Vec<u8>,
}

impl Machine {
    fn get_register_value(&self, reg: Register) -> i16 {
        self.registers[reg.to_idx()]
    }
    fn get_flag(&self) -> Flags {
        return self.flags;
    }

    fn exec(&mut self, inst: Inst) -> Step {
        let mut register_update = None;
        let mut flag_update = None;

        match (inst.t, inst.lhs, inst.rhs) {
            (
                InstType::MOV,
                Encoding::Operand(OperandEncoding::Register(reg1)),
                Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                self.registers[reg1.to_idx()] = val;
                register_update = Some((reg1, from_reg, self.registers[reg1.to_idx()]));
            }
            (
                InstType::MOV,
                Encoding::Operand(OperandEncoding::Register(reg1)),
                Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                self.registers[reg1.to_idx()] = self.registers[reg2.to_idx()];
                register_update = Some((reg1, from_reg, self.registers[reg1.to_idx()]));
            }
            (
                InstType::ADD,
                Encoding::Operand(OperandEncoding::Register(reg1)),
                Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                let from_reg = self.registers[reg1.to_idx()];
                self.registers[reg1.to_idx()] += val;
                register_update = Some((reg1, from_reg, self.registers[reg1.to_idx()]));
            }
            (
                InstType::SUB,
                Encoding::Operand(OperandEncoding::Register(reg1)),
                Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => {
                let from_flags = self.flags;
                let from_reg = self.registers[reg1.to_idx()];

                self.registers[reg1.to_idx()] -= self.registers[reg2.to_idx()];
                self.update_flags(reg1);

                register_update = Some((reg1, from_reg, self.registers[reg1.to_idx()]));
                if self.flags != from_flags {
                    flag_update = Some((from_flags, self.flags));
                }
            }
            (
                InstType::SUB,
                Encoding::Operand(OperandEncoding::Register(reg1)),
                Encoding::Operand(OperandEncoding::Immediate(val)),
            ) => {
                let from_flags = self.flags;
                let from_reg = self.registers[reg1.to_idx()];

                self.registers[reg1.to_idx()] -= val;
                self.update_flags(reg1);

                register_update = Some((reg1, from_reg, self.registers[reg1.to_idx()]));
                if self.flags != from_flags {
                    flag_update = Some((from_flags, self.flags));
                }
            }
            (
                InstType::CMP,
                Encoding::Operand(OperandEncoding::Register(reg1)),
                Encoding::Operand(OperandEncoding::Register(reg2)),
            ) => {
                let from_flags = self.flags;
                if self.registers[reg1.to_idx()] - self.registers[reg2.to_idx()] < 0 {
                    self.flags = self.flags.set_s()
                } else {
                    self.flags = self.flags.unset_s()
                }

                if self.flags != from_flags {
                    flag_update = Some((from_flags, self.flags));
                }
            }
            _ => {}
        };

        Step {
            flags: flag_update,
            register: register_update,
        }
    }

    fn update_flags(&mut self, reg1: Register) {
        if self.registers[reg1.to_idx()] < 0 {
            self.flags = self.flags.set_s();
        } else {
            self.flags = self.flags.unset_s();
        }
        if self.registers[reg1.to_idx()] == 0 {
            self.flags = self.flags.set_z();
        } else {
            self.flags = self.flags.unset_z();
        }
        if (self.registers[reg1.to_idx()] & 0xFF).count_ones() % 2 == 0 {
            self.flags = self.flags.set_p();
        } else {
            self.flags = self.flags.unset_p();
        }
    }
}

#[derive(Default)]
pub struct Tracer {
    registers: HashSet<Register>,
}

impl Tracer {
    pub fn trace_exec(&mut self, m: &mut Machine, inst: Inst) {
        let mut trace = inst.to_string();
        let fmt_flags = |from, to| format!(" flags:{}->{}", from, to);
        let fmt_reg = |reg, from, to| format!(" {}:{:#x}->{:#x}", reg, from, to);

        let step = m.exec(inst);
        if step.register.is_some() || step.flags.is_some() {
            write!(trace, " ;").expect("can't write");
            if let Some((reg, from, to)) = step.register {
                self.registers.insert(reg);
                write!(trace, "{}", fmt_reg(reg, from, to)).expect("can't write");
            }
            if let Some((from, to)) = step.flags {
                write!(trace, "{}", fmt_flags(from, to)).expect("can't write");
            }
        }

        println!("{}", trace);
    }

    pub fn trace_state(&mut self, m: &Machine) {
        let mut registers = self.registers.iter().map(|x| *x).collect::<Vec<Register>>();
        registers.sort();

        let mut trace = "Final registers:\n".to_string();
        for reg in registers {
            write!(
                trace,
                "{:>8}: {:#06x} ({})\n",
                reg.to_string(),
                m.get_register_value(reg),
                m.get_register_value(reg),
            )
            .expect("write str error");
        }
        if m.flags != Flags(0) {
            write!(trace, "   flags: {}", m.flags).expect("write str error");
        }
        print!("{}", trace);
    }
}
