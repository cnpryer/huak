use crate::{errors::HuakResult, project::Project};

pub fn activate_project_venv(project: &Project) -> HuakResult<()> {
    let venv = project
        .venv()
        .as_ref()
        .expect("`Project::from` creates venv if it doesn't exists.");

    println!("Venv activated: {}", venv.path.display());

    venv.activate()?;

    println!("Venv deactivated.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    use crate::{
        env::venv::HUAK_VENV_ENV_VAR,
        utils::{
            path::copy_dir,
            test_utils::{create_mock_project, get_resource_dir},
        },
    };

    #[test]
    fn venv_can_be_activated() {
        let directory = tempdir().unwrap().into_path();
        let mock_project_path = get_resource_dir().join("mock-project");
        copy_dir(&mock_project_path, &directory);

        let project =
            create_mock_project(directory.join("mock-project")).unwrap();

        assert!(std::env::var(HUAK_VENV_ENV_VAR).is_err());

        let result = activate_project_venv(&project);
        assert!(result.is_ok());
    }

    #[test]
    fn venv_cant_be_activated() {
        let directory = tempdir().unwrap().into_path();
        let mock_project_path = get_resource_dir().join("mock-project");
        copy_dir(&mock_project_path, &directory);

        let project =
            create_mock_project(directory.join("mock-project")).unwrap();

        std::env::set_var(HUAK_VENV_ENV_VAR, "1");
        assert!(std::env::var(HUAK_VENV_ENV_VAR).is_ok());

        let result = activate_project_venv(&project);
        assert!(result.is_err());

        std::env::remove_var(HUAK_VENV_ENV_VAR);
    }
}
