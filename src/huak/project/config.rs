use std::path::{Path, PathBuf};

use crate::config::pyproject::toml::Toml;

use crate::package::python::PythonPackage;
use crate::utils;

// TODO: Env/programmatically.
const DEFAULT_SEARCH_STEPS: usize = 5;

/// Traits for Python-specific configuration.
pub trait PythonConfig {
    fn package_list(&self) -> Vec<PythonPackage>;
    fn optional_package_list(&self) -> Vec<PythonPackage>;
}

/// `Manifest` data the configuration uses to manage standard configuration
/// information.
// TODO: Isolated container of information.
#[derive(Default)]
pub struct Manifest {
    pub(crate) path: PathBuf,
    pub(crate) toml: Toml,
}

impl Manifest {
    /// Initialize a `Manifest` from a `path` pointing to a manifest file.
    /// Use `new()` to initiate from files including: pyproject.toml.
    // TODO: More than just toml.
    fn new(path: PathBuf) -> Result<Manifest, anyhow::Error> {
        // TODO
        if !path.ends_with("pyproject.toml") {
            return Ok(Manifest::default());
        }

        // Just use the toml for now.
        let toml = Toml::open(&path)?;

        Ok(Manifest { path, toml })
    }
}

// TODO: PythonConfig?
#[derive(Default)]
pub struct Config {
    manifest: Manifest,
}

impl Config {
    /// Initialize a `Config` by scanning a directory for configuration files like pyproject.toml.
    // TODO:
    //       - Improve scan. Initially `new` will only expect pyproject.toml at the root of `from`.
    //       - Add other setup file types like requirements.txt.
    pub fn from(path: &Path) -> Result<Config, anyhow::Error> {
        let manifest_path = utils::path::search_parents_for_filepath(
            path,
            "pyproject.toml",
            DEFAULT_SEARCH_STEPS,
        )?;

        if manifest_path.is_none() {
            return Ok(Config {
                manifest: Manifest::default(),
            });
        }

        let manifest_path = manifest_path.unwrap();
        let manifest = Manifest::new(manifest_path)?;

        Ok(Config { manifest })
    }

    // Initialize from a `Manifest`.
    pub fn from_manifest(manifest: Manifest) -> Config {
        Config { manifest }
    }

    /// Get a reference to the `Manifest`.
    pub(crate) fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Get a reference to the project name from manifest data.
    // TODO: Use more than toml.
    pub fn project_name(&self) -> &String {
        let table = &self.manifest.toml.project;

        &table.name
    }

    /// Get a reference to the project version from manifest data.
    // TODO: Use more than toml.
    pub fn project_version(&self) -> &String {
        let table = &self.manifest.toml.project;

        &table.version
    }
}

impl PythonConfig for Config {
    // Get vec of `PythonPackage`s from the manifest.
    // TODO: More than toml.
    fn package_list(&self) -> Vec<PythonPackage> {
        // Get huak's spanned table found in the Toml.
        let table = &self.manifest.toml.project;

        // Dependencies to list from.
        let from = &table.dependencies;

        // Collect into vector of owned `PythonPackage` data.
        from.iter()
            .map(|d| PythonPackage::from(d.clone()))
            .collect()
    }
    // Get vec of `PythonPackage`s from the manifest.
    // TODO: More than toml.
    fn optional_package_list(&self) -> Vec<PythonPackage> {
        // Get huak's spanned table found in the Toml.
        let table = &self.manifest.toml.project;

        // Dependencies to list from.
        let from = match &table.optional_dependencies {
            Some(vec) => vec,
            None => return vec![],
        };

        // Collect into vector of owned `PythonPackage` data.
        from.iter()
            .map(|d| PythonPackage::from(d.clone()))
            .collect()
    }
}
