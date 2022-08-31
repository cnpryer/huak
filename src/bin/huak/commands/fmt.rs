use super::utils::{run_command, subcommand};
use clap::{arg, ArgAction, ArgMatches, Command};
use huak::errors::CliResult;
use huak::utils::get_venv_module_path;
use std::env;

pub fn arg() -> Command<'static> {
    subcommand("fmt").about("Format Python code.").arg(
        arg!(--check)
            .id("check")
            .takes_value(false)
            .action(ArgAction::SetTrue)
            .help("Check if Python code is formatted."),
    )
}

// TODO: Use pyproject.toml for configuration overrides.
pub fn run(args: &ArgMatches) -> CliResult {
    // This command runs from the context of the cwd.
    let cwd_buff = env::current_dir()?;
    let dir = cwd_buff.as_path();

    let black_path = get_venv_module_path("black")?;

    match args.get_one::<bool>("check").unwrap() {
        true => run_command(&black_path, &[".", "--line-length", "79", "--check"], dir)?,
        false => run_command(&black_path, &[".", "--line-length", "79"], dir)?,
    };

    Ok(())
}
