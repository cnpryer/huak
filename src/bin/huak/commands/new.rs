use std::io::{self, Write};

use clap::Command;
use huak::errors::CliResult;

use crate::utils::subcommand;

pub fn arg() -> Command<'static> {
    subcommand("new").about("Create a project from scratch.")
}

pub fn run() -> CliResult {
    let mut name = String::new();

    print!("Enter a name: ");

    let _ = io::stdout().flush();

    io::stdin()
        .read_line(&mut name)
        .expect("error reading from stdin");

    Ok(())
}
