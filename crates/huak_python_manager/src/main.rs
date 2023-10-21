use clap::Parser;
use cli::Cli;
use colored::Colorize;
use human_panic::setup_panic;

mod cli;
mod install;
mod releases;
mod resolve;

fn main() {
    setup_panic!();

    if let Err(e) = Cli::parse().run() {
        eprintln!("{}{} {}", "error".red(), ":".bold(), e);
    }
}
