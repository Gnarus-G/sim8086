use std::fmt::{Debug, Display};

use add::Add;
use mov::Mov;
use sub::Sub;

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
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Opcode::Mov(_) => "mov",
            Opcode::Add(_) => "add",
            Opcode::Sub(_) => "sub",
        };
        write!(f, "{}", s)
    }
}

impl Opcode {
    fn try_from(word: &Word) -> Option<Opcode> {
        // println!("word {:?}", word);
        let byte = word.lo;
        let first_four_bits = byte >> 4;
        let first_six_bits = byte >> 2;
        let first_seven_bits = byte >> 1;

        match first_six_bits {
            0b100010 => Some(Opcode::Mov(Mov::RM)),
            0b000000 => Some(Opcode::Add(Add::RM)),
            0b001010 => Some(Opcode::Sub(Sub::RM)),
            0b100000 => {
                let b = (word.hi & 0b00111000) >> 3;

                match b {
                    0b000 => Some(Opcode::Add(Add::ImmToRegOrMem)),
                    0b101 => Some(Opcode::Sub(Sub::ImmToRegOrMem)),
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
                    _ => None,
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
