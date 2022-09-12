//! The `huak` application.
//!
//! Huak implements a cli application with various subcommands.
use huak::errors::CliResult;

mod cli;
mod commands;

fn main() -> CliResult {
    cli::main()
}
