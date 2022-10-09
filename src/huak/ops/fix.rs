use crate::{
    errors::{HuakError, HuakResult},
    project::Project,
};

const MODULE: &str = "ruff";

/// Fixes the lint error the project from its root.
pub fn fix_project(project: &Project) -> HuakResult<()> {
    let venv = match project.venv() {
        Some(v) => v,
        _ => return Err(HuakError::VenvNotFound),
    };
    let args = [".", "--fix", "--extend-exclude", venv.name()?];

    venv.exec_module(MODULE, &args, &project.root)
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
    fn fix() {
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

        let lint_fix_filepath =
            project.root.join("mock_project").join("fix_me.py");
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
        fix_project(&project).unwrap();
        let post_fix_str = fs::read_to_string(&lint_fix_filepath).unwrap();

        assert_eq!(post_fix_str, expected);
    }
}
