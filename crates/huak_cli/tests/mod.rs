use std::path::PathBuf;

/// The resource directory found in the Huak repo used for testing purposes.
fn test_resources_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("dev-resources")
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta_cmd::assert_cmd_snapshot;
    use std::process::Command;

    #[test]
    fn test_activate_help() {
        assert_cmd_snapshot!(Command::new("huak")
            .arg("activate")
            .arg("--help"));
    }

    #[test]
    fn test_add_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("add").arg("--help"));
    }

    #[test]
    fn test_build_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("build").arg("--help"));
    }

    #[test]
    fn test_clean_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("clean").arg("--help"));
    }

    #[test]
    fn test_completion_help() {
        assert_cmd_snapshot!(Command::new("huak")
            .arg("completion")
            .arg("--help"));
    }

    #[test]
    fn test_fix_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("fix").arg("--help"));
    }

    #[test]
    fn test_fmt_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("fmt").arg("--help"));
    }

    #[test]
    fn test_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("help"));
        assert_cmd_snapshot!(Command::new("huak").arg("--help"));
    }

    #[test]
    fn test_init_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("init").arg("--help"));
    }

    #[test]
    fn test_install_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("install").arg("--help"));
    }

    #[test]
    fn test_lint_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("lint").arg("--help"));
    }

    #[test]
    fn test_new_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("new").arg("--help"));
    }

    #[test]
    fn test_publish_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("publish").arg("--help"));
    }

    #[test]
    fn test_python_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("python").arg("--help"));
    }

    #[test]
    fn test_remove_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("remove").arg("--help"));
    }

    #[test]
    fn test_run_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("run").arg("--help"));
    }

    #[test]
    fn test_test_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("test").arg("--help"));
    }

    #[test]
    fn test_update_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("update").arg("--help"));
    }

    #[test]
    fn test_version_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("version").arg("--help"));
    }

    #[test]
    fn test_version() {
        let from = test_resources_dir_path().join("mock-project");
        assert_cmd_snapshot!(Command::new("huak")
            .arg("version")
            .arg("--no-color")
            .current_dir(from))
    }
}
