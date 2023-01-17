use crate::{env::venv::Venv, errors::HuakResult, project::Project};

pub fn activate_project_venv(project: &Project) -> HuakResult<()> {
    let venv = &Venv::from_path(project.root())?;

    venv.create()?;
    venv.activate()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        env::venv::HUAK_VENV_ENV_VAR,
        utils::test_utils::create_mock_project_full,
    };

    #[ignore = "currently untestable"]
    #[test]
    // TODO
    fn venv_can_be_activated() {
        let project = create_mock_project_full().unwrap();

        assert!(std::env::var(HUAK_VENV_ENV_VAR).is_err());

        let result = activate_project_venv(&project);
        assert!(result.is_ok());
    }

    #[test]
    fn venv_cant_be_activated() {
        let project = create_mock_project_full().unwrap();

        std::env::set_var(HUAK_VENV_ENV_VAR, "1");
        assert!(std::env::var(HUAK_VENV_ENV_VAR).is_ok());

        let result = activate_project_venv(&project);
        assert!(result.is_err());

        std::env::remove_var(HUAK_VENV_ENV_VAR);
    }
}
