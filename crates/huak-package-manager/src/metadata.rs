use crate::{Error, HuakResult};
use huak_pyproject_toml::PyProjectToml;
use std::{ffi::OsStr, path::PathBuf, str::FromStr};
use toml_edit::Document;

const DEFAULT_METADATA_FILE_NAME: &str = "pyproject.toml";

/// A `LocalMetadata` struct used to manage local `Metadata` files such as
/// the pyproject.toml (<https://peps.python.org/pep-0621/>).
pub struct LocalMetadata {
    /// The core `Metadata`.
    /// See https://packaging.python.org/en/latest/specifications/core-metadata/.
    metadata: PyProjectToml, // TODO: https://github.com/cnpryer/huak/issues/574
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
            metadata: PyProjectToml {
                doc: Document::from_str(&default_pyproject_toml_contents("project name"))
                    .expect("template pyproject.toml contents"),
            },
            path: path.into(),
        }
    }

    /// Get a reference to the core `Metadata`.
    #[must_use]
    pub fn metadata(&self) -> &PyProjectToml {
        &self.metadata
    }

    /// Get a mutable reference to the core `Metadata`.
    pub fn metadata_mut(&mut self) -> &mut PyProjectToml {
        &mut self.metadata
    }

    /// Write the `LocalMetadata` file to its path.
    pub fn write_file(&self) -> HuakResult<()> {
        Ok(self.metadata.write_toml(&self.path)?)
    }
}

/// Create `LocalMetadata` from a pyproject.toml file.
fn pyproject_toml_metadata<T: Into<PathBuf>>(path: T) -> HuakResult<LocalMetadata> {
    let path = path.into();
    let pyproject_toml = PyProjectToml::read_toml(&path)?;
    let local_metadata = LocalMetadata {
        metadata: pyproject_toml,
        path,
    };

    Ok(local_metadata)
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
    use huak_dev::dev_resources_dir;

    #[test]
    fn toml_from_path() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metadata = LocalMetadata::new(path).unwrap();

        assert_eq!(
            local_metadata.metadata.project_name().unwrap().to_string(),
            "mock_project"
        );
        assert_eq!(
            *local_metadata
                .metadata
                .project_version()
                .unwrap()
                .to_string(),
            "0.0.1".to_string()
        );
        assert!(local_metadata.metadata.project_dependencies().is_some());
    }

    #[ignore = "unsupported"]
    #[test]
    fn toml_to_string_pretty() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metadata = LocalMetadata::new(path).unwrap();

        assert_eq!(
            local_metadata.metadata.to_string(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = ["click == 8.1.3"]

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
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metadata = LocalMetadata::new(path).unwrap();

        assert_eq!(
            local_metadata.metadata.project_dependencies().unwrap(),
            vec!["click == 8.1.7".to_string()]
        );
    }

    #[test]
    fn toml_optional_dependencies() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metadata = LocalMetadata::new(path).unwrap();

        assert_eq!(
            local_metadata
                .metadata
                .project_optional_dependencies()
                .unwrap()
                .get("dev")
                .unwrap(),
            &vec!["pytest == 7.4.3".to_string(), "ruff".to_string(),]
        );
    }

    #[test]
    fn toml_add_dependency() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetadata::new(path).unwrap();
        local_metadata
            .metadata
            .add_project_dependency("test")
            .formatted();

        assert_eq!(
            local_metadata.metadata.to_string(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = [
    "click == 8.1.7",
    "test",
]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest == 7.4.3",
    "ruff",
]
"#
        );
    }

    #[test]
    fn toml_add_optional_dependency() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetadata::new(path).unwrap();

        local_metadata
            .metadata
            .add_project_optional_dependency("test1", "dev");
        local_metadata
            .metadata
            .add_project_optional_dependency("test2", "new-group");
        assert_eq!(
            local_metadata.metadata.formatted().to_string(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = [
    "click == 8.1.7",
]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest == 7.4.3",
    "ruff",
    "test1",
]
new-group = [
    "test2",
]
"#
        );
    }

    #[test]
    fn toml_remove_dependency() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetadata::new(path).unwrap();
        local_metadata.metadata.remove_project_dependency("click");

        assert_eq!(
            local_metadata.metadata.formatted().to_string(),
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
    "pytest == 7.4.3",
    "ruff",
]
"#
        );
    }

    #[test]
    fn toml_remove_optional_dependency() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetadata::new(path).unwrap();

        local_metadata
            .metadata
            .remove_project_optional_dependency("ruff", "dev");
        assert_eq!(
            local_metadata.metadata.formatted().to_string(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = [
    "click == 8.1.7",
]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest == 7.4.3",
]
"#
        );
    }
}
