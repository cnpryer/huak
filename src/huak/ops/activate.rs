use crate::{
    env::python_environment::{Activatable, Venv},
    errors::HuakResult,
};

pub fn activate_venv(py_env: &Venv) -> HuakResult<()> {
    py_env.activate()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use super::*;

    use crate::env::python_environment::{env_var, PythonEnvironment};

    #[ignore = "currently untestable"]
    #[test]
    // TODO
    fn venv_can_be_activated() {
        let cwd = current_dir().unwrap();
        let venv = &Venv::new(&cwd.join(".venv"));
        venv.create().unwrap();

        assert!(std::env::var(env_var()).is_err());

        let result = activate_venv(&venv);
        assert!(result.is_ok());
    }
}
