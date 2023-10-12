use clap::Parser;
use sim8086::{decode::Decoder, exec::Executor};
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    // path to a binary file
    path: PathBuf,

    #[arg(short, long)]
    exec: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let buffer = fs::read(cli.path)?;

    if cli.exec {
        let mut exe = Executor::new(Decoder::new(&buffer));

        while let Some((i, diff)) = exe.execute_next() {
            print!("{}", i);
            println!(" ; {:?}", diff);
        }

        println!("\nFinal registers:");
        println!("{:#?}", exe.registers);
    } else {
        println!("bits 16\n");

        let mut decoder = Decoder::new(&buffer);

        while let Some(i) = decoder.decode_next() {
            println!("{}", i);
        }
    }

    Ok(())
}
