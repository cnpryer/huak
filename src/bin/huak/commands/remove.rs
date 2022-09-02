use std::{env, fs};

use super::utils::{run_command, subcommand};
use clap::{value_parser, Arg, ArgMatches, Command};
use huak::{
    errors::{CliError, CliResult},
    pyproject::toml::Toml,
    utils::get_venv_module_path,
};

pub fn arg() -> Command<'static> {
    subcommand("remove")
        .arg(
            Arg::new("dependency")
                .required(true)
                .value_parser(value_parser!(String)),
        )
        .about("Remove a dependency from the project.")
}

pub fn run(args: &ArgMatches) -> CliResult {
    let cwd_buff = env::current_dir()?;
    let cwd = cwd_buff.as_path();
    let toml_path = cwd.join("pyproject.toml");

    if !toml_path.exists() {
        eprintln!("pyproject.toml does not exist");
        return Ok(());
    }

    let dependency = match args.get_one::<String>("dependency") {
        Some(d) => d,
        None => {
            return Err(CliError::new(
                anyhow::format_err!("no dependency was provided"),
                2,
            ))
        }
    };

    let string = match fs::read_to_string(&toml_path) {
        Ok(s) => s,
        Err(_) => return Err(CliError::new(anyhow::format_err!("failed to read toml"), 2)),
    };

    let mut toml = match Toml::from(&string) {
        Ok(t) => t,
        Err(_) => {
            return Err(CliError::new(
                anyhow::format_err!("failed to build toml"),
                2,
            ))
        }
    };

    toml.remove_dependency(dependency, "dev");
    toml.remove_dependency(dependency, "main");

    // Attempt to prepare the serialization of pyproject.toml constructed.
    let string = match toml.to_string() {
        Ok(s) => s,
        Err(_) => {
            return Err(CliError::new(
                anyhow::format_err!("failed to serialize toml"),
                2,
            ))
        }
    };

    // Serialize pyproject.toml.
    fs::write(&toml_path, string)?;

    let pip_path = get_venv_module_path("pip")?;

    match pip_path.to_str() {
        Some(p) => run_command(p, &["uninstall", dependency, "-y"], cwd)?,
        _ => {
            return Err(CliError::new(
                anyhow::format_err!("failed to build path to pip module"),
                2,
            ))
        }
    };

    Ok(())
}
