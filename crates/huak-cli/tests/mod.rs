#[cfg(test)]
mod tests {
    use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
    use std::{path::PathBuf, process::Command};

    #[test]
    fn test_activate_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("activate").arg("--help"));
    }

    #[test]
    fn test_add_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("add").arg("--help"));
    }

    #[test]
    fn test_build_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("build").arg("--help"));
    }

    #[test]
    fn test_clean_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("clean").arg("--help"));
    }

    #[test]
    fn test_completion_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("completion").arg("--help"));
    }

    #[test]
    fn test_fix_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("fix").arg("--help"));
    }

    #[test]
    fn test_fmt_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("fmt").arg("--help"));
    }

    #[test]
    fn test_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("help"));
        assert_cmd_snapshot!(Command::new(bin()).arg("--help"));
    }

    #[test]
    fn test_init_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("init").arg("--help"));
    }

    #[test]
    fn test_install_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("install").arg("--help"));
    }

    #[test]
    fn test_lint_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("lint").arg("--help"));
    }

    #[test]
    fn test_new_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("new").arg("--help"));
    }

    #[test]
    fn test_publish_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("publish").arg("--help"));
    }

    #[test]
    fn test_python_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("python").arg("--help"));
    }

    #[test]
    fn test_remove_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("remove").arg("--help"));
    }

    #[test]
    fn test_run_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("run").arg("--help"));
    }

    #[test]
    fn test_test_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("test").arg("--help"));
    }

    #[test]
    fn test_update_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("update").arg("--help"));
    }

    #[test]
    fn test_version_help() {
        assert_cmd_snapshot!(Command::new(bin()).arg("version").arg("--help"));
    }

    #[test]
    fn test_version() {
        let from = dev_resources_dir().join("mock-project");
        assert_cmd_snapshot!(Command::new(bin())
            .arg("version")
            .arg("--no-color")
            .current_dir(from));
    }

    /// The resource directory found in the Huak repo used for testing purposes.
    fn dev_resources_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("dev-resources")
    }

    fn bin() -> PathBuf {
        get_cargo_bin("huak")
    }
}
