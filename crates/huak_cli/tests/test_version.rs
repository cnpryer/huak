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
    use std::process::Command;

    use insta_cmd::assert_cmd_snapshot;
    #[test]
    fn test_version() {
        let from = test_resources_dir_path().join("mock-project");
        assert_cmd_snapshot!(Command::new("huak")
            .arg("version")
            .current_dir(from))
    }
}
