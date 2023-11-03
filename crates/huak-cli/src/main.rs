//! The `huak` application.
//!
//! Huak implements a cli application with various subcommands.

mod cli;
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use huak_home::huak_home_dir;
use human_panic::setup_panic;
use std::{
    env,
    fs::create_dir_all,
    process::{exit, ExitCode},
};

mod error;

/// Launch Huak's cli process.
#[must_use]
pub fn main() -> ExitCode {
    setup_panic!();

    // Get home directory path.
    let Some(home) = huak_home_dir() else {
        eprintln!(
            "{}{} failed to resolve huak's home directory",
            "error".red(),
            ":".bold()
        );
        return ExitCode::FAILURE;
    };

    // If the home directory doesn't exist then spawn one. We only report an error if the
    // spawn fails due to anything other than the directory already existing.
    if !home.exists() {
        if let Err(e) = create_dir_all(home) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                eprintln!("{}{} {}", "error".red(), ":".bold(), e);
                return ExitCode::FAILURE;
            }
        }
    }

    // Capture and run CLI input.
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
