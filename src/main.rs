use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("need a path to a binary file");

    let buffer = fs::read(path)?;

    println!("bits 16\n");

    buffer.windows(2).for_each(|a| {
        if let [h, l] = a {
            if let Some(Opcode::Mov) = get_opcode(h) {
                let d_mask: u8 = 0b00000010;
                let w_mask: u8 = 0b00000001;

                let w_set = w_mask & h;

                let reg_is_destination = (d_mask & h) == 1;
                let reg = (l & 0b00111000) >> 3;
                let rm_reg = l & 0b00000111;

                let reg_name = get_reg_name((reg, w_set));

                let rm_reg_name = get_reg_name((rm_reg, w_set));

                if reg_is_destination {
                    println!("mov {}, {}", reg_name, rm_reg_name);
                } else {
                    println!("mov {}, {}", rm_reg_name, reg_name);
                }
            }
        }
    });

    Ok(())
}

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
        (1, 0) => "cl",
        (1, 1) => "cx",
        (2, 0) => "dl",
        (2, 1) => "dx",
        (3, 0) => "bl",
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
