use std::path::PathBuf;

use crate::env::venv::{self, Venv};
use crate::errors::HuakError;
use crate::project::Config;

use std::collections::HashMap;
use std::fs;

use crate::{
    config::pyproject::toml::Toml,
    errors::HuakResult,
};

/// There are two kinds of project, application and library.
/// Application projects usually have one or more entrypoints in the form of
/// runnable scripts while library projects do not.
#[derive(Default, Eq, PartialEq)]
pub enum ProjectType {
    Library,
    #[default]
    Application,
}

/// The ``Project`` struct.
/// The ``Project`` struct provides and API for maintaining project. The pattern for
/// implementing a new command may involve creating operations that interacts
/// with a `Project`.
///
/// ``Project``s can be initialized using `from` or `new` initialization functions.
/// ```rust
/// use std::env;
/// use huak::project::Project;
///
/// let cwd = env::current_dir().unwrap();
/// let project = Project::from(cwd);
/// ```
#[derive(Default)]
pub struct Project {
    pub root: PathBuf,
    pub project_type: ProjectType,
    config: Config,
    venv: Option<Venv>,
}

impl Project {
    /// Initialize `Project` at a given path. This creates a `Project` without
    /// attempting to construct it through project artifact searches.
    /// ```rust
    /// use std::env;
    /// use huak::project::Project;
    ///
    /// let cwd = env::current_dir().unwrap();
    /// let project = Project::from(cwd);
    /// ```
    pub fn new(path: PathBuf, project_type: ProjectType) -> Project {
        Project {
            root: path,
            project_type,
            config: Config::default(),
            venv: None,
        }
    }

    /// Initialize `Project` from a given path. If a manifest isn't found
    /// at the path, then we search for a manifest and set the project root
    /// if it's found.
    /// ```rust
    /// use std::env;
    /// use huak::project::Project;
    ///
    /// let cwd = env::current_dir().unwrap();
    /// let project = Project::from(cwd);
    /// ```
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

        // We can't know the project type here, but it probably doesn't matter
        // much. We'll just use the default.
        let project_type = ProjectType::default();

        Ok(Project {
            root,
            project_type,
            config,
            venv: Some(venv),
        })
    }

    pub fn create_from_template(&self) -> HuakResult<(), > {
        let name = crate::utils::path::parse_filename(&self.root)?.to_string();

        // Create src subdirectory
        fs::create_dir_all(self.root.join("src"))?;

        if self.project_type == ProjectType::Library {
            fs::write(&self.root.join("src").join("__init__.py"),
                      "from .my_math import add_one, times_two\n")?;

            fs::write(&self.root.join("src").join("my_math.py"),
                      "# welcome to Huak's sample Python library.\ndef add_one(my_number):\
                      \n\treturn my_number + 1\n\n\ndef times_two(my_number):\n\treturn my_number * 2\n")?;

            fs::write(&self.root.join("test.py"),
                      "import unittest\nfrom src import add_one\n\n\n\
                      class TestSum(unittest.TestCase):\n\tdef test_lib_function(self):\n\t\t\
                      self.assertEqual(add_one(1), 2)\n\n\nif __name__ == '__main__':\n\tunittest.main()\n")?;
        } else {
            fs::create_dir_all(self.root.join("src").join(&name))?;
            fs::write(&self.root.join("src").join(&name).join("main.py"),
                      "# This is a sample Python script.\ndef print_hi(name):\
                      \n\tprint(f'Hello, {name}')\n\n\nif __name__ == '__main__':\n\tprint_hi('World')\n")?;
        }
        Ok(())
    }

    /// Create project toml.
    // TODO: Config implementations?
    pub fn create_toml(&self) -> HuakResult<Toml> {
        let mut toml = Toml::default();
        let name = crate::utils::path::parse_filename(&self.root)?.to_string();

        if matches!(self.project_type, ProjectType::Application) {
            let entrypoint = format!("{name}:run");
            toml.project.scripts = Some(HashMap::from([(name.clone(), entrypoint)]))
        }

        toml.project.name = name;

        Ok(toml)
    }

    /// Get a reference to the `Project` `Config`.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a reference to the `Project` `Venv`.
    // TODO: Decouple to operate on `Config` data.
    pub fn venv(&self) -> &Option<Venv> {
        &self.venv
    }

    /// Set the `Project`'s `Venv`.
    // TODO: Decouple to operate on `Config` data.
    pub fn set_venv(&mut self, venv: Venv) {
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
        let directory = tempdir().unwrap().into_path();
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
