use crate::{Error, HuakResult};
use huak_pyproject_toml::PyProjectToml;
use std::{ffi::OsStr, path::PathBuf, str::FromStr};
use toml_edit::Document;

const DEFAULT_MANIFEST_FILE_NAME: &str = "pyproject.toml";

/// A `LocalManifest` struct used to manage local manifest files such as the pyproject.toml (<https://peps.python.org/pep-0621/>).
pub struct LocalManifest {
    /// The manifest's data including core metadata about the project.
    /// See https://packaging.python.org/en/latest/specifications/core-metadata/.
    manifest_data: PyProjectToml, // TODO: https://github.com/cnpryer/huak/issues/574
    /// The path to the `LocalManifest` file.
    path: PathBuf,
}

impl LocalManifest {
    /// Initialize `LocalManifest` from a path.
    pub fn new<T: Into<PathBuf>>(path: T) -> HuakResult<LocalManifest> {
        let path = path.into();

        // NOTE: Currently only pyproject.toml files are supported.
        if path.file_name() != Some(OsStr::new(DEFAULT_MANIFEST_FILE_NAME)) {
            return Err(Error::Unimplemented(format!(
                "{} is not supported",
                path.display()
            )));
        }
        let manifest = read_local_manifest(path)?;

        Ok(manifest)
    }

    /// Create a `LocalManifest` template.
    pub fn template<T: Into<PathBuf>>(path: T) -> LocalManifest {
        LocalManifest {
            manifest_data: PyProjectToml {
                doc: Document::from_str(&default_pyproject_toml_contents("project name"))
                    .expect("template pyproject.toml contents"),
            },
            path: path.into(),
        }
    }

    /// Get a reference to the manifest data.
    #[must_use]
    pub fn manifest_data(&self) -> &PyProjectToml {
        &self.manifest_data
    }

    /// Get a mutable reference to the manifest data.
    pub fn manifest_data_mut(&mut self) -> &mut PyProjectToml {
        &mut self.manifest_data
    }

    /// Write the `LocalManifest` file to its path.
    pub fn write_file(&self) -> HuakResult<()> {
        Ok(self.manifest_data.write_toml(&self.path)?)
    }
}

/// Create `LocalManifest` from a pyproject.toml file.
fn read_local_manifest<T: Into<PathBuf>>(path: T) -> HuakResult<LocalManifest> {
    let path = path.into();
    let pyproject_toml = PyProjectToml::read_toml(&path)?;
    let local_manifest = LocalManifest {
        manifest_data: pyproject_toml,
        path,
    };

    Ok(local_manifest)
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
        let local_manifest = LocalManifest::new(path).unwrap();

        assert_eq!(
            local_manifest
                .manifest_data
                .project_name()
                .unwrap()
                .to_string(),
            "mock_project"
        );
        assert_eq!(
            *local_manifest
                .manifest_data
                .project_version()
                .unwrap()
                .to_string(),
            "0.0.1".to_string()
        );
        assert!(local_manifest
            .manifest_data
            .project_dependencies()
            .is_some());
    }

    #[ignore = "unsupported"]
    #[test]
    fn toml_to_string_pretty() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let local_manifest = LocalManifest::new(path).unwrap();

        assert_eq!(
            local_manifest.manifest_data.to_string(),
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

[tool.huak.task]
string = "python -c 'print(\"string\")'"
array = ["python", "-c", "print('array')"]
inline-cmd = { cmd = "python -c 'print(\"cmd\")'" }
inline-args = { args = ["python", "-c", "print('args')"] }
inline-program = { program = "python", args = ["-c", "print('program')"], env = { TESTING_HUAK = "test"} }
"#
        );
    }

    #[test]
    fn toml_dependencies() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let local_manifest = LocalManifest::new(path).unwrap();

        assert_eq!(
            local_manifest.manifest_data.project_dependencies().unwrap(),
            vec!["click == 8.1.7".to_string()]
        );
    }

    #[test]
    fn toml_optional_dependencies() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let local_manifest = LocalManifest::new(path).unwrap();

        assert_eq!(
            local_manifest
                .manifest_data
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
        let mut local_manifest = LocalManifest::new(path).unwrap();
        local_manifest
            .manifest_data
            .add_project_dependency("test")
            .formatted();

        assert_eq!(
            local_manifest.manifest_data.to_string(),
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

[tool.huak.task]
string = "python -c 'print(\"string\")'"
array = ["python", "-c", "print('array')"]
inline-cmd = { cmd = "python -c 'print(\"cmd\")'" }
inline-args = { args = ["python", "-c", "print('args')"] }
inline-program = { program = "python", args = ["-c", "print('program')"], env = { TESTING_HUAK = "test"} }
"#
        );
    }

    #[test]
    fn toml_add_optional_dependency() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_manifest = LocalManifest::new(path).unwrap();

        local_manifest
            .manifest_data
            .add_project_optional_dependency("test1", "dev");
        local_manifest
            .manifest_data
            .add_project_optional_dependency("test2", "new-group");
        assert_eq!(
            local_manifest.manifest_data.formatted().to_string(),
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

[tool.huak.task]
string = "python -c 'print(\"string\")'"
array = ["python", "-c", "print('array')"]
inline-cmd = { cmd = "python -c 'print(\"cmd\")'" }
inline-args = { args = ["python", "-c", "print('args')"] }
inline-program = { program = "python", args = ["-c", "print('program')"], env = { TESTING_HUAK = "test"} }
"#
        );
    }

    #[test]
    fn toml_remove_dependency() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_manifest = LocalManifest::new(path).unwrap();
        local_manifest
            .manifest_data
            .remove_project_dependency("click");

        assert_eq!(
            local_manifest.manifest_data.formatted().to_string(),
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

[tool.huak.task]
string = "python -c 'print(\"string\")'"
array = ["python", "-c", "print('array')"]
inline-cmd = { cmd = "python -c 'print(\"cmd\")'" }
inline-args = { args = ["python", "-c", "print('args')"] }
inline-program = { program = "python", args = ["-c", "print('program')"], env = { TESTING_HUAK = "test"} }
"#
        );
    }

    #[test]
    fn toml_remove_optional_dependency() {
        let path = dev_resources_dir()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_manifest = LocalManifest::new(path).unwrap();

        local_manifest
            .manifest_data
            .remove_project_optional_dependency("ruff", "dev");
        assert_eq!(
            local_manifest.manifest_data.formatted().to_string(),
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

[tool.huak.task]
string = "python -c 'print(\"string\")'"
array = ["python", "-c", "print('array')"]
inline-cmd = { cmd = "python -c 'print(\"cmd\")'" }
inline-args = { args = ["python", "-c", "print('args')"] }
inline-program = { program = "python", args = ["-c", "print('program')"], env = { TESTING_HUAK = "test"} }
"#
        );
    }
}
