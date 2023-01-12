use crate::{env::venv::Venv, errors::HuakError, project::Project};

const MODULE: &str = "black";

/// Format Python code from the `Project`'s root.
pub fn fmt_project(
    project: &Project,
    is_check: &bool,
) -> Result<(), HuakError> {
    let venv = &Venv::from_path(project.root())?;

    match is_check {
        true => venv.exec_module(
            MODULE,
            &[".", "--line-length", "79", "--check"],
            project.root(),
        ),
        false => venv.exec_module(
            MODULE,
            &[".", "--line-length", "79"],
            project.root(),
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
        let venv = &Venv::from_path(project.root()).unwrap();

        venv.exec_module("pip", &["install", MODULE], project.root())
            .unwrap();

        let fmt_filepath =
            project.root().join("mock_project").join("fmt_me.py");
        let pre_fmt_str = r#"""
def fn( ):
    pass"#;
        fs::write(&fmt_filepath, pre_fmt_str).unwrap();
        fmt_project(&project, &false).unwrap();
        let post_fmt_str = fs::read_to_string(&fmt_filepath).unwrap();

        assert_ne!(pre_fmt_str, post_fmt_str);
    }
}
