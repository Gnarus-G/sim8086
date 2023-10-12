use std::fmt::Debug;

use crate::{decode::Decoder, jump::J, Instruction, Opcode, Operand, Register, Word};

pub struct Executor<'source> {
    decoder: Decoder<'source>,
    pub registers: Registers,
}

impl<'source> Executor<'source> {
    pub fn new(decoder: Decoder<'source>) -> Self {
        Self {
            registers: Registers::new(),
            decoder,
        }
    }

    fn eval_operand(&mut self, operand: &Operand) -> u16 {
        let value = match operand {
            Operand::Immediate(imm) => *imm,
            Operand::Register(reg) => self.registers.get_reg(reg).into(),
            Operand::MemoryAddress(_) => todo!(),
            Operand::ByteImmediate(_) => todo!(),
            Operand::WordImmediate(imm) => *imm,
            Operand::InstPtrIncrement(_) => todo!(),
        };

        value
    }

    pub fn execute_next(&mut self) -> Option<(Instruction, RegistersDiff)> {
        self.decoder.decode_next().map(|i| {
            let before = self.registers;
            self.registers.ip = (self.decoder.read_offset as u16).into();
            match &i.opcode {
                Opcode::Mov(_) => self.execute_mov(&i),
                Opcode::Add(_) => self.execute_add(&i),
                Opcode::Sub(_) => self.execute_sub(&i),
                Opcode::Cmp(_) => self.execute_cmp(&i),
                Opcode::J(_) => self.execute_jump(&i),
            };
            (i, RegistersDiff(before, self.registers))
        })
    }

    fn execute_add(&mut self, i: &Instruction) {
        let source = i.source.as_ref().expect("add to have a source operand");
        let source_value = self.eval_operand(source);

        match &i.destination {
            Operand::Register(reg) => {
                let reg_value = self.registers.get_reg(reg);
                let dest: u16 = reg_value.into();
                let result = dest + source_value;
                self.registers.set(reg, result);
                let msb_is_1 = (0x8000 & result) != 0;

                self.registers.flags.sign = msb_is_1;
                self.registers.flags.zero = result == 0;
            }
            Operand::MemoryAddress(_) => todo!(),
            Operand::Immediate(_) => todo!(),
            Operand::ByteImmediate(_) => todo!(),
            Operand::WordImmediate(_) => todo!(),
            Operand::InstPtrIncrement(_) => todo!(),
        }
    }

    fn execute_sub(&mut self, i: &Instruction) {
        let source = i.source.as_ref().expect("sub to have a source operand");
        let source_value = self.eval_operand(source);

        match &i.destination {
            Operand::Register(reg) => {
                let reg_value = self.registers.get_reg(reg);
                let dest: u16 = reg_value.into();
                let result = dest.wrapping_sub(source_value);
                self.registers.set(reg, result);
                let msb_is_1 = (0x8000 & result) != 0;

                self.registers.flags.sign = msb_is_1;
                self.registers.flags.zero = result == 0;
            }
            Operand::MemoryAddress(_) => todo!(),
            Operand::Immediate(_) => todo!(),
            Operand::ByteImmediate(_) => todo!(),
            Operand::WordImmediate(_) => todo!(),
            Operand::InstPtrIncrement(_) => todo!(),
        }
    }

    fn execute_cmp(&mut self, i: &Instruction) {
        let source = i.source.as_ref().expect("sub to have a source operand");
        let source_value = self.eval_operand(source);

        match &i.destination {
            Operand::Register(reg) => {
                let reg_value = self.registers.get_reg(reg);
                let dest: u16 = reg_value.into();
                let result = dest - source_value;
                let msb_is_1 = (0x8000 & result) != 0;

                self.registers.flags.sign = msb_is_1;
                self.registers.flags.zero = result == 0;
            }
            Operand::MemoryAddress(_) => todo!(),
            Operand::Immediate(_) => todo!(),
            Operand::ByteImmediate(_) => todo!(),
            Operand::WordImmediate(_) => todo!(),
            Operand::InstPtrIncrement(_) => todo!(),
        }
    }

    fn execute_mov(&mut self, i: &Instruction) {
        let source = i.source.as_ref().expect("movs to have a source operand");

        let value = match source {
            Operand::Immediate(imm) => *imm,
            Operand::Register(reg) => self.registers.get_reg(reg).into(),
            Operand::MemoryAddress(_) => todo!(),
            Operand::ByteImmediate(_) => todo!(),
            Operand::WordImmediate(_) => todo!(),
            Operand::InstPtrIncrement(_) => todo!(),
        };

        match &i.destination {
            Operand::Register(reg) => {
                self.registers.set(reg, value);
            }
            Operand::MemoryAddress(_) => todo!(),
            Operand::Immediate(_) => todo!(),
            Operand::ByteImmediate(_) => todo!(),
            Operand::WordImmediate(_) => todo!(),
            Operand::InstPtrIncrement(_) => todo!(),
        }
    }

    fn execute_jump(&mut self, i: &Instruction) {
        let inc = match &i.destination {
            Operand::InstPtrIncrement(inc) => *inc,
            _ => unreachable!(),
        };

        let new_offset = (u16::from(self.registers.ip) as i16 + inc as i16) as u16;

        match &i.opcode {
            Opcode::J(J::Jne) => {
                if !self.registers.flags.zero {
                    self.decoder.read_offset = new_offset as usize;
                    self.registers.ip = Word::from(new_offset);
                }
            }
            _ => todo!(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Registers {
    ax: Word,
    bx: Word,
    cx: Word,
    dx: Word,
    sp: Word,
    bp: Word,
    si: Word,
    di: Word,
    flags: Flags,
    ip: Word,
}

impl Registers {
    pub fn new() -> Registers {
        Registers::default()
    }

    pub fn get_reg(&mut self, reg: &Register) -> &mut Word {
        let r: &mut _ = match reg {
            Register::AL => &mut self.ax,
            Register::BL => &mut self.bx,
            Register::CL => &mut self.cx,
            Register::DL => &mut self.dx,
            Register::AH => &mut self.ax,
            Register::BH => &mut self.bx,
            Register::CH => &mut self.cx,
            Register::DH => &mut self.dx,
            Register::AX => &mut self.ax,
            Register::BX => &mut self.bx,
            Register::CX => &mut self.cx,
            Register::DX => &mut self.dx,
            Register::SI => &mut self.si,
            Register::DI => &mut self.di,
            Register::SP => &mut self.sp,
            Register::BP => &mut self.bp,
        };

        r
    }

    pub fn set(&mut self, reg: &Register, value: u16) {
        match reg {
            Register::AL => self.ax.lo = value as u8,
            Register::BL => self.bx.lo = value as u8,
            Register::CL => self.cx.lo = value as u8,
            Register::DL => self.dx.lo = value as u8,
            Register::AH => self.ax.hi = value as u8,
            Register::BH => self.bx.hi = value as u8,
            Register::CH => self.cx.hi = value as u8,
            Register::DH => self.dx.hi = value as u8,
            Register::AX => self.ax = value.into(),
            Register::BX => self.bx = value.into(),
            Register::CX => self.cx = value.into(),
            Register::DX => self.dx = value.into(),
            Register::SI => self.si = value.into(),
            Register::DI => self.di = value.into(),
            Register::SP => self.sp = value.into(),
            Register::BP => self.bp = value.into(),
        }
    }
}

impl Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! disp {
            ($reg:ident) => {
                disp!($reg, "\n")
            };
            ($reg:ident, $delim:expr) => {
                write!(f, "{:>5}: {:#?}{}", stringify!($reg), self.$reg, $delim)?
            };
        }

        disp!(ax);
        disp!(bx);
        disp!(cx);
        disp!(dx);
        disp!(sp);
        disp!(bp);
        disp!(si);
        disp!(di);
        disp!(ip);
        disp!(flags, "");

        Ok(())
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
struct Flags {
    sign: bool,
    zero: bool,
}

impl Debug for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display = |string, is_true| -> Result<(), std::fmt::Error> {
            if is_true {
                f.write_str(string)?;
            }
            Ok(())
        };

        display("S", self.sign)?;
        display("Z", self.zero)
    }
}

pub struct RegistersDiff(Registers, Registers);

impl Debug for RegistersDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! disp {
            ($reg:ident) => {
                disp!($reg, ", ")
            };
            ($reg:ident, $delim:expr) => {
                if self.0.$reg != self.1.$reg {
                    write!(
                        f,
                        "{}:{:?}->{:?}{}",
                        stringify!($reg),
                        self.0.$reg,
                        self.1.$reg,
                        $delim
                    )?
                }
            };
        }

        disp!(ax);
        disp!(bx);
        disp!(cx);
        disp!(dx);
        disp!(sp);
        disp!(bp);
        disp!(si);
        disp!(di);
        disp!(ip);
        disp!(flags, "");

        Ok(())
    }
}
