//! The `huak` application.
//!
//! Huak implements a cli application with various subcommands.

mod cli;
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use human_panic::setup_panic;
use std::process::{exit, ExitCode};

mod error;

/// Launch Huak's cli process.
#[must_use]
pub fn main() -> ExitCode {
    setup_panic!();

    match Cli::parse().run() {
        Ok(0) => ExitCode::SUCCESS,
        // Lazy-like exit of a subprocess failure. TODO: https://github.com/cnpryer/huak/issues/631
        Ok(code) => exit(code),
        Err(e) => {
            // TODO(cnpryer):
            //   - Make subprocess hack more clear
            //   - https://github.com/cnpryer/huak/issues/318
            if e.error.to_string().is_empty() {
                eprintln!("{}", e.error);
            } else {
                eprintln!("{}{} {}", "error".red(), ":".bold(), e.error);
            }
            e.exit_code
        }
    }
}
