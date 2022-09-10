use std::path::{Path, PathBuf};

use fs_extra::dir;

/// Copies one directory into another.
pub fn create_mock_project_from_dir(from: &PathBuf, to: &PathBuf) -> bool {
    if !Path::new(from).is_file() {
        eprintln!("resource archive does not exist");
    }

    if !Path::new(to).is_dir() {
        eprintln!("`to` {} does not exist", to.display());
    }

    // Copy mock project dir to target dir
    let copy_options = dir::CopyOptions::new();
    dir::copy(from.as_path(), to.as_path(), &copy_options).unwrap();

    true
}
