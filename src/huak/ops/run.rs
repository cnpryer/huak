use crate::{
    env::{python_environment::Venv, runner::Runner},
    errors::HuakResult,
    project::Project,
};

pub fn run_command(
    command: &[String],
    project: &Project,
    py_env: &Venv,
) -> HuakResult<()> {
    // TODO: Might make sense to add runner as a parameter for this operation
    let runner = Runner::new()?;
    runner.run_str_command(&command.join(" "), py_env, Some(project.root()))
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use std::str::FromStr;

    use super::*;
    use crate::env::python_environment::PythonEnvironment;
    use crate::package::installer::Installer;
    use crate::package::PythonPackage;
    use crate::utils::test_utils::create_mock_project_full;

    #[test]
    fn run() {
        let project = create_mock_project_full().unwrap();
        let cwd = current_dir().unwrap();
        let py_env = Venv::from_directory(&cwd).unwrap();
        let installer = Installer::new();
        let test_package = PythonPackage::from_str("xlcsv").unwrap();
        installer.install_package(&test_package, &py_env).unwrap();
        let existed = py_env.module_path("xlcsv").unwrap().exists();

        let command = "pip uninstall xlcsv -y"
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        run_command(&command, &project, &py_env).unwrap();

        assert!(existed);
        assert!(!py_env.module_path("xlcsv").unwrap().exists());
    }
}
