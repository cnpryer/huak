use crate::{errors::HuakError, project::Project};

const MODULE: &str = "black";

/// Format Python code from the `Project`'s root.
pub fn fmt_project(
    project: &Project,
    is_check: &bool,
) -> Result<(), HuakError> {
    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(HuakError::VenvNotFound),
    };

    let res = match is_check {
        true => venv.exec_module(
            MODULE,
            &[".", "--line-length", "79", "--check"],
            &project.root,
        ),
        false => venv.exec_module(
            MODULE,
            &[".", "--line-length", "79"],
            &project.root,
        ),
    };

    if let Err(e) = res {
        return Err(HuakError::PyBlackError(Box::new(e)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    use crate::utils::{
        path::copy_dir,
        test_utils::{create_mock_project, get_resource_dir},
    };

    #[test]
    fn fmt() {
        let directory = tempdir().unwrap().into_path().to_path_buf();
        let mock_project_dir = get_resource_dir().join("mock-project");
        copy_dir(&mock_project_dir, &directory);

        let project_path = directory.join("mock-project");
        let project = create_mock_project(project_path.clone()).unwrap();
        project
            .venv()
            .as_ref()
            .unwrap()
            .exec_module("pip", &["install", MODULE], &project.root)
            .unwrap();

        let fmt_filepath = project
            .root
            .join("src")
            .join("mock_project")
            .join("fmt_me.py");
        let pre_fmt_str = r#"""
def fn( ):
    pass"#;
        fs::write(&fmt_filepath, pre_fmt_str).unwrap();
        fmt_project(&project, &false).unwrap();
        let post_fmt_str = fs::read_to_string(&fmt_filepath).unwrap();

        assert_ne!(pre_fmt_str, post_fmt_str);
    }
}
