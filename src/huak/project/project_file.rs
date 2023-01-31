use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::config::pyproject::toml::Toml;

use crate::errors::{HuakError, HuakResult};
use crate::package::PythonPackage;
use crate::utils;

const DEFAULT_SEARCH_STEPS: usize = 5;

// TODO: Potentially use `File` (currently not using buf)
#[derive(Default, Clone)]
pub struct ProjectFile {
    pub filepath: Option<PathBuf>,
    pub data: Option<Toml>,
}

impl ProjectFile {
    pub fn from_filepath(path: &Path) -> HuakResult<ProjectFile> {
        let toml = Toml::open(path)?;

        Ok(ProjectFile {
            filepath: Some(path.to_path_buf()),
            data: Some(toml),
        })
    }

    pub fn from_directory(path: &Path) -> HuakResult<ProjectFile> {
        // TODO:
        //   - Allow more than pyproject.toml
        //   - Use .parent or similar path search utilities
        let filepath = utils::path::search_parents_for_filepath(
            path,
            "pyproject.toml",
            DEFAULT_SEARCH_STEPS,
        )?;

        if let Some(it) = filepath {
            Ok(ProjectFile::from_filepath(&it)?)
        } else {
            Ok(ProjectFile::default())
        }
    }

    /// Get a reference to the project name from project file data.
    // TODO: Use more than toml.
    pub fn project_name(&self) -> Option<&str> {
        if let Some(it) = &self.data {
            return Some(&it.project.name);
        }

        None
    }

    pub fn set_project_name(&mut self, name: &str) {
        if let Some(it) = &mut self.data {
            it.project.name = name.to_string();
        }
    }

    /// Get a reference to the project version from project file data.
    pub fn project_version(&self) -> Option<&str> {
        // NOTE: This feels like a messy way to retain ownership.
        if let Some(some_data) = &self.data {
            if let Some(some_path) = &some_data.project.version {
                return Some(some_path);
            }
        }

        None
    }

    pub fn pyproject_path(&self) -> Option<&PathBuf> {
        if let Some(it) = &self.filepath {
            return Some(it);
        }

        None
    }

    pub fn pyproject_toml(&self) -> Option<&Toml> {
        if let Some(it) = &self.data {
            return Some(it);
        }

        None
    }

    pub fn dependency_list(&self) -> Option<&Vec<String>> {
        if let Some(some_data) = &self.data {
            return some_data.project.dependencies.as_ref();
        }

        None
    }

    pub fn optional_dependency_list(
        &self,
        group: &str,
    ) -> Option<&Vec<String>> {
        if let Some(some_data) = &self.data {
            if let Some(some_list) = &some_data.project.optional_dependencies {
                return some_list.get(group);
            }
        }

        None
    }

    pub fn serialize(&self) -> HuakResult<()> {
        if let Some(some_data) = &self.data {
            let string = some_data.to_string()?;
            if let Some(some_path) = &self.filepath {
                fs::write(some_path, string)?;
            } else {
                return Err(HuakError::PyProjectFileNotFound);
            }
        }

        Ok(())
    }

    pub fn add_dependency(&mut self, dependency: &str) -> HuakResult<()> {
        if let Some(some_data) = &mut self.data {
            if let Some(some_list) = &mut some_data.project.dependencies {
                add_to_dependency_list(some_list, dependency)?;
            }
        }

        Ok(())
    }

    pub fn add_optional_dependency(
        &mut self,
        dependency: &str,
        group: &str,
    ) -> HuakResult<()> {
        if let Some(some_data) = &mut self.data {
            if let Some(some_groups) =
                &mut some_data.project.optional_dependencies
            {
                match &mut some_groups.entry(group.to_string()) {
                    std::collections::hash_map::Entry::Occupied(some_entry) => {
                        let list = some_entry.get_mut();
                        add_to_dependency_list(list, dependency)?
                    }
                    std::collections::hash_map::Entry::Vacant(_) => {
                        some_groups.insert(
                            group.to_string(),
                            vec![dependency.to_string()],
                        );
                    }
                }
            }
        }

        Ok(())
    }

    pub fn remove_dependency(
        &mut self,
        dependency: &str,
        group: &Option<String>,
    ) -> HuakResult<()> {
        if let Some(some_data) = &mut self.data {
            match &group {
                Some(some_group) => {
                    if let Some(some_groups) =
                        &mut some_data.project.optional_dependencies
                    {
                        if let Some(some_list) = some_groups.get_mut(some_group)
                        {
                            remove_from_dependency_list(some_list, dependency)?;
                        }
                    }
                }
                None => {
                    if let Some(some_list) = &mut some_data.project.dependencies
                    {
                        remove_from_dependency_list(some_list, dependency)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn search_dependency_list(
        &self,
        package: &PythonPackage,
        group: &Option<String>,
    ) -> HuakResult<Option<&str>> {
        if let Some(some_data) = &self.data {
            // Get list of dependencies to search
            let list = match group {
                // If there's a group, and if it exists, search it. Otherwise there's
                // nothing to search.
                Some(some_group) => {
                    if let Some(some_groups) =
                        &some_data.project.optional_dependencies
                    {
                        match some_groups.get(some_group) {
                            Some(some_list) => some_list,
                            None => return Ok(None),
                        }
                    } else {
                        return Ok(None);
                    }
                }
                // If there's no group and there's dependencies listed, search the
                // listed dependencies. Otherwise there's nothing to search.
                None => {
                    if let Some(some_list) = &some_data.project.dependencies {
                        some_list
                    } else {
                        return Ok(None);
                    }
                }
            };

            // Iterate over the existing list of dependencies and return a match on
            // package name.
            if let Some((i, _)) = list
                .iter()
                .map(|x| PythonPackage::from_str(x))
                .enumerate()
                .find(|x| {
                    if let Ok(some_x) = &x.1 {
                        some_x.name == package.name
                    } else {
                        false
                    }
                })
            {
                return Ok(Some(&list[i]));
            }
        }

        Ok(None)
    }
}

fn add_to_dependency_list(
    list: &mut Vec<String>,
    dependency: &str,
) -> HuakResult<()> {
    let package = PythonPackage::from_str(dependency)?;

    if let Some((i, _)) = list
        .iter_mut()
        .map(|x| PythonPackage::from_str(x))
        .enumerate()
        .find(|x| {
            if let Ok(some_x) = &x.1 {
                some_x.name == package.name
            } else {
                false
            }
        })
    {
        list[i] = dependency.to_string();

        return Ok(());
    }

    list.push(dependency.to_string());

    Ok(())
}

fn remove_from_dependency_list(
    list: &mut Vec<String>,
    dependency: &str,
) -> HuakResult<()> {
    let package = PythonPackage::from_str(dependency)?;

    if let Some((i, _)) = list
        .iter_mut()
        .map(|x| PythonPackage::from_str(x))
        .enumerate()
        .find(|x| {
            if let Ok(some_x) = &x.1 {
                some_x.name == package.name
            } else {
                false
            }
        })
    {
        list.remove(i);

        return Ok(());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn from_filepath() {
        let filepath = tempdir().unwrap().into_path().join("test.toml");
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[project.optional-dependencies]
test = ["pytest>=6", "mock"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;

        // toml_edit does not preserve the ordering of the tables
        let expected_output = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = [
    "click==8.1.3",
    "black==22.8.0",
]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
test = [
    "pytest>=6",
    "mock",
]

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;

        let toml = Toml::from(string).unwrap();
        fs::write(&filepath, toml.to_string().unwrap()).unwrap();
        assert_eq!(
            expected_output,
            ProjectFile::from_filepath(&filepath)
                .unwrap()
                .data
                .unwrap()
                .to_string()
                .unwrap()
        );
    }

    #[test]
    fn serialize() {
        let filepath = tempdir().unwrap().into_path().join("test.toml");
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[project.optional-dependencies]
test = ["pytest>=6", "mock"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;

        // toml_edit does not preserve the ordering of the tables
        let expected_output = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = [
    "click==8.1.3",
    "black==22.8.0",
]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
test = [
    "pytest>=6",
    "mock",
]

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;

        let toml = Toml::from(string).unwrap();
        fs::write(&filepath, toml.to_string().unwrap()).unwrap();
        assert_eq!(
            expected_output,
            Toml::open(&filepath).unwrap().to_string().unwrap()
        );
    }

    #[test]
    fn dependency_list() {
        let filepath = tempdir().unwrap().into_path().join("test.toml");
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();
        fs::write(&filepath, toml.to_string().unwrap()).unwrap();

        let project_file = ProjectFile::from_filepath(&filepath).unwrap();
        let expected_dependencies = vec!["click==8.1.3", "black==22.8.0"];

        assert_eq!(
            project_file.dependency_list().unwrap(),
            &expected_dependencies
        );
    }

    #[test]
    fn optional_dependency_list() {
        let filepath = tempdir().unwrap().into_path().join("test.toml");
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""

[project.optional-dependencies]
test = ["click==8.1.3", "black==22.8.0"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();
        fs::write(&filepath, toml.to_string().unwrap()).unwrap();

        let project_file = ProjectFile::from_filepath(&filepath).unwrap();
        let expected_dependencies = vec!["click==8.1.3", "black==22.8.0"];

        assert_eq!(
            project_file.optional_dependency_list("test").unwrap(),
            &expected_dependencies
        );
    }

    #[test]
    fn search_dependency_list() {
        let filepath = tempdir().unwrap().into_path().join("test.toml");
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["black==22.8.0"]

[project.optional-dependencies]
test = ["click==8.1.3"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();
        fs::write(&filepath, toml.to_string().unwrap()).unwrap();

        let project_file = ProjectFile::from_filepath(&filepath).unwrap();
        let expected_dependency = "black==22.8.0";
        let expected_optional_dependency = "click==8.1.3";

        assert_eq!(
            project_file
                .search_dependency_list(
                    &PythonPackage::from_str("black").unwrap(),
                    &None
                )
                .unwrap(),
            Some(expected_dependency)
        );
        assert_eq!(
            project_file
                .search_dependency_list(
                    &PythonPackage::from_str("click").unwrap(),
                    &Some("test".to_string())
                )
                .unwrap(),
            Some(expected_optional_dependency)
        );
    }

    #[test]
    fn add_dependency() {
        let filepath = tempdir().unwrap().into_path().join("test.toml");
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["black==22.8.0"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();
        fs::write(&filepath, toml.to_string().unwrap()).unwrap();

        let mut project_file = ProjectFile::from_filepath(&filepath).unwrap();
        let original_dependencies =
            project_file.dependency_list().unwrap().clone();
        project_file.add_dependency("package").unwrap();

        assert_ne!(
            &original_dependencies,
            project_file.dependency_list().unwrap()
        );
        assert_eq!(
            project_file.dependency_list().unwrap().last().unwrap(),
            "package"
        );
    }

    #[test]
    fn add_optional_dependency() {
        let filepath = tempdir().unwrap().into_path().join("test.toml");
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""

[project.optional-dependencies]
test = ["black==22.8.0"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();
        fs::write(&filepath, toml.to_string().unwrap()).unwrap();

        let mut project_file = ProjectFile::from_filepath(&filepath).unwrap();
        let original_dependencies = project_file
            .optional_dependency_list("test")
            .unwrap()
            .clone();
        project_file
            .add_optional_dependency("package", "test")
            .unwrap();

        assert_ne!(
            &original_dependencies,
            project_file.optional_dependency_list("test").unwrap()
        );
        assert_eq!(
            project_file
                .optional_dependency_list("test")
                .unwrap()
                .last()
                .unwrap(),
            "package"
        );
    }

    #[test]
    fn remove_dependency() {
        let filepath = tempdir().unwrap().into_path().join("test.toml");
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["black==22.8.0"]

[project.optional-dependencies]
test = ["test"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();
        fs::write(&filepath, toml.to_string().unwrap()).unwrap();

        let mut project_file = ProjectFile::from_filepath(&filepath).unwrap();
        let original_dependencies =
            project_file.dependency_list().unwrap().clone();
        let original_optional_dependencies = project_file
            .optional_dependency_list("test")
            .unwrap()
            .clone();
        project_file.remove_dependency("black", &None).unwrap();
        project_file
            .remove_dependency("test", &Some("test".to_string()))
            .unwrap();

        assert!(!original_dependencies.is_empty());
        assert!(!original_optional_dependencies.is_empty());

        assert!(project_file.dependency_list().unwrap().is_empty());
        assert!(project_file
            .optional_dependency_list("test")
            .unwrap()
            .is_empty());
    }
}
