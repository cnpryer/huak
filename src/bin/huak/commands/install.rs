use std::{env, fs};

use super::utils::{create_venv, install_all_dependencies, subcommand};
use clap::Command;
use huak::{
    errors::{CliError, CliResult},
    pyproject::toml::Toml,
    utils::get_venv_module_path,
};

pub fn arg() -> Command<'static> {
    subcommand("install").about("Install the dependencies of an existing project.")
}

pub fn run() -> CliResult {
    let cwd_buff = env::current_dir()?;
    let cwd = cwd_buff.as_path();
    let toml_path = cwd.join("pyproject.toml");

    if !toml_path.exists() {
        create_venv("python", cwd, ".venv")?;
    }

    let string = match fs::read_to_string(toml_path) {
        Ok(s) => s,
        Err(_) => return Err(CliError::new(anyhow::format_err!("failed to read toml"), 2)),
    };

    let toml = match Toml::from(&string) {
        Ok(t) => t,
        Err(_) => {
            return Err(CliError::new(
                anyhow::format_err!("failed to build toml"),
                2,
            ))
        }
    };

    let pip_path = get_venv_module_path("pip")?;
    install_all_dependencies(&pip_path, &toml, cwd)?;

    Ok(())
}
