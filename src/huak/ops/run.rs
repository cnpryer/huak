use crate::{env::venv::Venv, errors::HuakResult};

pub fn run_command(
    python_envrionment: &Venv,
    command: &[String],
) -> HuakResult<()> {
    python_envrionment.exec_command(&command.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::install::install_project_dependencies;
    use crate::package::installer::Installer;
    use crate::utils::test_utils::create_mock_project_full;

    #[ignore = "currently untestable"]
    #[test]
    fn run() {
        let project = create_mock_project_full().unwrap();
        let venv = Venv::from_directory(project.root()).unwrap();
        let installer = Installer::new();

        install_project_dependencies(&project, &venv, &installer, &None)
            .unwrap();

        let command = "pip list --format=freeze > test_req.txt"
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        run_command(&venv, &command).unwrap();

        let data = std::fs::read_to_string("test_req.txt").unwrap();
        assert!(data.contains("black"));
        assert!(data.contains("click"));
        assert!(data.contains("pytest"));

        std::fs::remove_file("test_req.txt").unwrap();
    }
}
