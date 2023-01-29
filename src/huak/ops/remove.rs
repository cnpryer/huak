use crate::{env::venv::Venv, errors::HuakError, project::Project};

/// Remove a dependency from a project by uninstalling it and updating the
/// project's config.
pub fn remove_project_dependency(
    project: &mut Project,
    venv: &Venv,
    dependency: &str,
    group: Option<String>,
) -> Result<(), HuakError> {
    // TODO: #109
    venv.uninstall_package(dependency)?;

    let project_file = &mut project.project_file;

    project_file.remove_dependency(dependency, group)?;
    project_file.serialize()
}

#[cfg(test)]
mod tests {

    use crate::{
        config::pyproject::toml::Toml,
        utils::test_utils::create_mock_project_full,
    };

    use super::*;

    #[test]
    fn removes_dependencies() {
        // TODO: Optional deps test is passing but the operation wasn't fully
        //       implemented.
        let mut project = create_mock_project_full().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let venv = Venv::new(cwd.join(".venv"));
        let toml_path = project.root().join("pyproject.toml");
        let toml = Toml::open(&toml_path).unwrap();
        let existed = toml
            .project
            .dependencies
            .unwrap()
            .iter()
            .any(|d| d.starts_with("click"));
        let existed = existed
            && toml.project.optional_dependencies.map_or(false, |deps| {
                deps.values().flatten().any(|d| d.starts_with("pytest"))
            });

        remove_project_dependency(&mut project, &venv, "click", None).unwrap();

        let toml = Toml::open(&toml_path).unwrap();
        let exists = toml
            .project
            .dependencies
            .unwrap()
            .iter()
            .any(|s| s.starts_with("click"));

        let exists = exists
            && toml.project.optional_dependencies.map_or(false, |deps| {
                deps.values().flatten().any(|d| d.starts_with("click"))
            });

        assert!(existed);
        assert!(!exists);
    }
}
