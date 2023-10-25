use crate::cli::Cli;
use clap::Parser;
use colored::Colorize;
use human_panic::setup_panic;

mod cli;

fn main() {
    setup_panic!();

    if let Err(e) = Cli::parse().run() {
        eprintln!("{}{} {}", "error".red(), ":".bold(), e);
    }
}
