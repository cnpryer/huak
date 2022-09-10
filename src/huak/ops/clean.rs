use std::fs::remove_dir_all;

use crate::{
    errors::{CliError, CliResult},
    project::Project,
};

pub fn clean_project(project: &Project) -> CliResult {
    if !project.root.join("dist").is_dir() {
        Ok(())
    } else {
        match remove_dir_all("dist") {
            Ok(_) => Ok(()),
            Err(e) => Err(CliError::new(anyhow::format_err!(e), 2)),
        }
    }
}
