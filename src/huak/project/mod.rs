pub mod config;
pub mod python;
use std::path::PathBuf;

use crate::env::venv::{self, Venv};
use crate::errors::HuakError;

use self::config::Config;
use self::python::PythonProject;

#[derive(Default)]
pub struct Project {
    pub root: PathBuf,
    config: Config,
    venv: Option<Venv>,
}

impl Project {
    /// Initialize `Project` at a given path. This creates a `Project` without
    /// attempting to construct it through project artifact searches.
    pub fn new(path: PathBuf) -> Project {
        Project {
            root: path,
            config: Config::default(),
            venv: None,
        }
    }

    /// Initialize `Project` from a given path. If a manifest isn't found
    /// at the path, then we search for a manifest and set the project root
    /// if it's found.
    pub fn from(path: PathBuf) -> Result<Project, HuakError> {
        // TODO: Builder.
        let config = Config::from(&path)?;
        let venv = match Venv::from(&path) {
            Ok(v) => v,
            Err(HuakError::VenvNotFound) => {
                Venv::new(path.join(venv::DEFAULT_VENV_NAME))
            }
            Err(e) => return Err(e),
        };
        let manifest_path = &config.manifest().path;

        // Set the root to the directory the manifest file was found.
        // TODO: This is probably not the right way to do this.
        let mut root = path;
        if let Some(parent) = manifest_path.parent() {
            root = parent.to_path_buf()
        }

        Ok(Project {
            root,
            config,
            venv: Some(venv),
        })
    }
}

impl PythonProject for Project {
    /// Get a reference to the `Project` `Config`.
    fn config(&self) -> &Config {
        &self.config
    }

    /// Get a reference to the `Project` `Venv`.
    // TODO: Decouple to operate on `Config` data.
    fn venv(&self) -> &Option<Venv> {
        &self.venv
    }

    /// Set the `Project`'s `Venv`.
    // TODO: Decouple to operate on `Config` data.
    fn set_venv(&mut self, venv: Venv) {
        self.venv = Some(venv);
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    use crate::utils::{path::copy_dir, test_utils::get_resource_dir};

    #[test]
    fn from() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let mock_dir = get_resource_dir().join("mock-project");

        copy_dir(&mock_dir, &directory);

        let project1 = Project::from(directory.join("mock-project")).unwrap();
        let venv = Venv::new(project1.root.join(".venv"));

        venv.create().unwrap();

        let project2 = Project::from(
            directory
                .join("mock-project")
                .join("src")
                .join("mock_project"),
        )
        .unwrap();

        assert_eq!(project1.root, project2.root);
        assert_eq!(
            project1.venv().as_ref().unwrap().path,
            project2.venv().as_ref().unwrap().path
        );
    }
}
