use std::{env, fs};
use clap::{value_parser, Arg, ArgMatches, Command}; 
use serde_derive::{Serialize, Deserialize}; 
use huak::package::metadata::PyPi;
use super::utils::{subcommand};

use huak::package::python::PythonPackage;
use huak::{ 
    env::python::PythonEnvironment,
    env::venv::Venv,
    project::{python::PythonProject, Project, },
    errors::{CliError, CliResult},
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
    let dep = PythonPackage{
        name,
        version,
    }; 
     
    //let cwd = cwd_buff.as_path();
    let venv = Venv::default();
    venv.install_package(&dep)?;
 
    Ok(())
}