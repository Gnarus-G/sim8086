use std::fs;

use sim8086::decode::decode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("need a path to a binary file");

    let buffer = fs::read(path)?;

    println!("bits 16\n");

    for i in decode(&buffer) {
        println!("{}", i);
    }

    Ok(())
}
