use crate::{env::venv::Venv, errors::HuakResult};

pub fn activate_venv(py_env: &Venv) -> HuakResult<()> {
    py_env.activate()?;

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
        let venv = &Venv::from_directory(project.root()).unwrap();

        assert!(std::env::var(HUAK_VENV_ENV_VAR).is_err());

        let result = activate_venv(&venv);
        assert!(result.is_ok());
    }
}
