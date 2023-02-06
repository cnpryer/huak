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

    use super::*;
    use crate::ops::install::install_project_dependencies;
    use crate::package::installer::Installer;
    use crate::utils::test_utils::create_mock_project_full;

    #[ignore = "unfinished"]
    #[test]
    fn run() {
        let project = create_mock_project_full().unwrap();
        let cwd = current_dir().unwrap();
        let venv = Venv::from_directory(&cwd).unwrap();
        let installer = Installer::new();

        install_project_dependencies(&project, &venv, &installer, &None)
            .unwrap();

        let command = "pip list --format=freeze > test_req.txt"
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        run_command(&command, &project, &venv).unwrap();

        let data = std::fs::read_to_string(project.root().join("test_req.txt"))
            .unwrap();
        assert!(data.contains("black"));
        assert!(data.contains("click"));
        assert!(data.contains("pytest"));

        std::fs::remove_file(project.root().join("test_req.txt")).unwrap();
    }
}
