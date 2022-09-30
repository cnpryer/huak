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

    return match cli.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            err.exit_code
        }
    }
}

/*
/// Command gating for Huak.
fn run(args: ArgMatches) -> CliResult<()> {
    match args.subcommand() {
        Some(("activate", _)) => commands::activate::run(),
        Some(("add", subargs)) => commands::add::run(subargs),
        Some(("build", _)) => commands::build::run(),
        Some(("clean", _)) => commands::clean::run(),
        Some(("clean-pycache", _)) => commands::clean_pycache::run(),
        Some(("doc", subargs)) => commands::doc::run(subargs),
        Some(("help", _)) => commands::help::run(),
        Some(("fmt", subargs)) => commands::fmt::run(subargs),
        Some(("init", _)) => commands::init::run(),
        Some(("install", _)) => commands::install::run(),
        Some(("lint", _)) => commands::lint::run(),
        Some(("new", subargs)) => commands::new::run(subargs),
        Some(("publish", _)) => commands::publish::run(),
        Some(("remove", subargs)) => commands::remove::run(subargs),
        Some(("run", subargs)) => commands::run::run(subargs),
        Some(("update", subargs)) => commands::update::run(subargs),
        Some(("test", _)) => commands::test::run(),
        Some(("version", _)) => commands::version::run(),
        _ => Err(CliError::new(HuakError::UnknownCommand, ExitCode::FAILURE)),
    }
}
*/
