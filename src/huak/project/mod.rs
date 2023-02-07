mod project_file;
pub use self::project_file::ProjectFile;

use crate::errors::HuakError;
use crate::{config::pyproject::toml::Toml, errors::HuakResult};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Every project has a root path and a project type.
///
/// ``Project``s can be initialized using `from` or `new` initialization functions.
/// ```rust
/// use std::env;
/// use huak::project::Project;
///
/// let cwd = env::current_dir().unwrap();
/// let project = Project::from_directory(cwd);
/// ```
pub struct Project {
    /// The path to the root directory of the project. Attempts to discover
    /// the root path locally are used if one is not provided. The default
    /// is the current working directory.
    pub root: PathBuf,
    /// A project can be application-like or library-like. A project defaults
    /// as library-type.
    pub project_type: ProjectType,
    /// Project file contains data about dependancies and more (TODO: put PEP here).
    pub project_file: ProjectFile,
}

/// There are two kinds of projects: application and library.
/// Application projects usually have one or more entrypoint(s) in the form of
/// runnable scripts while library projects do not. TODO: Document the different
/// advantages and disadvantages for each here.
#[derive(Default, Eq, PartialEq)]
pub enum ProjectType {
    #[default]
    Library,
    Application,
}

impl Project {
    /// Initialize `Project` at a given path. This creates a `Project` without
    /// attempting to construct it through project artifact searches.
    /// ```rust
    /// use std::env;
    /// use huak::project::Project;
    ///
    /// let cwd = env::current_dir().unwrap();
    /// let project = Project::from_directory(cwd);
    /// ```
    pub fn new(path: PathBuf, project_type: ProjectType) -> Project {
        let mut project_file = ProjectFile::default();

        // TODO: Don't unwrap_or_else like this.
        let name = project_name_from_root(path.as_path())
            .unwrap_or_else(|_| "".to_string());

        project_file.set_project_name(name.as_str());

        Project {
            root: path,
            project_type,
            project_file,
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
    /// let project = Project::from_directory(cwd);
    /// ```
    pub fn from_directory(path: PathBuf) -> HuakResult<Project> {
        // TODO: Builder.
        let project_file = ProjectFile::from_directory(&path)?;
        let project_file_filepath = match &project_file.filepath {
            Some(it) => it,
            None => {
                return Err(HuakError::PyProjectFileNotFound);
            }
        };

        // Set the root to the directory the manifest file was found.
        // TODO: This is probably not the right way to do this.
        let mut root = path;
        if let Some(parent) = project_file_filepath.parent() {
            root = parent.to_path_buf()
        }

        // We can't know the project type here, but it probably doesn't matter
        // much. We'll just use the default.
        let project_type = ProjectType::default();

        Ok(Project {
            root,
            project_type,
            project_file,
        })
    }

    pub fn bootstrap(&self) -> HuakResult<()> {
        if let Some(it) = &self.project_file.filepath {
            if it.exists() {
                return Err(HuakError::PyProjectTomlExistsError);
            }
        }

        let name = match self.project_file.project_name() {
            Some(it) => it,
            None => {
                return Err(HuakError::PyProjectFileNotFound);
            }
        };

        let version = match self.project_file.project_version() {
            Some(it) => it,
            None => return Err(HuakError::PyProjectVersionNotFound),
        };

        // Create package dir
        let src_path = self.root.join("src");
        fs::create_dir_all(src_path.join(name))?;

        // Add directories and example python files to new project
        match self.project_type {
            ProjectType::Library => {
                fs::create_dir_all(self.root.join("tests"))?;
                fs::write(
                    src_path.join(name).join("__init__.py"),
                    format!(
                        "__version__ = \"{version}\"
"
                    ),
                )?;
                fs::write(
                    self.root.join("tests").join("test_version.py"),
                    format!(
                        "\
from {name} import __version__


def test_version():
    __version__
"
                    ),
                )?;
            }
            ProjectType::Application => {
                fs::create_dir_all(src_path.join(name))?;
                fs::write(src_path.join(name).join("__init__.py"), "")?;
                fs::write(
                    src_path.join(name).join("main.py"),
                    "\
def main():
    print(\"Hello, World!\")


if __name__ == \"__main__\":
    main()
",
                )?;
            }
        }

        Ok(())
    }

    pub fn init_project_file(&mut self) -> HuakResult<()> {
        if self.project_file.filepath.is_none() {
            self.project_file.filepath = Some(self.root.join("pyproject.toml"));
        }

        if let Some(it) = &self.project_file.filepath {
            let mut toml = if it.exists() {
                Toml::open(it)?
            } else {
                Toml::default()
            };

            let name = project_name_from_root(&self.root)?;

            if matches!(self.project_type, ProjectType::Application)
                && toml.project.scripts.is_none()
            {
                let entrypoint = format!("{name}.main:main");
                toml.project.scripts =
                    Some(HashMap::from([(name.clone(), entrypoint)]))
            }

            toml.project.name = name;
            self.project_file.data = Some(toml);

            return Ok(());
        }

        Err(HuakError::PyProjectFileNotFound)
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Check if the project file lists any dependencies (including optional).
    pub fn has_dependencies(&self) -> bool {
        !self
            .project_file
            .dependency_list()
            .unwrap_or(&vec![])
            .is_empty()
            | !self
                .project_file
                .optional_dependencies()
                .unwrap_or(&HashMap::new())
                .is_empty()
    }
}

fn project_name_from_root(root: &Path) -> HuakResult<String> {
    Ok(crate::utils::path::parse_filename(root)?.replace('-', "_"))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    use crate::utils::{path::copy_dir, test_utils::get_resource_dir};

    #[test]
    fn from_directory() {
        let directory = tempdir().unwrap().into_path();
        let mock_dir = get_resource_dir().join("mock-project");

        copy_dir(&mock_dir, &directory.join("mock-project")).unwrap();

        let project1 =
            Project::from_directory(directory.join("mock-project")).unwrap();

        let project2 = Project::from_directory(
            directory
                .join("mock-project")
                .join("src")
                .join("mock_project"),
        )
        .unwrap();

        assert_eq!(project1.root, project2.root);
    }
}
