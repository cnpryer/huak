use clap::{self, App, AppSettings};
use huak::{
    errors::{CliError, CliResult},
    pyproject::toml::Toml,
    Dependency,
};
use std::{path::Path, process};

/// Create a clap subcommand.
pub fn subcommand(name: &'static str) -> clap::Command<'static> {
    App::new(name)
        .dont_collapse_args_in_usage(true)
        .setting(AppSettings::DeriveDisplayOrder)
}

/// Creates a venv using python -m venv `name` from a given directory.
pub fn create_venv(python_target: &str, dir: &Path, name: &str) -> CliResult {
    run_command(python_target, &["-m", "venv", name], dir)?;

    Ok(())
}

/// Installs all dependencies found in Toml from a dir.
pub fn install_all_dependencies(pip_path: &Path, toml: &Toml, dir: &Path) -> CliResult {
    let (dependencies, dev_dependencies) = (
        toml.tool().huak().dependencies(),
        toml.tool().huak().dev_dependencies(),
    );

    // Install main dependencies listed in pyproject.toml.
    for (name, version) in dependencies {
        install_dependency(
            pip_path,
            Dependency {
                name: name.to_string(),
                version: version.as_str().unwrap().to_string(),
            },
            dir,
        )?;
    }

    // Inntall dev dependencies listed in pyproject.toml.
    for (name, version) in dev_dependencies {
        install_dependency(
            pip_path,
            Dependency {
                name: name.to_string(),
                version: version.as_str().unwrap().to_string(),
            },
            dir,
        )?;
    }

    Ok(())
}

/// Installs a Python dependency using std::process::Command from a dir.
pub fn install_dependency(pip_path: &Path, dependency: Dependency, dir: &Path) -> CliResult {
    let command = pip_path.to_str();

    if command.is_none() {
        return Err(CliError::new(
            anyhow::format_err!("failed to construct pip command"),
            2,
        ));
    }

    let command = command.unwrap();
    let args = [
        "install",
        &format!("{}=={}", dependency.name, dependency.version),
    ];

    run_command(command, &args, dir)?;

    Ok(())
}

/// Run a command using std::process::Command
pub fn run_command(command: &str, args: &[&str], dir: &Path) -> CliResult {
    let output = process::Command::new(command)
        .args(args)
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        return Err(CliError::new(
            anyhow::format_err!("failed to run command '{}' with {:?}", command, args),
            2,
        ));
    }

    Ok(())
}
