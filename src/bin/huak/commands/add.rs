use std::{env, fs};
use clap::{value_parser, Arg, ArgMatches, Command}; 
use serde_derive::{Serialize, Deserialize}; 
use huak::package::metadata::PyPi;
use huak::Dependency; 
use super::utils::{create_venv, install_dependency, subcommand};

use huak::{
    errors::{CliError, CliResult},
    pyproject::toml::Toml,
    utils::get_venv_module_path,
};

/// Get the `add` subcommand.
pub fn cmd() -> Command<'static> {
    subcommand("add")
        .arg(
            Arg::new("dependency")
                .required(true)
                .value_parser(value_parser!(String)),
        )
        .about("Add a Python module to the existing project.")
}

/// Run the `add` subcommand.
pub fn run(args: &ArgMatches) -> CliResult {
    let dependency = args.get_one::<String>("dependency");

    let path = format!("https://pypi.org/pypi/{}/json", dependency.unwrap());

    //info!("Requesting data from {}", path);
    let resp: reqwest::blocking::Response = reqwest::blocking::get(path)?;

    let json: PyPi = resp.json()?;
    println!("{:#?}", json);
    
    // Get the version

    let version = json.info.version;
    let name = json.info.name;
    let dep = Dependency{
        name,
        version,
    };

    // Proceed to add the Dependency to th toml file

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
    // Add to the toml file
    install_dependency(
        &pip_path,
        dep,
        cwd,
    )?;

    Ok(())
}