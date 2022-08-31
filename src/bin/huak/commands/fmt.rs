use super::utils::{run_command, subcommand};
use clap::{arg, ArgAction, ArgMatches, Command};
use huak::errors::{CliError, CliResult};
use huak::utils::get_venv_bin;
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

pub fn run(args: &ArgMatches) -> CliResult {
    // This command runs from the context of the cwd.
    let cwd_buff = env::current_dir()?;
    let dir = cwd_buff.as_path();

    // TODO: Use environment management to determine venv target.
    //       This assumes there is a .venv in cwd.
    let black_path = dir.join(".venv").join(get_venv_bin()).join("black");
    let black_path = black_path.as_os_str().to_str();

    if black_path.is_none() {
        return Err(CliError::new(
            anyhow::format_err!("failed to create flake8 path"),
            2,
        ));
    }

    match args.get_one::<bool>("check").unwrap() {
        true => run_command(
            black_path.unwrap(),
            &[".", "--line-length", "79", "--check"],
            dir,
        )?,
        false => run_command(black_path.unwrap(), &[".", "--line-length", "79"], dir)?,
    };

    Ok(())
}
