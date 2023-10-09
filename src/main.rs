use clap::Parser;
use sim8086::{
    decode::{decode, Scanner},
    exec::Executor,
};
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
        let mut scanner = Scanner::new(&buffer);
        scanner.scan();
        let mut exe = Executor::new();
        exe.execute(&scanner.instructions);

        println!("{:#?}", exe.registers);
    } else {
        println!("bits 16\n");

        for i in decode(&buffer) {
            println!("{}", i);
        }
    }

    Ok(())
}
