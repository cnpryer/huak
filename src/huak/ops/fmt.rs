use std::str::FromStr;

use crate::{
    env::{runner::Runner, venv::Venv},
    errors::HuakResult,
    package::{installer::Installer, PythonPackage},
    project::Project,
};

const MODULE: &str = "black";

/// Format Python code from the `Project`'s root.
pub fn fmt_project(
    project: &Project,
    py_env: &Venv,
    installer: &Installer,
    is_check: &bool,
) -> HuakResult<()> {
    if !py_env.module_path(MODULE)?.exists() {
        let package = PythonPackage::from_str(MODULE)?;
        installer.install_package(&package, py_env)?;
    }

    let runner = Runner::new()?;
    match is_check {
        true => runner.run_installed_module(
            MODULE,
            &[".", "--line-length", "79", "--check"],
            py_env,
            Some(project.root()),
        ),
        false => runner.run_installed_module(
            MODULE,
            &[".", "--line-length", "79"],
            py_env,
            Some(project.root()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::utils::test_utils::create_mock_project_full;

    use super::*;

    #[test]
    fn fmt() {
        let project = create_mock_project_full().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let venv = Venv::new(cwd.join(".venv"));
        let runner = Runner::new().unwrap();
        let installer = Installer::new();

        runner
            .run_installed_module(
                "pip",
                &["install", MODULE],
                &venv,
                Some(project.root()),
            )
            .unwrap();

        let fmt_filepath =
            project.root().join("mock_project").join("fmt_me.py");
        let pre_fmt_str = r#"""
def fn( ):
    pass"#;
        fs::write(&fmt_filepath, pre_fmt_str).unwrap();
        fmt_project(&project, &venv, &installer, &false).unwrap();
        let post_fmt_str = fs::read_to_string(&fmt_filepath).unwrap();

        assert_ne!(pre_fmt_str, post_fmt_str);
    }
}
