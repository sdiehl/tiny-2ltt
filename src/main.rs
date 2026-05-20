use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use tiny_2ltt::run_program;

#[derive(Parser, Debug)]
#[command(version, about = "tiny-2ltt: two-level type theory")]
struct Cli {
    file: PathBuf,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let src = match read_to_string(&cli.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {}: {e}", cli.file.display());
            return ExitCode::FAILURE;
        }
    };
    match run_program(&src) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
