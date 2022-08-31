use super::utils::{run_command, subcommand};
use clap::Command;
use huak::errors::CliResult;
use huak::utils::get_venv_module_path;
use std::env;

pub fn arg() -> Command<'static> {
    subcommand("test").about("Test Python code.")
}

// TODO: Use pyproject.toml for configuration overrides.
pub fn run() -> CliResult {
    // This command runs from the context of the cwd.
    let cwd_buff = env::current_dir()?;
    let cwd = cwd_buff.as_path();

    let pytest_path = get_venv_module_path("pytest")?;

    run_command(&pytest_path, &[], cwd)?;

    Ok(())
}
