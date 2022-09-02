use super::utils::{run_command, subcommand};
use clap::{arg, ArgAction, ArgMatches, Command};
use huak::errors::{CliError, CliResult};
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
    let cwd = cwd_buff.as_path();

    let black_path = get_venv_module_path("black")?;
    let black_path = match black_path.to_str() {
        Some(p) => p,
        None => {
            return Err(CliError::new(
                anyhow::format_err!("failed to construct path to black module"),
                2,
            ))
        }
    };

    match args.get_one::<bool>("check").unwrap() {
        true => run_command(black_path, &[".", "--line-length", "79", "--check"], cwd)?,
        false => run_command(black_path, &[".", "--line-length", "79"], cwd)?,
    };

    Ok(())
}
