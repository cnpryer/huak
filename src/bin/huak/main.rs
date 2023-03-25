//! The `huak` application.
//!
//! Huak implements a cli application with various subcommands.
mod cli;
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use human_panic::setup_panic;
use std::process::ExitCode;
mod error;

/// Launch Huak's cli process.
pub fn main() -> ExitCode {
    setup_panic!();
    let cli = Cli::parse();
    match cli.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(it) => {
            if it.error.to_string().is_empty() {
                eprintln!("{}", it.error);
            } else {
                eprintln!("{}{} {}", "error".red(), ":".bold(), it.error);
            }
            it.exit_code
        }
    }
}
