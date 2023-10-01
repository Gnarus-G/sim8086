use std::fmt::{Debug, Display};

struct Scanner<'source> {
    input: &'source [u8],
    offset: usize,
    instructions: Vec<Instruction>,
}

struct Word<'w> {
    high: &'w u8,
    low: &'w u8,
}

impl<'w> Debug for Word<'w> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:b} {:b}", self.high, self.low)
    }
}

impl<'source> Scanner<'source> {
    pub fn new(input: &'source [u8]) -> Self {
        Self {
            input,
            offset: 0,
            instructions: vec![],
        }
    }

    fn next_word(&mut self) -> Option<Word> {
        let Some([a, b]) = self.input.get(self.offset..self.offset + 2) else {
            return None;
        };
        self.offset += 2;
        Some(Word { high: a, low: b })
    }

    fn scan(&mut self) {
        while let Some(word) = self.next_word() {
            let opcode = get_opcode(word.high).unwrap();

            let destination;
            let source;

            // D
            let d_mask = 0x10;
            let reg_is_destination = (d_mask & word.high) == 1;

            // W
            let w_mask = 0x01;
            let wide = w_mask & word.high;

            // MOD
            let mode = (word.low & 0b11000000) >> 6;

            // REG
            let reg_code = (word.low & 0b00111000) >> 3;

            if reg_is_destination {
                destination = Operand::Register(Register::try_from(&reg_code, &wide).unwrap());

                match mode {
                    0b00 => todo!(),
                    0b01 => todo!(),
                    0b10 => todo!(),
                    0b11 => {
                        let rm_reg_code = word.low & 0b00000111;
                        source = Operand::Register(Register::try_from(&rm_reg_code, &wide).unwrap())
                    }
                    _ => unreachable!(),
                }
            } else {
                source = Operand::Register(Register::try_from(&reg_code, &wide).unwrap());

                match mode {
                    0b00 => todo!(),
                    0b01 => todo!(),
                    0b10 => todo!(),
                    0b11 => {
                        let rm_reg_code = word.low & 0b00000111;
                        destination =
                            Operand::Register(Register::try_from(&rm_reg_code, &wide).unwrap())
                    }
                    _ => unreachable!(),
                }
            }

            self.instructions.push(Instruction {
                opcode,
                wide: wide == 1,
                source,
                destination,
            })
        }
    }
}

struct Instruction {
    opcode: Opcode,
    wide: bool,
    source: Operand,
    destination: Operand,
    // mode: u8,
    // rm: u8,
    // displacement: Option<Displacement>,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}, {}", self.opcode, self.destination, self.source)
    }
}

enum Operand {
    Register(Register),
    MemoryAddress,
    Immediate(u8),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operand::Register(r) => r.to_string(),
                Operand::MemoryAddress => todo!(),
                Operand::Immediate(value) => value.to_string(),
            }
        )
    }
}

impl Instruction {}

#[derive(Debug)]
enum Register {
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

enum Dest {
    RegSource,
    RegDestination,
}

enum Displacement {
    Low(u8),
    High(u8),
}

#[derive(Debug, PartialEq)]
enum Opcode {
    Mov = 0b100010,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

fn get_opcode(byte: &u8) -> Option<Opcode> {
    let first_six_bits = byte >> 2;
    match first_six_bits {
        0b100010 => Some(Opcode::Mov),
        _ => None,
    }
}

pub fn decode(buffer: &[u8]) -> Vec<String> {
    let mut scanner = Scanner::new(buffer);

    scanner.scan();

    scanner.instructions.iter().map(|i| i.to_string()).collect()
}
