use std::path::{Path, PathBuf};

use crate::config::pyproject::toml::Toml;

use crate::config::requirements::PythonPackage;

// TODO: Env/programatically.
const DEFAULT_SEARCH_STEPS: usize = 5;

pub trait PythonConfig {
    fn dependency_list(&self, kind: &str) -> Vec<PythonPackage>;
}

#[derive(Default)]
pub struct Manifest {
    pub path: PathBuf,
    pub toml: Toml,
}

impl Manifest {
    fn new(path: PathBuf) -> Manifest {
        if !path.ends_with("pyproject.toml") {
            return Manifest::default();
        }

        // Just use the toml for now.
        // TODO: Manage failure to open.
        let toml = Toml::open(&path).unwrap_or_default();

        Manifest { path, toml }
    }
}

// TODO: PythonConfig?
#[derive(Default)]
pub struct Config {
    pub manifest: Manifest,
}

impl Config {
    /// Initialize a `Config` by scanning a directory for configuration files like pyproject.toml.
    // TODO:
    //       - Improve scan. Initialy `new` will only expect pyproject.toml at the root of `from`.
    //       - Add other setup file types like requirements.txt.
    pub fn from(path: &Path) -> Result<Config, anyhow::Error> {
        let manifest_path = utils::find_manifest(path, DEFAULT_SEARCH_STEPS)?;

        if manifest_path.is_none() {
            eprintln!("no manifest found");
            eprintln!("creating default manifest");

            return Ok(Config {
                manifest: Manifest::default(),
            });
        }

        let manifest_path = manifest_path.unwrap();
        let manifest = Manifest::new(manifest_path);

        Ok(Config { manifest })
    }

    // Get project name from manifest data. TODO: Use more than toml.
    pub fn project_name(&self) -> &String {
        let table = &self.manifest.toml.tool.huak;

        &table.name
    }

    // Get project version from manifest data. TODO: Use more than toml.
    pub fn project_version(&self) -> &String {
        let table = &self.manifest.toml.tool.huak;

        &table.version
    }
}

impl PythonConfig for Config {
    // Get vec of dependencies from the manifest.
    // TODO: More than toml.
    fn dependency_list(&self, kind: &str) -> Vec<PythonPackage> {
        let table = &self.manifest.toml.tool.huak;

        let from = match kind {
            "dev" => &table.dev_dependencies,
            _ => &table.dependencies,
        };

        from.into_iter()
            .map(|d| PythonPackage {
                name: d.0.to_string(),
                version: d.1.as_str().unwrap().to_string(),
            })
            .collect()
    }
}

mod utils {
    use std::path::{Path, PathBuf};

    // Only checks for toml right now. TODO
    pub fn find_manifest(
        from: &Path,
        steps: usize,
    ) -> Result<Option<PathBuf>, anyhow::Error> {
        if steps == 0 {
            return Ok(None);
        }

        let filename = "pyproject.toml";

        if from.join(filename).exists() {
            return Ok(Some(from.join(filename)));
        }

        if let Some(parent) = from.parent() {
            return find_manifest(parent, steps - 1);
        }

        Ok(None)
    }
}
