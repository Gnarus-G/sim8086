use clap::Parser;
use sim8086::{
    decode::Decoder,
    exec::{clock_est::ClockEstimate, Executor},
};
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    // path to a binary file
    path: PathBuf,

    #[arg(short, long)]
    exec: bool,

    #[arg(short, long, requires = "exec")]
    dump: bool,

    /// Show clock cycle estimates for each instructions
    #[arg(short, long)]
    clock_estimate: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let buffer = fs::read(cli.path)?;

    if cli.exec {
        let mut exe = Executor::new(Decoder::new(&buffer));

        let mut clock_estimate_sum = 0;
        while let Some((i, diff)) = exe.execute_next() {
            if cli.clock_estimate {
                print!("{}", i);
                let curr_est = ClockEstimate::from(i);
                clock_estimate_sum += curr_est.value();
                println!(
                    " ; Clocks: +{} = {} {} | {:?}",
                    curr_est.value(),
                    clock_estimate_sum,
                    if let Some(ea) = curr_est.ea {
                        format!("({} + {}ea)", curr_est.base, ea)
                    } else {
                        "".to_string()
                    },
                    diff
                );
            } else {
                print!("{}", i);
                println!(" ; {:?}", diff);
            }
        }

        println!("\nFinal registers:");
        println!("{:#?}", exe.registers);

        if cli.dump {
            let data = exe.memory.dump();
            fs::write("sim86_memory_0.data", data)?
        }
    } else {
        println!("bits 16\n");

        let mut decoder = Decoder::new(&buffer);

        while let Some(i) = decoder.decode_next() {
            println!("{}", i);
        }
    }

    Ok(())
}
