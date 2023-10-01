#[derive(Debug, PartialEq)]
enum Opcode {
    Mov = 0b100010,
}

fn get_opcode(byte: &u8) -> Option<Opcode> {
    let first_six_bits = byte >> 2;
    match first_six_bits {
        0b100010 => Some(Opcode::Mov),
        _ => None,
    }
}

fn get_reg_name((reg, w_set): (u8, u8)) -> &'static str {
    match (reg, w_set) {
        (0, 0) => "al",
        (0, 1) => "ax",
        (0b01, 0) => "cl",
        (0b01, 1) => "cx",
        (0b10, 0) => "dl",
        (0b10, 1) => "dx",
        (0b11, 0) => "bl",
        (0b11, 1) => "bx",
        (0b100, 0) => "ah",
        (0b100, 1) => "sp",
        (0b101, 0) => "ch",
        (0b101, 1) => "bp",
        (0b110, 0) => "dh",
        (0b110, 1) => "si",
        (0b111, 0) => "bh",
        (0b111, 1) => "di",
        _ => todo!(),
    }
}

pub fn decode(buffer: &[u8]) -> Vec<String> {
    let mut res = vec![];

    buffer.windows(2).for_each(|a| {
        if let [h, l] = a {
            if let Some(text) = decode_instruction(h, l) {
                res.push(text);
            }
        }
    });

    res
}

fn decode_instruction(high: &u8, low: &u8) -> Option<String> {
    if let Some(Opcode::Mov) = get_opcode(high) {
        let d_mask = 0x10;
        let w_mask = 0x01;

        let w_set = w_mask & high;

        let reg_is_destination = (d_mask & high) == 1;
        let reg = (low & 0b00111000) >> 3;
        let rm_reg = low & 0b00000111;

        let reg_name = get_reg_name((reg, w_set));

        let rm_reg_name = get_reg_name((rm_reg, w_set));

        let text = if reg_is_destination {
            format!("mov {}, {}", reg_name, rm_reg_name)
        } else {
            format!("mov {}, {}", rm_reg_name, reg_name)
        };

        Some(text)
    } else {
        None
    }
}
