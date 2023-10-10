use std::{ffi::OsStr, fmt::Display, path::PathBuf, str::FromStr};

use indexmap::IndexMap;
use pep440_rs::Version;
use pep508_rs::Requirement;
use pyproject_toml::{BuildSystem, Project, PyProjectToml as ProjectToml};
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::{dependency::Dependency, Error, HuakResult};

const DEFAULT_METADATA_FILE_NAME: &str = "pyproject.toml";

#[derive(Debug)]
/// A `LocalMetadata` struct used to manage local `Metadata` files such as
/// the pyproject.toml (<https://peps.python.org/pep-0621/>).
pub struct LocalMetadata {
    /// The core `Metadata`.
    /// See https://packaging.python.org/en/latest/specifications/core-metadata/.
    metadata: Metadata, // TODO: https://github.com/cnpryer/huak/issues/574
    /// The path to the `LocalMetadata` file.
    path: PathBuf,
}

impl LocalMetadata {
    /// Initialize `LocalMetadata` from a path.
    pub fn new<T: Into<PathBuf>>(path: T) -> HuakResult<LocalMetadata> {
        let path = path.into();

        // NOTE: Currently only pyproject.toml files are supported.
        if path.file_name() != Some(OsStr::new(DEFAULT_METADATA_FILE_NAME)) {
            return Err(Error::Unimplemented(format!(
                "{} is not supported",
                path.display()
            )));
        }
        let local_metadata = pyproject_toml_metadata(path)?;

        Ok(local_metadata)
    }

    /// Create a `LocalMetadata` template.
    pub fn template<T: Into<PathBuf>>(path: T) -> LocalMetadata {
        LocalMetadata {
            metadata: Metadata {
                build_system: BuildSystem {
                    requires: vec![Requirement::from_str("hatchling").unwrap()],
                    build_backend: Some(String::from("hatchling.build")),
                    backend_path: None,
                },
                project: PyProjectToml::default().project.clone().unwrap(),
                tool: None,
            },
            path: path.into(),
        }
    }

    #[must_use]
    /// Get a reference to the core `Metadata`.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Get a mutable reference to the core `Metadata`.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    /// Write the `LocalMetadata` file to its path.
    pub fn write_file(&self) -> HuakResult<()> {
        let string = self.to_string_pretty()?;
        Ok(std::fs::write(&self.path, string)?)
    }

    /// Serialize the `Metadata` to a formatted string.
    pub fn to_string_pretty(&self) -> HuakResult<String> {
        Ok(toml_edit::ser::to_string_pretty(&self.metadata)?)
    }
}

impl Display for LocalMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.metadata)
    }
}

/// Create `LocalMetadata` from a pyproject.toml file.
fn pyproject_toml_metadata<T: Into<PathBuf>>(path: T) -> HuakResult<LocalMetadata> {
    let path = path.into();
    let pyproject_toml = PyProjectToml::new(&path)?;
    let project = match pyproject_toml.project.as_ref() {
        Some(it) => it,
        None => {
            return Err(Error::InternalError(format!(
                "{} is missing a project table",
                path.display()
            )))
        }
    }
    .to_owned();
    let build_system = pyproject_toml.build_system.clone();
    let tool = pyproject_toml.tool;

    let metadata = Metadata {
        build_system,
        project,
        tool,
    };
    let local_metadata = LocalMetadata { metadata, path };

    Ok(local_metadata)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
/// The `Metadata` of a `Package`.
///
/// See <https://peps.python.org/pep-0621/> for more about core metadata.
pub struct Metadata {
    /// The build system used for the `Package`.
    build_system: BuildSystem,
    /// The `Project` table.
    project: Project,
    /// The `Tool` table.
    tool: Option<Table>,
}

impl Metadata {
    #[allow(dead_code)]
    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn project_name(&self) -> &str {
        &self.project.name
    }

    pub fn set_project_name(&mut self, name: String) {
        self.project.name = name;
    }

    pub fn project_version(&self) -> Option<&Version> {
        self.project.version.as_ref()
    }

    pub fn dependencies(&self) -> Option<&[Requirement]> {
        self.project.dependencies.as_deref()
    }

    pub fn contains_dependency(&self, dependency: &Dependency) -> bool {
        if let Some(deps) = self.dependencies() {
            for d in deps {
                if d.name == dependency.name() {
                    return true;
                }
            }
        }
        false
    }

    pub fn contains_dependency_any(&self, dependency: &Dependency) -> bool {
        if self.contains_dependency(dependency) {
            return true;
        }

        if let Some(deps) = self.optional_dependencies().as_ref() {
            if deps.is_empty() {
                return false;
            }
            for d in deps.values().flatten() {
                if d.name == dependency.name() {
                    return true;
                }
            }
        }

        false
    }

    pub fn add_dependency(&mut self, dependency: &Dependency) {
        self.project
            .dependencies
            .get_or_insert_with(Vec::new)
            .push(dependency.requirement().to_owned());
    }

    pub fn optional_dependencies(&self) -> Option<&IndexMap<String, Vec<Requirement>>> {
        self.project.optional_dependencies.as_ref()
    }

    pub fn contains_optional_dependency(&self, dependency: &Dependency, group: &str) -> bool {
        if let Some(deps) = self.optional_dependencies().as_ref() {
            if let Some(g) = deps.get(group) {
                if deps.is_empty() {
                    return false;
                }
                for d in g {
                    if d.name == dependency.name() {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn optional_dependency_group(&self, group: &str) -> Option<&Vec<Requirement>> {
        self.project
            .optional_dependencies
            .as_ref()
            .and_then(|deps| deps.get(group))
    }

    pub fn add_optional_dependency(&mut self, dependency: &Dependency, group: &str) {
        self.project
            .optional_dependencies
            .get_or_insert_with(IndexMap::new)
            .entry(group.to_string())
            .or_default()
            .push(dependency.requirement().to_owned());
    }

    pub fn remove_dependency(&mut self, dependency: &Dependency) {
        self.project.dependencies.as_mut().and_then(|deps| {
            deps.iter()
                .position(|dep| dep.name == dependency.name())
                .map(|i| deps.remove(i))
        });
    }

    pub fn remove_optional_dependency(&mut self, dependency: &Dependency, group: &str) {
        self.project
            .optional_dependencies
            .as_mut()
            .and_then(|g| g.get_mut(group))
            .and_then(|deps| {
                deps.iter()
                    .position(|dep| dep.name == dependency.name())
                    .map(|i| deps.remove(i))
            });
    }

    pub fn add_script(&mut self, name: &str, entrypoint: &str) {
        self.project
            .scripts
            .get_or_insert_with(IndexMap::new)
            .entry(name.to_string())
            .or_insert(entrypoint.to_string());
    }
}

impl Default for Metadata {
    fn default() -> Self {
        // Initializing a `Package` from a `&str` would not include any additional
        // `Metadata` besides the name.
        let build_system = BuildSystem {
            requires: vec![Requirement::from_str("hatchling").unwrap()],
            build_backend: Some(String::from("hatchling.build")),
            backend_path: None,
        };

        let project = Project::new(String::from("Default Project"));

        Metadata {
            build_system,
            project,
            tool: None,
        }
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.project == other.project && self.tool == other.tool
    }
}

impl Eq for Metadata {}

/// A pyproject.toml as specified in PEP 621 with tool table.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PyProjectToml {
    #[serde(flatten)]
    inner: ProjectToml,
    tool: Option<Table>,
}

impl std::ops::Deref for PyProjectToml {
    type Target = ProjectToml;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for PyProjectToml {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl PyProjectToml {
    /// Initialize a `PyProjectToml` from its path.
    pub fn new<T: Into<PathBuf>>(path: T) -> HuakResult<PyProjectToml> {
        let contents = std::fs::read_to_string(path.into())?;
        let pyproject_toml: PyProjectToml = toml::from_str(&contents)?;

        Ok(pyproject_toml)
    }
}

impl Default for PyProjectToml {
    fn default() -> Self {
        Self {
            inner: ProjectToml::new(&default_pyproject_toml_contents(""))
                .expect("valid pyproject.toml contents"),
            tool: None,
        }
    }
}

#[must_use]
pub fn default_pyproject_toml_contents(name: &str) -> String {
    format!(
        r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "{name}"
version = "0.0.1"
description = ""
dependencies = []
"#
    )
}

#[must_use]
pub fn default_package_entrypoint_string(importable_name: &str) -> String {
    format!("{importable_name}.main:main")
}

#[must_use]
pub fn default_package_test_file_contents(importable_name: &str) -> String {
    format!(
        r#"from {importable_name} import __version__


def test_version():
    assert isinstance(__version__, str)
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_resources_dir_path;

    #[test]
    fn toml_from_path() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metadata = LocalMetadata::new(path).unwrap();

        assert_eq!(local_metadata.metadata.project_name(), "mock_project");
        assert_eq!(
            *local_metadata.metadata.project_version().unwrap(),
            Version::from_str("0.0.1").unwrap()
        );
        assert!(local_metadata.metadata.dependencies().is_some());
    }

    #[test]
    fn toml_to_string_pretty() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metadata = LocalMetadata::new(path).unwrap();

        assert_eq!(
            local_metadata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = ["click ==8.1.3"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest >=6",
    "black ==22.8.0",
    "isort ==5.12.0",
]
"#
        );
    }

    #[test]
    fn toml_dependencies() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metadata = LocalMetadata::new(path).unwrap();

        assert_eq!(
            local_metadata.metadata.dependencies().unwrap(),
            vec![Requirement::from_str("click==8.1.3").unwrap()]
        );
    }

    #[test]
    fn toml_optional_dependencies() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metadata = LocalMetadata::new(path).unwrap();

        assert_eq!(
            &**local_metadata
                .metadata
                .optional_dependency_group("dev")
                .unwrap(),
            vec![
                Requirement::from_str("pytest>=6").unwrap(),
                Requirement::from_str("black==22.8.0").unwrap(),
                Requirement::from_str("isort==5.12.0").unwrap()
            ]
        );
    }

    #[test]
    fn toml_add_dependency() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetadata::new(path).unwrap();
        let dep = Dependency::from(Requirement {
            name: "test".to_string(),
            extras: None,
            version_or_url: None,
            marker: None,
        });
        local_metadata.metadata.add_dependency(&dep);

        assert_eq!(
            local_metadata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = [
    "click ==8.1.3",
    "test",
]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest >=6",
    "black ==22.8.0",
    "isort ==5.12.0",
]
"#
        );
    }

    #[test]
    fn toml_add_optional_dependency() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetadata::new(path).unwrap();

        local_metadata
            .metadata
            .add_optional_dependency(&Dependency::from_str("test1").unwrap(), "dev");
        local_metadata
            .metadata
            .add_optional_dependency(&Dependency::from_str("test2").unwrap(), "new-group");
        assert_eq!(
            local_metadata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = ["click ==8.1.3"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest >=6",
    "black ==22.8.0",
    "isort ==5.12.0",
    "test1",
]
new-group = ["test2"]
"#
        );
    }

    #[test]
    fn toml_remove_dependency() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetadata::new(path).unwrap();
        local_metadata
            .metadata
            .remove_dependency(&Dependency::from_str("click").unwrap());

        assert_eq!(
            local_metadata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = []

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest >=6",
    "black ==22.8.0",
    "isort ==5.12.0",
]
"#
        );
    }

    #[test]
    fn toml_remove_optional_dependency() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetadata::new(path).unwrap();

        local_metadata
            .metadata
            .remove_optional_dependency(&Dependency::from_str("isort").unwrap(), "dev");
        assert_eq!(
            local_metadata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = ["click ==8.1.3"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest >=6",
    "black ==22.8.0",
]
"#
        );
    }
}
