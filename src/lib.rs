use std::fmt::{Debug, Display};

use add::Add;
use cmp::Cmp;
use mov::Mov;
use sub::Sub;

use crate::jump::J;

pub mod decode;

#[derive(Clone, Copy)]
struct Word {
    lo: u8,
    hi: u8,
}

impl Word {
    fn new(lo: u8, hi: u8) -> Self {
        Self { lo, hi }
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

impl Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lo({:08b}) hi({:08b})", self.lo, self.hi)
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
        // println!("word {:?}", word);
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
