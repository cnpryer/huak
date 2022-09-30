//! The `huak` application.
//!
//! Huak implements a cli application with various subcommands.
use std::process::ExitCode;

use clap::Parser;

mod commands;
use commands::Cli;

/// Launch Huak's cli process.
pub fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            err.exit_code
        }
    }
}
