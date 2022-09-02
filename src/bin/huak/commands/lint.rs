use super::utils::{run_command, subcommand};
use clap::Command;
use huak::errors::{CliError, CliResult};
use huak::utils::get_venv_module_path;
use std::env;

pub fn arg() -> Command<'static> {
    subcommand("lint").about("Lint Python code.")
}

// TODO: Use pyproject.toml or .flake8 to override configuration.
pub fn run() -> CliResult {
    // This command runs from the context of the cwd.
    let cwd_buff = env::current_dir()?;
    let cwd = cwd_buff.as_path();

    let flake8_path = get_venv_module_path("flake8")?;
    let flake8_path = match flake8_path.to_str() {
        Some(p) => p,
        None => {
            return Err(CliError::new(
                anyhow::format_err!("failed to construct path to flake8 module"),
                2,
            ))
        }
    };
    let mypy_path = get_venv_module_path("mypy")?;
    let mypy_path = match mypy_path.to_str() {
        Some(p) => p,
        None => {
            return Err(CliError::new(
                anyhow::format_err!("failed to construct path to mypy module"),
                2,
            ))
        }
    };

    run_command(
        flake8_path,
        &["--ignore", "E203,W503", "--exclude", ".venv"],
        cwd,
    )?;
    run_command(mypy_path, &["."], cwd)?;

    Ok(())
}
