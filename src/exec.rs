use std::fmt::Debug;

use crate::{
    decode::Decoder, jump::J, EffectiveAddressCalc, Instruction, Opcode, Operand, Register, Word,
};

pub struct Executor<'source> {
    pub memory: mem::Memory,
    decoder: Decoder<'source>,
    pub registers: Registers,
}

impl<'source> Executor<'source> {
    pub fn new(decoder: Decoder<'source>) -> Self {
        Self {
            memory: mem::Memory::new(),
            registers: Registers::new(),
            decoder,
        }
    }

    fn eval_operand(&mut self, operand: &Operand) -> Word {
        let value = match operand {
            Operand::Immediate(imm) => (*imm).into(),
            Operand::Register(reg) => *self.registers.get_reg(reg),
            Operand::MemoryAddress(eac) => {
                let addr = self.resolve_eac(eac);
                self.memory.load(addr)
            }
            Operand::ByteImmediate(imm) => Word::new(0, *imm),
            Operand::WordImmediate(imm) => (*imm).into(),
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
        let source_value: u16 = self.eval_operand(source).into();

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
            Operand::MemoryAddress(eac) => {
                let dest: u16 = self.eval_operand(&i.destination).into();
                let result = dest + source_value;
                let addr = self.resolve_eac(eac);
                self.memory.store(addr, result)
            }
            Operand::Immediate(_) => todo!(),
            Operand::ByteImmediate(_) => todo!(),
            Operand::WordImmediate(_) => todo!(),
            Operand::InstPtrIncrement(_) => todo!(),
        }
    }

    fn execute_sub(&mut self, i: &Instruction) {
        let source = i.source.as_ref().expect("sub to have a source operand");
        let source_value: u16 = self.eval_operand(source).into();

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
        let source_value: u16 = self.eval_operand(source).into();

        match &i.destination {
            Operand::Register(reg) => {
                let reg_value = self.registers.get_reg(reg);
                let dest: u16 = reg_value.into();
                let result = dest.wrapping_sub(source_value);
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

        let value = self.eval_operand(source);

        match &i.destination {
            Operand::Register(reg) => {
                self.registers.set(reg, value.into());
            }
            Operand::MemoryAddress(eac) => {
                let addr = self.resolve_eac(eac);
                self.memory.store(addr, value)
            }
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

    fn resolve_eac(&mut self, eac: &EffectiveAddressCalc) -> u16 {
        let addr = match eac {
            EffectiveAddressCalc::SingleReg(reg) => {
                let addr: u16 = self.registers.get_reg(reg).into();
                addr
            }
            EffectiveAddressCalc::SingleRegPlus(reg, disp) => {
                let addr_base: u16 = self.registers.get_reg(reg).into();

                (addr_base as i16 + disp) as u16
            }
            EffectiveAddressCalc::Plus(reg, reg1) => {
                let x: u16 = self.registers.get_reg(reg).into();
                let y: u16 = self.registers.get_reg(reg1).into();

                x + y
            }
            EffectiveAddressCalc::PlusConstant(reg, reg1, disp) => {
                let x: u16 = self.registers.get_reg(reg).into();
                let y: u16 = self.registers.get_reg(reg1).into();

                ((x + y) as i16 + *disp) as u16
            }
            EffectiveAddressCalc::DirectAddress(addr) => *addr,
        };

        addr
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
            Register::AL => self.ax.high = value as u8,
            Register::BL => self.bx.high = value as u8,
            Register::CL => self.cx.high = value as u8,
            Register::DL => self.dx.high = value as u8,
            Register::AH => self.ax.low = value as u8,
            Register::BH => self.bx.low = value as u8,
            Register::CH => self.cx.low = value as u8,
            Register::DH => self.dx.low = value as u8,
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

mod mem {
    use crate::Word;

    const MEMORY_SIZE: usize = 65536;

    pub struct Memory {
        buffer: [u8; MEMORY_SIZE],
    }

    impl Memory {
        pub fn new() -> Self {
            Self {
                buffer: [0; MEMORY_SIZE],
            }
        }

        pub fn store(&mut self, addr: u16, word: impl Into<Word>) {
            let addr = addr as usize;
            let word: Word = word.into();

            self.buffer[addr] = word.low;
            self.buffer[addr + 1] = word.high;
        }

        pub fn load(&mut self, addr: u16) -> Word {
            let addr = addr as usize;
            Word::new(self.buffer[addr + 1], self.buffer[addr])
        }

        pub fn dump(&self) -> Vec<u8> {
            self.buffer.to_vec()
        }
    }
}

pub mod clock_est {
    use crate::{mov, Instruction, Opcode, Operand, Register};

    #[derive(Debug)]
    pub struct ClockEstimate {
        pub base: usize,
        pub ea: Option<usize>,
    }

    impl ClockEstimate {
        pub fn value(&self) -> usize {
            self.base + self.ea.unwrap_or_default()
        }
    }

    impl From<Instruction> for ClockEstimate {
        fn from(value: Instruction) -> Self {
            use Operand as O;

            let mut ea = None;

            let base = match value.opcode {
                Opcode::Mov(m) => match m {
                    mov::Mov::RM => match (value.destination, value.source.unwrap()) {
                        (O::Register(_), O::Register(_)) => 2,
                        (O::Register(_), O::MemoryAddress(eac)) => {
                            ea = Some(ea_clock(&eac));
                            8
                        }
                        (O::MemoryAddress(eac), O::Register(_)) => {
                            ea = Some(ea_clock(&eac));
                            9
                        }
                        operands => todo!("{:?}", operands),
                    },
                    mov::Mov::ImmToReg => 4,
                    mov::Mov::ImmToRegOrMem => todo!(),
                    mov::Mov::MemToAcc => 10,
                    mov::Mov::AccToMem => 10,
                },
                Opcode::Add(a) => match a {
                    crate::add::Add::RM => match (value.destination, value.source.unwrap()) {
                        (O::Register(_), O::Register(_)) => 3,
                        (O::Register(_), O::MemoryAddress(eac)) => {
                            ea = Some(ea_clock(&eac));
                            9
                        }
                        (O::MemoryAddress(eac), O::Register(_)) => {
                            ea = Some(ea_clock(&eac));
                            16
                        }
                        operands => todo!("{:?}", operands),
                    },
                    crate::add::Add::ImmToRegOrMem => {
                        match (value.destination, value.source.unwrap()) {
                            (O::Register(_), O::WordImmediate(_)) => 4,
                            operands => todo!("{:?}", operands),
                        }
                    }
                    crate::add::Add::ImmToAcc => todo!(),
                },
                Opcode::Sub(_) => todo!(),
                Opcode::Cmp(_) => todo!(),
                Opcode::J(_) => todo!(),
            };

            Self { base, ea }
        }
    }

    fn ea_clock(eac: &crate::EffectiveAddressCalc) -> usize {
        match eac {
            crate::EffectiveAddressCalc::SingleReg(_) => 5,
            crate::EffectiveAddressCalc::SingleRegPlus(_, d) => {
                if *d == 0 {
                    5
                } else {
                    9
                }
            }
            crate::EffectiveAddressCalc::Plus(base, index) => {
                use Register as R;
                match (base, index) {
                    (R::BP, R::DI) | (R::BX, R::SI) => 7,
                    (R::BP, R::SI) | (R::BX, R::DI) => 8,
                    _ => unreachable!(),
                }
            }
            crate::EffectiveAddressCalc::PlusConstant(base, index, _) => {
                use Register as R;
                match (base, index) {
                    (R::BP, R::DI) | (R::BX, R::SI) => 11,
                    (R::BP, R::SI) | (R::BX, R::DI) => 12,
                    _ => unreachable!(),
                }
            }
            crate::EffectiveAddressCalc::DirectAddress(_) => 6,
        }
    }
}
