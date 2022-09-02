use super::utils::{run_command, subcommand};
use clap::Command;
use huak::errors::{CliError, CliResult};
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

    let ptyest_path = get_venv_module_path("pytest")?;
    let pytest_path = match ptyest_path.to_str() {
        Some(p) => p,
        None => {
            return Err(CliError::new(
                anyhow::format_err!("failed to construct path to pytest module"),
                2,
            ))
        }
    };

    run_command(pytest_path, &[], cwd)?;

    Ok(())
}
