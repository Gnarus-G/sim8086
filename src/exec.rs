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
