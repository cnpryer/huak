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
        if let Some(it_data) = &self.data {
            if let Some(it_path) = &it_data.project.version {
                return Some(it_path);
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

    pub fn dependency_list(&self) -> Vec<PythonPackage> {
        if let Some(it_data) = &self.data {
            if let Some(it_list) = it_data.project.dependencies.as_ref() {
                return it_list
                    .iter()
                    .filter_map(|d| PythonPackage::from_str(d).ok())
                    .collect();
            }
        }

        vec![]
    }

    // TODO: Check if "group" used it's always "optional"
    pub fn optional_package_list(&self, group: &str) -> Vec<PythonPackage> {
        // TODO: Look into Option handling with method chaining or some mapping pattern
        //       without using unwrap
        if let Some(it_data) = &self.data {
            if let Some(it_list) = &it_data.project.optional_dependencies {
                if let Some(it_group) = it_list.get(group) {
                    return it_group
                        .iter()
                        .filter_map(|d| PythonPackage::from_str(d).ok())
                        .collect();
                }
            }
        }

        vec![]
    }

    pub fn serialize(&self) -> HuakResult<()> {
        if let Some(it_data) = &self.data {
            let string = it_data.to_string()?;
            if let Some(it_path) = &self.filepath {
                fs::write(it_path, string)?;
            } else {
                return Err(HuakError::PyProjectFileNotFound);
            }
        }
        Ok(())
    }

    pub fn add_dependency(&mut self, dependency: &str) -> HuakResult<()> {
        if let Some(it_data) = &mut self.data {
            if let Some(it_list) = &mut it_data.project.dependencies {
                add_to_dependency_list(it_list, dependency)?;
            }
        }

        Ok(())
    }

    pub fn add_optional_dependency(
        &mut self,
        dependency: &str,
        group: &str,
    ) -> HuakResult<()> {
        if let Some(it_data) = &mut self.data {
            if let Some(it_groups) = &mut it_data.project.optional_dependencies
            {
                match &mut it_groups.entry(group.to_string()) {
                    std::collections::hash_map::Entry::Occupied(it_entry) => {
                        let list = it_entry.get_mut();
                        add_to_dependency_list(list, dependency)?
                    }
                    std::collections::hash_map::Entry::Vacant(_) => {
                        it_groups.insert(
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
        group: Option<String>,
    ) -> HuakResult<()> {
        if let Some(it_data) = &mut self.data {
            match &group {
                Some(it_group) => {
                    if let Some(it_groups) =
                        &mut it_data.project.optional_dependencies
                    {
                        if let Some(it_list) = it_groups.get_mut(it_group) {
                            remove_from_dependency_list(it_list, dependency)?;
                        }
                    }
                }
                None => {
                    if let Some(it_list) = &mut it_data.project.dependencies {
                        remove_from_dependency_list(it_list, dependency)?;
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
        if let Some(it_data) = &self.data {
            let list;

            // TODO: Clean up.
            match group {
                // If there's a group, and if it exists, search it.
                // Otherwise there's nothing to search.
                Some(it_group) => {
                    if let Some(it_groups) =
                        &it_data.project.optional_dependencies
                    {
                        match it_groups.get(it_group) {
                            Some(it_list) => {
                                list = it_list;
                            }
                            None => return Ok(None),
                        }
                    } else {
                        return Ok(None);
                    }
                }
                // If there's no group and there's dependencies listed, search the
                // listed dependencies. Otherwise there's nothing to search.
                None => {
                    if let Some(it_list) = &it_data.project.dependencies {
                        list = it_list;
                    } else {
                        return Ok(None);
                    }
                }
            }

            // Iterate over the existing list of dependencies and return a match on
            // package name.
            if let Some((i, _)) = list
                .iter()
                .map(|x| PythonPackage::from_str(x))
                .enumerate()
                .find(|x| {
                    if let Ok(it_x) = &x.1 {
                        it_x.name == package.name
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
            if let Ok(it_x) = &x.1 {
                it_x.name == package.name
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
            if let Ok(it_x) = &x.1 {
                it_x.name == package.name
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
