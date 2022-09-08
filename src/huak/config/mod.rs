pub mod pyproject;
pub mod python;
pub mod requirements;

use std::path::Path;

use python::PythonConfig;

use self::{pyproject::toml::Toml, requirements::PythonPackage};

#[derive(Default)]
pub struct Config {
    pub name: String,
    pub version: String,
    dependencies: Vec<PythonPackage>,
}

impl Config {
    /// Initialize a `Config` by scanning a directory for configuration files like pyproject.toml.
    // TODO:
    //       - Improve scan. Initialy `new` will only expect pyproject.toml at the root of `from`.
    //       - Add other setup file types like requirements.txt.
    pub fn new(from: &Path) -> Result<Config, anyhow::Error> {
        let toml_path = from.join("pyproject.toml");

        if !toml_path.exists() {
            return Err(anyhow::format_err!("no pyproject.toml found"));
        }

        let toml = Toml::open(&toml_path)?;
        let name = toml.tool.huak.name;
        let version = toml.tool.huak.version;

        // Build a vector of dependencies. TODO: Main vs dev if needed.
        let mut dependencies: Vec<PythonPackage> = toml
            .tool
            .huak
            .dependencies
            .into_iter()
            .map(|d| PythonPackage {
                name: d.0,
                version: d.1.to_string(),
            })
            .collect();

        dependencies.extend(
            toml.tool
                .huak
                .dev_dependencies
                .into_iter()
                .map(|d| PythonPackage {
                    name: d.0,
                    version: d.1.to_string(),
                }),
        );

        Ok(Config {
            name,
            version,
            dependencies,
        })
    }
}

impl PythonConfig for Config {
    fn dependencies(&self) -> &Vec<PythonPackage> {
        &self.dependencies
    }
}
