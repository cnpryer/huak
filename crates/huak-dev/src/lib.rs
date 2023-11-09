use std::{path::PathBuf, process::Command};

#[must_use]
pub fn dev_resources_dir() -> PathBuf {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .expect("command output");

    if output.status.success() {
        return PathBuf::from(
            String::from_utf8(output.stdout)
                .expect("valid utf-8")
                .trim(),
        )
        .join("dev-resources");
    }

    panic!("failed to resolve repository root")
}
