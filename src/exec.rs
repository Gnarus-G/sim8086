use crate::{Instruction, Opcode, Operand, Register, Word};

pub struct Executor {
    pub registers: Registers,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
        }
    }
    pub fn execute(&mut self, instructions: &[Instruction]) {
        for i in instructions {
            match &i.opcode {
                Opcode::Mov(_) => self.execute_mov(i),
                Opcode::Add(_) => todo!(),
                Opcode::Sub(_) => todo!(),
                Opcode::Cmp(_) => todo!(),
                Opcode::J(_) => todo!(),
            }
        }
    }

    fn execute_mov(&mut self, i: &Instruction) {
        let source = i.source.as_ref().expect("movs to have a source operand");

        let value = match source {
            Operand::Immediate(imm) => imm,
            Operand::Register(_) => todo!(),
            Operand::MemoryAddress(_) => todo!(),
            Operand::ByteImmediate(_) => todo!(),
            Operand::WordImmediate(_) => todo!(),
            Operand::InstPtrIncrement(_) => todo!(),
        };

        match &i.destination {
            Operand::Register(reg) => {
                self.registers.set(reg, *value);
            }
            Operand::MemoryAddress(_) => todo!(),
            Operand::Immediate(_) => todo!(),
            Operand::ByteImmediate(_) => todo!(),
            Operand::WordImmediate(_) => todo!(),
            Operand::InstPtrIncrement(_) => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    ax: Word,
    bx: Word,
    cx: Word,
    dx: Word,
    sp: Word,
    bp: Word,
    si: Word,
    di: Word,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            ax: Word::default(),
            bx: Word::default(),
            cx: Word::default(),
            dx: Word::default(),
            sp: Word::default(),
            bp: Word::default(),
            si: Word::default(),
            di: Word::default(),
        }
    }

    pub fn set(&mut self, reg: &Register, value: u16) {
        match reg {
            Register::AL => self.ax.lo = value as u8,
            Register::BL => self.bx.lo = value as u8,
            Register::CL => self.cx.lo = value as u8,
            Register::DL => self.dx.lo = value as u8,
            Register::AH => self.ax.hi = value as u8,
            Register::BH => self.ax.hi = value as u8,
            Register::CH => self.ax.hi = value as u8,
            Register::DH => self.ax.hi = value as u8,
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
