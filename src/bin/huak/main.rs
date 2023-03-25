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
            // TODO: Still want to return sterr from wrapped commands which return non-zero error
            //       codes. Will revisit. Ideally if you HUAK_MUTE_COMMAND=1 you're expecting to
            //       ignore errors from wrapped commands, but I haven't done enough "integration"
            //       testing with huak to feel comfortable with that.
            if it.error.to_string().is_empty() {
                eprintln!("{}", it.error);
            } else {
                eprintln!("{}{} {}", "error".red(), ":".bold(), it.error);
            }
            it.exit_code
        }
    }
}
