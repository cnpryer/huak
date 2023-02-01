use std::str::FromStr;

use crate::{
    env::{runner::Runner, venv::Venv},
    errors::HuakResult,
    package::{installer::Installer, PythonPackage},
    project::Project,
};

const MODULE: &str = "ruff";

/// Fixes the lint error the project from its root.
pub fn fix_project(
    project: &Project,
    py_env: &Venv,
    installer: &Installer,
) -> HuakResult<()> {
    if !py_env.module_path(MODULE)?.exists() {
        let package = PythonPackage::from_str(MODULE)?;
        installer.install_package(&package, py_env)?;
    }

    let args = [".", "--fix", "--extend-exclude", py_env.name()?];
    let runner = Runner::new()?;
    runner.run_installed_module(MODULE, &args, py_env, Some(project.root()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    use crate::utils::test_utils::create_mock_project_full;

    #[test]
    fn fix() {
        let project = create_mock_project_full().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let venv = &Venv::new(cwd.join(".venv"));
        let runner = Runner::new().unwrap();
        let installer = Installer::new();

        runner
            .run_installed_module(
                "pip",
                &["install", MODULE],
                venv,
                Some(project.root()),
            )
            .unwrap();

        let lint_fix_filepath =
            project.root().join("mock_project").join("fix_me.py");
        let pre_fix_str = r#"
import json # this gets removed(autofixed)


def fn():
    pass
"#;
        let expected = r#"


def fn():
    pass
"#;

        fs::write(&lint_fix_filepath, pre_fix_str).unwrap();
        fix_project(&project, &venv, &installer).unwrap();
        let post_fix_str = fs::read_to_string(&lint_fix_filepath).unwrap();

        assert_eq!(post_fix_str, expected);
    }
}
