use crate::{
    add::Add, cmp::Cmp, mov::Mov, sub::Sub, EffectiveAddressCalc, Instruction, Opcode, Operand,
    Register, Word,
};

pub struct Decoder<'source> {
    input: &'source [u8],
    offset: usize,
    pub(crate) read_offset: usize,
}

impl<'source> Decoder<'source> {
    pub fn new(input: &'source [u8]) -> Self {
        Self {
            input,
            offset: 0,
            read_offset: 0,
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

    pub fn decode_next(&mut self) -> Option<Instruction> {
        if let Some(word) = self.next_word() {
            let opcode = Opcode::try_from(&word).unwrap();
            let i = match &opcode {
                Opcode::Mov(m) => match m {
                    Mov::ImmToReg => self.decode_mov_immediate_to_register(opcode),
                    Mov::RM => self.decode_register_memory_to_from_either(opcode),
                    Mov::ImmToRegOrMem => self.decode_mov_immediate_to_reg_or_memory(opcode),
                    Mov::MemToAcc => self.decode_mov_mem_to_acc(opcode),
                    Mov::AccToMem => self.decode_mov_acc_to_mem(opcode),
                },
                Opcode::Add(a) => match a {
                    Add::RM => self.decode_register_memory_to_from_either(opcode),
                    Add::ImmToRegOrMem => {
                        self.decode_immediate_to_reg_or_memory_with_sign_extension(opcode)
                    }
                    Add::ImmToAcc => self.decode_immediate_to_acc(opcode),
                },
                Opcode::Sub(s) => match s {
                    Sub::RM => self.decode_register_memory_to_from_either(opcode),
                    Sub::ImmToRegOrMem => {
                        self.decode_immediate_to_reg_or_memory_with_sign_extension(opcode)
                    }
                    Sub::ImmToAcc => self.decode_immediate_to_acc(opcode),
                },
                Opcode::Cmp(c) => match c {
                    Cmp::RM => self.decode_register_memory_to_from_either(opcode),
                    Cmp::ImmToRegOrMem => {
                        self.decode_immediate_to_reg_or_memory_with_sign_extension(opcode)
                    }
                    Cmp::ImmToAcc => self.decode_immediate_to_acc(opcode),
                },
                Opcode::J(_) => self.decode_jump(opcode),
            };

            return Some(i);
        }

        None
    }

    fn decode_register_memory_to_from_either(&mut self, opcode: Opcode) -> Instruction {
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
            source: Some(source),
            destination,
        }
    }

    fn decode_mov_immediate_to_register(&mut self, opcode: Opcode) -> Instruction {
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
            source: Some(source),
            destination: Operand::Register(Register::try_from(&reg_code, &wide).unwrap()),
        }
    }

    fn decode_mov_immediate_to_reg_or_memory(&mut self, opcode: Opcode) -> Instruction {
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
            source: Some(source),
            destination,
        }
    }

    fn decode_mov_mem_to_acc(&mut self, opcode: Opcode) -> Instruction {
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
            source: Some(Operand::MemoryAddress(EffectiveAddressCalc::DirectAddress(
                addr,
            ))),
            destination: Operand::Register(Register::AX),
        }
    }

    fn decode_mov_acc_to_mem(&mut self, opcode: Opcode) -> Instruction {
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
            source: Some(Operand::Register(Register::AX)),
            destination: Operand::MemoryAddress(EffectiveAddressCalc::DirectAddress(addr)),
        }
    }

    fn decode_immediate_to_acc(&mut self, opcode: Opcode) -> Instruction {
        let word = self.curr_word().unwrap();

        // W
        let w_mask = 1;
        let wide = w_mask & word.lo;

        let (imm, reg) = if wide == 1 {
            let imm = Word::new(word.hi, self.next_byte().unwrap()).into();
            (imm, Register::AX)
        } else {
            let imm = word.hi as u16;
            (imm, Register::AL)
        };

        Instruction {
            opcode,
            source: Some(Operand::Immediate(imm)),
            destination: Operand::Register(reg),
        }
    }

    fn decode_immediate_to_reg_or_memory_with_sign_extension(
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

        let destination = get_destination_operand();

        let source = match (sign_extend, wide) {
            (0, 1) => {
                let data = self.next_word().unwrap();
                Operand::WordImmediate(data.into())
            }
            (1, 1) => {
                let data = self.next_byte().unwrap();
                Operand::WordImmediate(data as i8 as i16 as u16) // casts are for sign extending
            }
            _ => {
                let data = self.next_byte().unwrap();
                Operand::ByteImmediate(data)
            }
        };

        Instruction {
            opcode,
            source: Some(source),
            destination,
        }
    }

    fn decode_jump(&mut self, opcode: Opcode) -> Instruction {
        let word = self.curr_word().unwrap();
        let inc = word.hi as i8;

        Instruction {
            opcode,
            source: None,
            destination: Operand::InstPtrIncrement(inc),
        }
    }
}
