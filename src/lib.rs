use std::fmt::{format, Debug, Display};

use add::Add;
use cmp::Cmp;
use mov::Mov;
use sub::Sub;

use crate::jump::J;

pub mod decode;
pub mod exec;

pub struct Instruction {
    opcode: Opcode,
    source: Option<Operand>,
    destination: Operand,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) => write!(f, "{} {}, {}", self.opcode, self.destination, source),
            None => write!(f, "{} {}", self.opcode, self.destination),
        }
    }
}

pub enum Operand {
    Register(Register),
    MemoryAddress(EffectiveAddressCalc),
    Immediate(u16),
    ByteImmediate(u8),
    WordImmediate(u16),
    InstPtrIncrement(i8),
}

pub enum EffectiveAddressCalc {
    SingleReg(Register),
    SingleRegPlus(Register, i16),
    Plus(Register, Register),
    PlusConstant(Register, Register, i16),
    DirectAddress(u16),
}

impl EffectiveAddressCalc {
    fn with_no_disp<F: FnMut() -> u16>(rm: u8, mut da_value: F) -> Self {
        use Register as R;
        match rm {
            0 => Self::Plus(R::BX, R::SI),
            1 => Self::Plus(R::BX, R::DI),
            2 => Self::Plus(R::BP, R::SI),
            3 => Self::Plus(R::BP, R::DI),
            4 => Self::SingleReg(R::SI),
            5 => Self::SingleReg(R::DI),
            6 => Self::DirectAddress(da_value()),
            7 => Self::SingleReg(R::BX),
            _ => unreachable!(),
        }
    }

    fn with_disp(rm: u8, disp: i16) -> Self {
        use Register as R;
        match rm {
            0 => Self::PlusConstant(R::BX, R::SI, disp),
            1 => Self::PlusConstant(R::BX, R::DI, disp),
            2 => Self::PlusConstant(R::BP, R::SI, disp),
            3 => Self::PlusConstant(R::BP, R::DI, disp),
            4 => Self::SingleRegPlus(R::SI, disp),
            5 => Self::SingleRegPlus(R::DI, disp),
            6 => Self::SingleRegPlus(R::BP, disp),
            7 => Self::SingleRegPlus(R::BX, disp),
            _ => unreachable!(),
        }
    }
}

impl Display for EffectiveAddressCalc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = match self {
            EffectiveAddressCalc::SingleReg(r) => r.to_string(),
            EffectiveAddressCalc::SingleRegPlus(r, c) => {
                if c.signum() == -1 {
                    format!("{} - {}", r, c * -1)
                } else {
                    format!("{} + {}", r, c)
                }
            }
            EffectiveAddressCalc::Plus(ra, rb) => format!("{} + {}", ra, rb),
            EffectiveAddressCalc::PlusConstant(ra, rb, c) => {
                if c.signum() == -1 {
                    format!("{} + {} - {}", ra, rb, c * -1)
                } else {
                    format!("{} + {} + {}", ra, rb, c)
                }
            }
            EffectiveAddressCalc::DirectAddress(c) => c.to_string(),
        };

        write!(f, "[{}]", a)
    }
}
impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operand::Register(r) => r.to_string(),
                Operand::MemoryAddress(eac) => eac.to_string(),
                Operand::Immediate(value) => value.to_string(),
                Operand::ByteImmediate(b) => format!("byte {}", b),
                Operand::WordImmediate(w) => format!("word {}", w),
                Operand::InstPtrIncrement(p) => format!(
                    "${}",
                    if p.signum() >= 0 {
                        format!("+{}", p)
                    } else {
                        format!("{}", p)
                    }
                ),
            }
        )
    }
}

#[derive(Debug)]
pub enum Register {
    // low
    AL,
    BL,
    CL,
    DL,
    // high
    AH,
    BH,
    CH,
    DH,
    // wide
    AX,
    BX,
    CX,
    DX,
    // others
    SI,
    DI,
    SP,
    BP,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl Register {
    fn try_from(code: &u8, wide: &u8) -> Option<Self> {
        let r = match (code, wide) {
            (0, 0) => Register::AL,
            (0, 1) => Register::AX,
            (0b01, 0) => Register::CL,
            (0b01, 1) => Register::CX,
            (0b10, 0) => Register::DL,
            (0b10, 1) => Register::DX,
            (0b11, 0) => Register::BL,
            (0b11, 1) => Register::BX,
            (0b100, 0) => Register::AH,
            (0b100, 1) => Register::SP,
            (0b101, 0) => Register::CH,
            (0b101, 1) => Register::BP,
            (0b110, 0) => Register::DH,
            (0b110, 1) => Register::SI,
            (0b111, 0) => Register::BH,
            (0b111, 1) => Register::DI,
            _ => todo!(),
        };

        Some(r)
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Word {
    pub lo: u8,
    pub hi: u8,
}

impl Word {
    fn new(lo: u8, hi: u8) -> Self {
        Self { lo, hi }
    }
}

impl From<&mut Word> for u16 {
    fn from(value: &mut Word) -> Self {
        u16::from(*value)
    }
}

impl From<Word> for u16 {
    fn from(val: Word) -> Self {
        let high_bits = (val.hi as u16) << 8;
        let low_bits = val.lo as u16;
        high_bits | low_bits
    }
}

impl From<Word> for i16 {
    fn from(val: Word) -> Self {
        u16::from(val) as i16
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        let high = value >> 8;
        let low = (value << 8) >> 8;

        Self {
            lo: low as u8,
            hi: high as u8,
        }
    }
}

impl std::fmt::Binary for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lo({:08b}) hi({:08b})", self.lo, self.hi)
    }
}

impl Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lit = u16::from(*self);
        if f.alternate() {
            let lit = u16::from(*self);
            write!(f, "{:#06x} ({})", lit, lit)
        } else {
            write!(f, "{:#x}", lit)
        }
    }
}

#[derive(Debug)]
enum Opcode {
    Mov(mov::Mov),
    Add(add::Add),
    Sub(sub::Sub),
    Cmp(cmp::Cmp),
    J(jump::J),
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Opcode::Mov(_) => "mov",
            Opcode::Add(_) => "add",
            Opcode::Sub(_) => "sub",
            Opcode::Cmp(_) => "cmp",
            Opcode::J(j) => return write!(f, "{}", format!("{:?}", j).to_lowercase()),
        };
        write!(f, "{}", s)
    }
}

impl Opcode {
    fn try_from(word: &Word) -> Option<Opcode> {
        // println!("word {:b}", word);
        let first_four_bits = word.lo >> 4;
        let first_six_bits = word.lo >> 2;
        let first_seven_bits = word.lo >> 1;

        match first_six_bits {
            0b100010 => Some(Opcode::Mov(Mov::RM)),
            0b000000 => Some(Opcode::Add(Add::RM)),
            0b001010 => Some(Opcode::Sub(Sub::RM)),
            0b001110 => Some(Opcode::Cmp(Cmp::RM)),
            0b100000 => {
                let b = (word.hi & 0b00111000) >> 3;

                match b {
                    0b000 => Some(Opcode::Add(Add::ImmToRegOrMem)),
                    0b101 => Some(Opcode::Sub(Sub::ImmToRegOrMem)),
                    0b111 => Some(Opcode::Cmp(Cmp::ImmToRegOrMem)),
                    _ => todo!(),
                }
            }
            _ => match first_four_bits {
                0b1011 => Some(Opcode::Mov(Mov::ImmToReg)),
                _ => match first_seven_bits {
                    0b1100011 => Some(Opcode::Mov(Mov::ImmToRegOrMem)),
                    0b1010000 => Some(Opcode::Mov(Mov::MemToAcc)),
                    0b1010001 => Some(Opcode::Mov(Mov::AccToMem)),
                    0b0000010 => Some(Opcode::Add(Add::ImmToAcc)),
                    0b0010110 => Some(Opcode::Sub(Sub::ImmToAcc)),
                    0b0011110 => Some(Opcode::Cmp(Cmp::ImmToAcc)),
                    _ => match word.lo {
                        0b01110101 => Some(Opcode::J(J::Jne)),
                        0b01110100 => Some(Opcode::J(J::Je)),
                        0b01111100 => Some(Opcode::J(J::Jl)),
                        0b01111110 => Some(Opcode::J(J::Jle)),
                        0b01110010 => Some(Opcode::J(J::Jb)),
                        0b01110110 => Some(Opcode::J(J::Jbe)),
                        0b01111010 => Some(Opcode::J(J::Jp)),
                        0b01110000 => Some(Opcode::J(J::Jo)),
                        0b01111000 => Some(Opcode::J(J::Js)),
                        0b01111101 => Some(Opcode::J(J::Jnl)),
                        0b01111111 => Some(Opcode::J(J::Jg)),
                        0b01110011 => Some(Opcode::J(J::Jnb)),
                        0b01110111 => Some(Opcode::J(J::Ja)),
                        0b01111011 => Some(Opcode::J(J::Jnp)),
                        0b01110001 => Some(Opcode::J(J::Jno)),
                        0b01111001 => Some(Opcode::J(J::Jns)),
                        0b11100010 => Some(Opcode::J(J::Loop)),
                        0b11100001 => Some(Opcode::J(J::Loopz)),
                        0b11100000 => Some(Opcode::J(J::Loopnz)),
                        0b11100011 => Some(Opcode::J(J::Jcxz)),
                        _ => None,
                    },
                },
            },
        }
    }
}

mod mov {
    #[derive(Debug)]
    pub enum Mov {
        RM,
        ImmToReg,
        ImmToRegOrMem,
        MemToAcc,
        AccToMem,
    }
}

mod add {
    #[derive(Debug)]
    pub enum Add {
        RM,
        ImmToRegOrMem,
        ImmToAcc,
    }
}

mod sub {
    #[derive(Debug)]
    pub enum Sub {
        RM,
        ImmToRegOrMem,
        ImmToAcc,
    }
}

mod cmp {
    #[derive(Debug)]
    pub enum Cmp {
        RM,
        ImmToRegOrMem,
        ImmToAcc,
    }
}

mod jump {

    #[derive(Debug)]
    pub enum J {
        Je,
        Jl,
        Jle,
        Jb,
        Jbe,
        Jp,
        Jo,
        Js,
        Jne, // Jnz
        Jnl,
        Jg,
        Jnb,
        Ja,
        Jnp,
        Jno,
        Jns,
        Loop,
        Loopz,
        Loopnz,
        Jcxz,
    }
}
