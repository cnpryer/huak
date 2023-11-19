//! ## huak-pyproject-toml
//!
//! Projects have manifest files named pyproject.toml (as specified in [PEP 517](https://peps.python.org/pep-0517/)). The data can consist of project metadata as well as tooling configuration. Here's Huak's pyproject.toml
//!
//! ```toml
//! [project]
//! name = "huak"
//! version = "0.0.20a1"
//! description = "A Python package manager written in Rust and inspired by Cargo."
//! authors = [
//!     {email = "cnpryer@gmail.com"},
//!     {name = "Chris Pryer"}
//! ]
//! readme = "README.md"
//! license = {text = "MIT"}
//! requires-python = ">=3.7"
//! classifiers = [
//!     "Programming Language :: Rust",
//! ]
//!
//! [project.urls]
//! issues = "https://github.com/cnpryer/huak/issues"
//! documentation = "https://github.com/cnpryer/huak"
//! homepage = "https://github.com/cnpryer/huak"
//! repository = "https://github.com/cnpryer/huak"
//!
//! [tool.maturin]
//! bindings = "bin"
//! manifest-path = "crates/huak-cli/Cargo.toml"
//! module-name = "huak"
//! python-source = "python"
//! strip = true
//!
//! [build-system]
//! requires = ["maturin>=0.14,<0.15"]
//! build-backend = "maturin"
//!
//! [tool.huak]
//! toolchain = "default"
//! ```
//!
//! This manifest identifies the workspace for the Huak project. It contains metadata about the project, it's authors, build configuration, and config for other tools like maturin. At the bottom is the `[tool.huak]` table (see [PEP 518](https://peps.python.org/pep-0518/#tool-table)).
//!
//! ### `[tool.huak]`
//!
//! Huak's pyproject.toml implementation needs to expect a tool table, especially Huak's tool table. See:
//!
//! - #833
//! - #814
//! - #815
//!
//! Example:
//! ```toml
//! [tool.huak]
//! toolchain = "3.11.6"
//! repositories = { package = "url to repo" }  # TODO
//!
//! [tool.huak.run]  # TODO: Compare with new project.run table.
//! hello-world = "python -c 'print('hello, world.')'"
//!
//! [tool.huak.workspace]
//! members = ["projects/*"]
//! ```

pub use error::Error;
use pep508_rs::Requirement;
use std::{collections::HashMap, fmt::Display, path::Path, str::FromStr};
use toml_edit::{Array, Document, Formatted, Item, Table, Value};
pub use utils::value_to_sanitized_string;
use utils::{format_array, format_table, sanitize_str};

mod error;
mod utils;

#[derive(Clone)]
/// Huak's `PyProjectToml` implementation.
///
/// - Core `PyProjectToml`
/// - Tool table
///   - Huak's table
pub struct PyProjectToml {
    pub doc: Document,
}

impl Default for PyProjectToml {
    fn default() -> Self {
        Self::new()
    }
}

impl PyProjectToml {
    #[must_use]
    pub fn new() -> Self {
        Self {
            doc: Document::new(),
        }
    }

    /// Read `PyProjectToml` from a toml file.
    pub fn read_toml<T: AsRef<Path>>(path: T) -> Result<Self, Error> {
        read_pyproject_toml(path)
    }

    pub fn formatted(&mut self) -> &mut Self {
        format_pyproject_toml(self);
        self
    }

    /// Write the `PyProjectToml` to a toml file.
    pub fn write_toml<T: AsRef<Path>>(&self, path: T) -> Result<(), Error> {
        write_pyproject_toml(self, path)
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Item> {
        self.doc.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Item> {
        self.doc.get_mut(key)
    }

    // TODO(cnpryer): Tablelike or section(?)
    #[must_use]
    pub fn project_table(&self) -> Option<&Table> {
        self.get("project").and_then(Item::as_table)
    }

    pub fn project_table_mut(&mut self) -> Option<&mut Table> {
        self.get_mut("project").and_then(Item::as_table_mut)
    }

    // TODO(cnpryer): Tablelike or section(?)
    #[must_use]
    pub fn tool_table(&self) -> Option<&Table> {
        self.get("tool").and_then(Item::as_table)
    }

    // TODO(cnpryer): Tablelike or section(?)
    pub fn tool_table_mut(&mut self) -> Option<&mut Table> {
        self.get_mut("tool").and_then(Item::as_table_mut)
    }

    #[must_use]
    pub fn project_name(&self) -> Option<String> {
        self.project_table()
            .and_then(|it| it.get("name"))
            .and_then(Item::as_value)
            .map(value_to_sanitized_string)
    }

    pub fn set_project_name(&mut self, name: &str) -> &mut Self {
        self.doc["project"]["name"] = Item::Value(Value::String(Formatted::new(name.to_string())));
        self
    }

    #[must_use]
    pub fn project_version(&self) -> Option<String> {
        self.project_table()
            .and_then(|it| it.get("version"))
            .and_then(Item::as_value)
            .map(value_to_sanitized_string)
    }

    pub fn set_project_version(&mut self, version: &str) -> &mut Self {
        self.doc["project"]["version"] =
            Item::Value(Value::String(Formatted::new(version.to_string())));
        self
    }

    #[must_use]
    pub fn project_description(&self) -> Option<String> {
        self.project_table()
            .and_then(|it| it.get("description"))
            .and_then(Item::as_value)
            .map(value_to_sanitized_string)
    }

    pub fn set_project_description(&mut self, description: &str) -> &mut Self {
        self.doc["project"]["version"] =
            Item::Value(Value::String(Formatted::new(description.to_string())));
        self
    }

    #[must_use]
    pub fn project_dependencies(&self) -> Option<Vec<String>> {
        let Some(array) = self
            .project_table()
            .and_then(|it| it.get("dependencies"))
            .and_then(Item::as_array)
        else {
            return None;
        };

        Some(
            array
                .into_iter()
                .map(value_to_sanitized_string)
                .collect::<Vec<_>>(),
        )
    }

    pub fn project_dependencies_mut(&mut self) -> Option<&mut Array> {
        self.project_table_mut()
            .and_then(|it| it.get_mut("dependencies"))
            .and_then(Item::as_array_mut)
    }

    pub fn add_project_dependency(&mut self, dependency: &str) -> &mut Self {
        let item = &mut self.doc["project"]["dependencies"];

        add_array_str(item, dependency);

        self
    }

    #[must_use]
    pub fn contains_project_dependency_any(&self, dependency: &str) -> bool {
        self.project_dependencies().map_or(false, |it| {
            it.iter().any(|v| matches_dependency(v, dependency))
        }) || self.contains_project_optional_dependency_any(dependency)
    }

    #[must_use]
    pub fn contains_project_dependency(&self, dependency: &str) -> bool {
        self.project_dependencies().map_or(false, |it| {
            it.iter().any(|v| matches_dependency(v, dependency))
        })
    }

    pub fn remove_project_dependency(&mut self, dependency: &str) -> &mut Self {
        let item = &mut self.doc["project"]["dependencies"];

        remove_array_dependency(item, dependency);

        self
    }

    #[must_use]
    pub fn project_optional_dependency_groups(&self) -> Option<Vec<String>> {
        // TODO(cnpryer): Perf
        self.project_optional_dependencies()
            .map(|it| it.keys().cloned().collect::<Vec<_>>())
    }

    #[must_use]
    pub fn project_optional_dependencies(&self) -> Option<HashMap<String, Vec<String>>> {
        let Some(table) = self
            .project_table()
            .and_then(|it| it.get("optional-dependencies"))
            .and_then(Item::as_table)
        else {
            return None;
        };

        let mut deps = HashMap::new();
        let groups = table.iter().map(|(k, _)| k).collect::<Vec<_>>();

        for it in &groups {
            if let Some(array) = table.get(it).and_then(|item| item.as_array()) {
                deps.insert(
                    sanitize_str(it),
                    array
                        .iter()
                        .map(value_to_sanitized_string)
                        .collect::<Vec<_>>(),
                );
            }
        }

        Some(deps)
    }

    pub fn project_optional_dependencies_mut(&mut self) -> Option<&mut Table> {
        self.project_table_mut()
            .and_then(|it| it.get_mut("optional-dependencies"))
            .and_then(Item::as_table_mut)
    }

    pub fn add_project_optional_dependency(&mut self, dependency: &str, group: &str) -> &mut Self {
        let item: &mut Item = &mut self.doc["project"]["optional-dependencies"];

        if item.is_none() {
            *item = Item::Table(Table::new());
        }

        add_array_str(&mut item[group], dependency);

        self
    }

    pub fn remove_project_optional_dependency(
        &mut self,
        dependency: &str,
        group: &str,
    ) -> &mut Self {
        let item = &mut self.doc["project"]["optional-dependencies"][group];

        remove_array_dependency(item, dependency);

        self
    }

    #[must_use]
    pub fn contains_project_optional_dependency_any(&self, dependency: &str) -> bool {
        let Some(keys) = self.project_optional_dependency_groups() else {
            return false;
        };

        for key in keys {
            if self.contains_project_optional_dependency(dependency, &key) {
                return true;
            }
        }

        false
    }

    #[must_use]
    pub fn contains_project_optional_dependency(&self, dependency: &str, group: &str) -> bool {
        // TODO(cnpryer): Perf
        self.project_optional_dependencies().map_or(false, |it| {
            it.get(&group.to_string()).map_or(false, |g| {
                g.iter().any(|s| matches_dependency(s, dependency))
            })
        })
    }
}

/// Read and return a `PyProjectToml` from a pyproject.toml file.
fn read_pyproject_toml<T: AsRef<Path>>(path: T) -> Result<PyProjectToml, Error> {
    PyProjectToml::from_str(&std::fs::read_to_string(path)?)
}

#[allow(dead_code)]
fn format_pyproject_toml(pyproject_toml: &mut PyProjectToml) -> &mut PyProjectToml {
    // Format the dependencies
    pyproject_toml.project_dependencies_mut().map(format_array);
    pyproject_toml
        .project_optional_dependencies_mut()
        .map(format_table);

    pyproject_toml
}

/// Save the `PyProjectToml` to a filepath.
fn write_pyproject_toml<T: AsRef<Path>>(toml: &PyProjectToml, path: T) -> Result<(), Error> {
    Ok(std::fs::write(path, toml.to_string())?)
}

// TODO(cnpryer): If contains requirement
fn add_array_str(item: &mut Item, s: &str) {
    if item.is_none() {
        *item = Item::Value(Value::Array(Array::new()));
    }

    // Replace the entry if it exists
    if let Some(index) = item.as_array().and_then(|it| {
        it.iter()
            .position(|v| v.as_str().map_or(false, |x| matches_dependency(x, s)))
    }) {
        if let Some(array) = item.as_array_mut() {
            array.replace(index, s);
        }
    } else {
        item.as_array_mut().get_or_insert(&mut Array::new()).push(s);
    }
}

fn remove_array_dependency(item: &mut Item, dependency: &str) {
    if let Some(array) = item.as_array_mut() {
        array.retain(|it| {
            it.as_str()
                .map_or(false, |s| !matches_dependency(s, dependency))
        });

        if let Some(it) = array.get_mut(0) {
            let Some(s) = it.as_str() else {
                return;
            };
            *it = Value::String(Formatted::new(s.trim_start().to_string()));
        }
    }
}

fn matches_dependency(s: &str, dependency: &str) -> bool {
    let Ok(req) = Requirement::from_str(dependency) else {
        return false;
    };

    Requirement::from_str(s).map_or(false, |it| it.name == req.name)
}

impl FromStr for PyProjectToml {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PyProjectToml {
            doc: Document::from_str(s)?,
        })
    }
}

impl AsMut<PyProjectToml> for PyProjectToml {
    fn as_mut(&mut self) -> &mut PyProjectToml {
        self
    }
}

impl Display for PyProjectToml {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.doc)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use tempfile::TempDir;
    use toml_edit::{Formatted, Value};

    use super::*;

    #[test]
    fn test_get_core() {
        let pyproject_toml = PyProjectToml::from_str(mock_pyproject_toml_content()).unwrap();
        let name = pyproject_toml.project_name().unwrap();
        let version = pyproject_toml.project_version().unwrap();
        let dependencies = pyproject_toml
            .project_dependencies()
            .map(|it| it.into_iter().collect::<Vec<String>>());
        let optional_dependencies = pyproject_toml.project_optional_dependencies();

        assert_eq!(name, "huak".to_string());
        assert_eq!(version, "0.0.20a1".to_string());
        assert!(dependencies.is_some());
        assert!(pyproject_toml.contains_project_dependency("test"));
        assert!(optional_dependencies.is_none());
        assert!(!pyproject_toml.contains_project_optional_dependency("test", "test"));
    }

    #[test]
    fn test_get_tool() {
        let pyproject_toml = PyProjectToml::from_str(mock_pyproject_toml_content()).unwrap();
        let tool = pyproject_toml.get("tool");
        let maturin = tool.as_ref().and_then(|it| it.get("maturin"));
        let maturin_table = maturin
            .and_then(Item::as_table)
            .map(ToString::to_string)
            .unwrap();

        assert_eq!(
            maturin_table,
            r#"bindings = "bin"
manifest-path = "crates/huak-cli/Cargo.toml"
module-name = "huak"
python-source = "python"
strip = true
"#
            .to_string()
        );
    }

    #[test]
    fn test_get_huak() {
        let pyproject_toml = PyProjectToml::from_str(mock_pyproject_toml_content()).unwrap();
        let toolchain = pyproject_toml
            .get("tool")
            .and_then(|it| it.get("huak"))
            .and_then(Item::as_table)
            .and_then(|it| it.get("toolchain"))
            .map(ToString::to_string)
            .unwrap();

        assert_eq!(toolchain, " \"default\"".to_string());
    }

    #[test]
    fn test_read_file() {
        let dir = TempDir::new().unwrap();
        let dir = dir.path();
        let workspace = dir.join("workspace");

        std::fs::create_dir_all(&workspace).unwrap();

        std::fs::write(
            workspace.join("pyproject.toml"),
            mock_pyproject_toml_content(),
        )
        .unwrap();

        let pyproject_toml = PyProjectToml::read_toml(workspace.join("pyproject.toml")).unwrap();

        assert_eq!(&pyproject_toml.to_string(), mock_pyproject_toml_content());
    }

    #[test]
    fn test_write_file() {
        let dir = TempDir::new().unwrap();
        let dir = dir.path();
        let workspace = dir.join("workspace");

        std::fs::create_dir_all(&workspace).unwrap();

        let content = mock_pyproject_toml_content();

        let pyproject_toml = PyProjectToml::from_str(content).unwrap();
        pyproject_toml
            .write_toml(workspace.join("pyproject.toml"))
            .unwrap();

        let pyproject_toml = PyProjectToml::read_toml(workspace.join("pyproject.toml")).unwrap();

        assert_eq!(&pyproject_toml.to_string(), content);
    }

    #[test]
    fn test_update_core_section() {
        let dir = TempDir::new().unwrap();
        let dir = dir.path();
        let workspace = dir.join("workspace");

        std::fs::create_dir_all(&workspace).unwrap();

        let content = mock_pyproject_toml_content();

        let mut pyproject_toml = PyProjectToml::from_str(content).unwrap();

        pyproject_toml
            .set_project_name("new name")
            .add_project_dependency("test")
            .add_project_dependency("new")
            .remove_project_dependency("test")
            .add_project_optional_dependency("test", "test")
            .add_project_optional_dependency("new", "test")
            .remove_project_optional_dependency("test", "test")
            .formatted()
            .write_toml(workspace.join("pyproject.toml"))
            .unwrap();
        let pyproject_toml = PyProjectToml::read_toml(workspace.join("pyproject.toml")).unwrap();
        let optional_deps = pyproject_toml.project_optional_dependencies().unwrap();

        assert_eq!(
            pyproject_toml
                .get("project")
                .and_then(|it| it.get("name"))
                .and_then(Item::as_value)
                .map(ToString::to_string)
                .unwrap(),
            " \"new name\"".to_string()
        );

        assert_eq!(optional_deps.get("test").unwrap(), &vec!["new".to_string()]);

        assert_eq!(
            pyproject_toml.to_string(),
            r#"[build-system]
requires = ["maturin>=0.14,<0.15"]
build-backend = "maturin"

[project]
name = "new name"
version = "0.0.20a1"
description = "A Python package manager written in Rust and inspired by Cargo."
authors = [
    {email = "cnpryer@gmail.com"},
    {name = "Chris Pryer"}
]
readme = "README.md"
license = {text = "MIT"}
requires-python = ">=3.7"
classifiers = [
    "Programming Language :: Rust",
]
dependencies = [
    "new",
] # Trailing comment

[project.urls]
issues = "https://github.com/cnpryer/huak/issues"
documentation = "https://github.com/cnpryer/huak"
homepage = "https://github.com/cnpryer/huak"
repository = "https://github.com/cnpryer/huak"

[project.optional-dependencies]
test = [
    "new",
]

[tool.maturin]
bindings = "bin"
manifest-path = "crates/huak-cli/Cargo.toml"
module-name = "huak"
python-source = "python"
strip = true

[tool.huak]
toolchain = "default"

[tool.huak.run]
hello-world = "python -c 'print(\"hello, world.\")'"

[tool.huak.workspace]
members = ["projects/*"]
"#
        );
    }

    #[test]
    fn test_update_tool_section() {
        let dir = TempDir::new().unwrap();
        let dir = dir.path();
        let workspace = dir.join("workspace");

        std::fs::create_dir_all(&workspace).unwrap();

        let content = mock_pyproject_toml_content();

        let mut pyproject_toml = PyProjectToml::from_str(content).unwrap();
        let tool = pyproject_toml.tool_table_mut().unwrap();
        let maturin = tool.get_mut("maturin").unwrap().as_table_mut().unwrap();
        maturin.insert(
            "module-name",
            Item::Value(Value::String(Formatted::new("new name".to_string()))),
        );

        pyproject_toml
            .formatted()
            .write_toml(workspace.join("pyproject.toml"))
            .unwrap();
        let pyproject_toml = PyProjectToml::read_toml(workspace.join("pyproject.toml")).unwrap();

        assert_eq!(
            pyproject_toml
                .get("tool")
                .and_then(|tool| tool.get("maturin"))
                .and_then(|maturin| maturin.get("module-name"))
                .and_then(|name| name.as_value())
                .map(ToString::to_string)
                .unwrap(),
            " \"new name\"".to_string()
        );

        assert_eq!(
            pyproject_toml.to_string(),
            r#"[build-system]
requires = ["maturin>=0.14,<0.15"]
build-backend = "maturin"

[project]
name = "huak"
version = "0.0.20a1"
description = "A Python package manager written in Rust and inspired by Cargo."
authors = [
    {email = "cnpryer@gmail.com"},
    {name = "Chris Pryer"}
]
readme = "README.md"
license = {text = "MIT"}
requires-python = ">=3.7"
classifiers = [
    "Programming Language :: Rust",
]
dependencies = [
    "test",
] # Trailing comment

[project.urls]
issues = "https://github.com/cnpryer/huak/issues"
documentation = "https://github.com/cnpryer/huak"
homepage = "https://github.com/cnpryer/huak"
repository = "https://github.com/cnpryer/huak"

[tool.maturin]
bindings = "bin"
manifest-path = "crates/huak-cli/Cargo.toml"
module-name = "new name"
python-source = "python"
strip = true

[tool.huak]
toolchain = "default"

[tool.huak.run]
hello-world = "python -c 'print(\"hello, world.\")'"

[tool.huak.workspace]
members = ["projects/*"]
"#
        );
    }

    #[test]
    fn test_update_huak_section() {
        let dir = TempDir::new().unwrap();
        let dir = dir.path();
        let workspace = dir.join("workspace");

        std::fs::create_dir_all(&workspace).unwrap();

        let content = mock_pyproject_toml_content();

        let mut pyproject_toml = PyProjectToml::from_str(content).unwrap();
        let tool = pyproject_toml.tool_table_mut().unwrap();
        let huak = tool.get_mut("huak").unwrap().as_table_mut().unwrap();
        huak.insert(
            "toolchain",
            Item::Value(Value::String(Formatted::new("3.11".to_string()))),
        );

        pyproject_toml
            .formatted()
            .write_toml(workspace.join("pyproject.toml"))
            .unwrap();
        let pyproject_toml = PyProjectToml::read_toml(workspace.join("pyproject.toml")).unwrap();

        assert_eq!(
            pyproject_toml
                .get("tool")
                .and_then(|it| it.get("huak"))
                .and_then(|it| it.get("toolchain"))
                .and_then(Item::as_value)
                .map(ToString::to_string)
                .unwrap(),
            " \"3.11\"".to_string()
        );
    }

    fn mock_pyproject_toml_content() -> &'static str {
        r#"[build-system]
requires = ["maturin>=0.14,<0.15"]
build-backend = "maturin"

[project]
name = "huak"
version = "0.0.20a1"
description = "A Python package manager written in Rust and inspired by Cargo."
authors = [
    {email = "cnpryer@gmail.com"},
    {name = "Chris Pryer"}
]
readme = "README.md"
license = {text = "MIT"}
requires-python = ">=3.7"
classifiers = [
    "Programming Language :: Rust",
]
dependencies = ["test"] # Trailing comment

[project.urls]
issues = "https://github.com/cnpryer/huak/issues"
documentation = "https://github.com/cnpryer/huak"
homepage = "https://github.com/cnpryer/huak"
repository = "https://github.com/cnpryer/huak"

[tool.maturin]
bindings = "bin"
manifest-path = "crates/huak-cli/Cargo.toml"
module-name = "huak"
python-source = "python"
strip = true

[tool.huak]
toolchain = "default"

[tool.huak.run]
hello-world = "python -c 'print(\"hello, world.\")'"

[tool.huak.workspace]
members = ["projects/*"]
"#
    }
}
