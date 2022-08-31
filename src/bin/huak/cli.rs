use super::commands;
use clap::{self, ArgMatches};
use huak::errors::{CliError, CliResult};
use std::env;

/// Launch Huak's cli process.
pub fn main() -> CliResult {
    let args = commands::args();

    run(args.get_matches())
}

/// Command gating for Huak.
fn run(args: ArgMatches) -> CliResult {
    let (cmd, subargs) = match args.subcommand() {
        Some((cmd, subargs)) => (cmd, subargs),
        _ => unimplemented!(),
    };

    // Each command's behavior is triggered from the context of the cwd.
    let cwd_buff = env::current_dir()?;
    let cwd = cwd_buff.as_path();

    match cmd {
        "activate" => commands::activate::run(),
        "add" => commands::add::run(subargs),
        "build" => commands::build::run(),
        "clean" => commands::clean::run(),
        "help" => commands::help::run(),
        "init" => commands::init::run(),
        "new" => commands::new::run(cwd, subargs),
        "remove" => commands::remove::run(subargs),
        "run" => commands::run::run(subargs),
        "update" => commands::update::run(subargs),
        "version" => commands::version::run(),
        "clean-pycache" => commands::clean_pycache::run(),
        _ => Err(CliError::new(
            anyhow::format_err!("unrecognized command"),
            2,
        )),
    }
}
