use std::fmt::{Debug, Display};

use crate::{add::Add, mov::Mov, sub::Sub, Opcode, Word};

struct Scanner<'source> {
    input: &'source [u8],
    offset: usize,
    read_offset: usize,
    instructions: Vec<Instruction>,
}

impl<'source> Scanner<'source> {
    pub fn new(input: &'source [u8]) -> Self {
        Self {
            input,
            offset: 0,
            read_offset: 0,
            instructions: vec![],
        }
    }

    fn curr_word(&self) -> Option<Word> {
        let Some([a, b]) = self.input.get(self.offset..self.offset + 2) else {
            return None;
        };
        Some(Word { lo: *a, hi: *b })
    }

    fn next_byte(&mut self) -> Option<u8> {
        let Some(a) = self.input.get(self.read_offset) else {
            return None;
        };
        self.offset = self.read_offset;
        self.read_offset += 1;
        Some(*a)
    }

    fn next_word(&mut self) -> Option<Word> {
        self.offset = self.read_offset;
        self.read_offset += 2;

        self.curr_word()
    }

    fn scan(&mut self) {
        while let Some(word) = self.next_word() {
            let opcode = Opcode::try_from(&word).unwrap();
            let i = match &opcode {
                Opcode::Mov(m) => match m {
                    Mov::ImmToReg => self.scan_mov_immediate_to_register(opcode),
                    Mov::RM => self.scan_register_memory_to_from_either(opcode),
                    Mov::ImmToRegOrMem => self.scan_mov_immediate_to_reg_or_memory(opcode),
                    Mov::MemToAcc => self.scan_mov_mem_to_acc(opcode),
                    Mov::AccToMem => self.scan_mov_acc_to_mem(opcode),
                },
                Opcode::Add(a) => match a {
                    Add::RM => self.scan_register_memory_to_from_either(opcode),
                    Add::ImmToRegOrMem => {
                        self.scan_immediate_to_reg_or_memory_with_sign_extension(opcode)
                    }
                    Add::ImmToAcc => self.scan_immediate_to_acc(opcode),
                },
                Opcode::Sub(s) => match s {
                    Sub::RM => self.scan_register_memory_to_from_either(opcode),
                    Sub::ImmToRegOrMem => {
                        self.scan_immediate_to_reg_or_memory_with_sign_extension(opcode)
                    }
                    Sub::ImmToAcc => self.scan_immediate_to_acc(opcode),
                },
            };

            self.instructions.push(i);
        }
    }

    fn scan_register_memory_to_from_either(&mut self, opcode: Opcode) -> Instruction {
        let word = self.curr_word().unwrap();

        let destination;
        let source;

        // D
        let d_mask = 0x02;
        let reg_is_destination = (d_mask & word.lo) == d_mask;

        // W
        let w_mask = 1;
        let wide = w_mask & word.lo;

        // MOD
        let mode = (word.hi & 0b11000000) >> 6;

        // REG
        let reg_code = (word.hi & 0b00111000) >> 3;

        // R/M
        let rm = word.hi & 0x07;

        let mut get_other_operand = || match mode {
            0b00 => {
                let eac =
                    EffectiveAddressCalc::with_no_disp(rm, || self.next_word().unwrap().into());
                Operand::MemoryAddress(eac)
            }
            0b01 => {
                let eac = EffectiveAddressCalc::with_disp(
                    rm,
                    self.next_byte().unwrap() /* should sign extends so...*/ as i8 as i16,
                );
                Operand::MemoryAddress(eac)
            }
            0b10 => {
                let eac = EffectiveAddressCalc::with_disp(rm, self.next_word().unwrap().into());
                Operand::MemoryAddress(eac)
            }
            0b11 => {
                let rm_reg_code = word.hi & 0b00000111;
                Operand::Register(Register::try_from(&rm_reg_code, &wide).unwrap())
            }
            _ => unreachable!(),
        };

        if reg_is_destination {
            destination = Operand::Register(Register::try_from(&reg_code, &wide).unwrap());
            source = get_other_operand();
        } else {
            source = Operand::Register(Register::try_from(&reg_code, &wide).unwrap());
            destination = get_other_operand();
        }

        Instruction {
            opcode,
            source,
            destination,
        }
    }

    fn scan_mov_immediate_to_register(&mut self, opcode: Opcode) -> Instruction {
        let word = self.curr_word().unwrap();

        // W
        let wide = (0b00001000 & word.lo) >> 3;

        // REG
        let reg_code = 0b00000111 & word.lo;

        let source = if wide == 0 {
            Operand::Immediate(word.hi as u16)
        } else {
            let next_byte = self.next_byte().expect("a byte after current word");
            let next_word = Word::new(word.hi, next_byte);
            let value = next_word.into();
            Operand::Immediate(value)
        };

        Instruction {
            opcode,
            source,
            destination: Operand::Register(Register::try_from(&reg_code, &wide).unwrap()),
        }
    }

    fn scan_mov_immediate_to_reg_or_memory(&mut self, opcode: Opcode) -> Instruction {
        let word = self.curr_word().unwrap();

        // W
        let w_mask = 1;
        let wide = w_mask & word.lo;

        // MOD
        let mode = (word.hi & 0b11000000) >> 6;

        // R/M
        let rm = word.hi & 0x07;

        let mut get_destination_operand = || match mode {
            0b00 => {
                let eac =
                    EffectiveAddressCalc::with_no_disp(rm, || self.next_byte().unwrap().into());
                Operand::MemoryAddress(eac)
            }
            0b01 => {
                let eac = EffectiveAddressCalc::with_disp(
                    rm,
                    self.next_byte().unwrap() /* should sign extends so...*/ as i8 as i16,
                );
                Operand::MemoryAddress(eac)
            }
            0b10 => {
                let eac = EffectiveAddressCalc::with_disp(rm, self.next_word().unwrap().into());
                Operand::MemoryAddress(eac)
            }
            0b11 => {
                let rm_reg_code = word.hi & 0b00000111;
                Operand::Register(Register::try_from(&rm_reg_code, &wide).unwrap())
            }
            _ => unreachable!(),
        };

        let destination = get_destination_operand();

        let source = if wide == 1 {
            let data = self.next_word().unwrap();
            Operand::WordImmediate(data.into())
        } else {
            let data = self.next_byte().unwrap();
            Operand::ByteImmediate(data)
        };

        Instruction {
            opcode,
            source,
            destination,
        }
    }

    fn scan_mov_mem_to_acc(&mut self, opcode: Opcode) -> Instruction {
        let word = self.curr_word().unwrap();

        // W
        let w_mask = 1;
        let wide = w_mask & word.lo;

        let addr: u16 = if wide == 1 {
            Word::new(word.hi, self.next_byte().unwrap()).into()
        } else {
            word.hi as u16
        };

        Instruction {
            opcode,
            source: Operand::MemoryAddress(EffectiveAddressCalc::DirectAddress(addr)),
            destination: Operand::Register(Register::AX),
        }
    }

    fn scan_mov_acc_to_mem(&mut self, opcode: Opcode) -> Instruction {
        let word = self.curr_word().unwrap();

        // W
        let w_mask = 1;
        let wide = w_mask & word.lo;

        let addr: u16 = if wide == 1 {
            Word::new(word.hi, self.next_byte().unwrap()).into()
        } else {
            word.hi as u16
        };

        Instruction {
            opcode,
            source: Operand::Register(Register::AX),
            destination: Operand::MemoryAddress(EffectiveAddressCalc::DirectAddress(addr)),
        }
    }

    fn scan_immediate_to_acc(&mut self, opcode: Opcode) -> Instruction {
        let word = self.curr_word().unwrap();

        // W
        let w_mask = 1;
        let wide = w_mask & word.lo;

        let imm: u16 = if wide == 1 {
            Word::new(word.hi, self.next_byte().unwrap()).into()
        } else {
            word.hi as u16
        };

        Instruction {
            opcode,
            source: Operand::Immediate(imm),
            destination: Operand::Register(Register::AX),
        }
    }

    fn scan_immediate_to_reg_or_memory_with_sign_extension(
        &mut self,
        opcode: Opcode,
    ) -> Instruction {
        let word = self.curr_word().unwrap();

        // S
        let sign_extend = (word.lo & 0b10) >> 1;

        // W
        let w_mask = 1;
        let wide = w_mask & word.lo;

        // MOD
        let mode = (word.hi & 0b11000000) >> 6;

        // R/M
        let rm = word.hi & 0x07;

        let mut get_destination_operand = || match mode {
            0b00 => {
                let eac =
                    EffectiveAddressCalc::with_no_disp(rm, || self.next_byte().unwrap().into());
                Operand::MemoryAddress(eac)
            }
            0b01 => {
                let eac = EffectiveAddressCalc::with_disp(
                    rm,
                    self.next_byte().unwrap() /* should sign extends so...*/ as i8 as i16,
                );
                Operand::MemoryAddress(eac)
            }
            0b10 => {
                let eac = EffectiveAddressCalc::with_disp(rm, self.next_word().unwrap().into());
                Operand::MemoryAddress(eac)
            }
            0b11 => {
                let rm_reg_code = word.hi & 0b00000111;
                Operand::Register(Register::try_from(&rm_reg_code, &wide).unwrap())
            }
            _ => unreachable!(),
        };

        let destination = get_destination_operand();

        let source = match (sign_extend, wide) {
            (0, 1) => {
                let data = self.next_word().unwrap();
                Operand::Immediate(data.into())
            }
            (1, 1) => {
                let data = self.next_byte().unwrap();
                Operand::Immediate(data as i8 as i16 as u16) // casts are for sign extending
            }
            _ => {
                let data = self.next_byte().unwrap();
                Operand::Immediate(data as u16)
            }
        };

        Instruction {
            opcode,
            source,
            destination,
        }
    }
}

struct Instruction {
    opcode: Opcode,
    source: Operand,
    destination: Operand,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}, {}", self.opcode, self.destination, self.source)
    }
}

enum Operand {
    Register(Register),
    MemoryAddress(EffectiveAddressCalc),
    Immediate(u16),
    ByteImmediate(u8),
    WordImmediate(u16),
}

enum EffectiveAddressCalc {
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

pub fn decode(buffer: &[u8]) -> Vec<String> {
    let mut scanner = Scanner::new(buffer);

    scanner.scan();

    scanner.instructions.iter().map(|i| i.to_string()).collect()
}
