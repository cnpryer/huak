use super::commands;
use clap::{self, ArgMatches};
use huak::errors::{CliError, CliResult};

/// Launch Huak's cli process.
pub fn main() -> CliResult {
    let args = commands::args();

    run(args.get_matches())
}

/// Command gating for Huak.
fn run(args: ArgMatches) -> CliResult {
    // Check to see if the help command was passed.
    if let Some(cmd) = args.get_one::<bool>("help") {
        if *cmd {
            return commands::help::run(&args);
        }
    }

    // Check to see if the new command was passed.
    if let Some(cmd) = args.get_one::<bool>("new") {
        if *cmd {
            return commands::new::run(&args);
        }
    }

    // Check to see if the init command was passed.
    if let Some(cmd) = args.get_one::<bool>("init") {
        if *cmd {
            return commands::init::run(&args);
        }
    }

    // Check to see if the add command was passed.
    if args.get_one::<&str>("add").is_some() {
        return commands::add::run(&args);
    }

    // Check to see if the remove command was passed.
    if args.get_one::<&str>("remove").is_some() {
        return commands::remove::run(&args);
    }

    // Check to see if the update command was passed.
    if args.get_one::<&str>("udpate").is_some() {
        return commands::update::run(&args);
    }

    // Check to see if the build command was passed.
    if let Some(cmd) = args.get_one::<bool>("build") {
        if *cmd {
            return commands::build::run(&args);
        }
    }

    // Check to see if the clean command was passed.
    if let Some(cmd) = args.get_one::<bool>("clean") {
        if *cmd {
            return commands::clean::run(&args);
        }
    }

    // Check to see if the run command was passed.
    if args.get_many::<&str>("run").is_some() {
        return commands::run::run(&args);
    }

    // Check to see if the activate command was passed.
    if let Some(cmd) = args.get_one::<bool>("activate") {
        if *cmd {
            return commands::activate::run(&args);
        }
    }

    // Check to see if the version command was passed.
    if args.get_one::<bool>("version").is_some() {
        return commands::version::run(&args);
    }

    Err(CliError::new(
        anyhow::format_err!("unrecognized command"),
        2,
    ))
}
