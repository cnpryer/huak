use std::path::{Path, PathBuf};

use fs_extra::dir;

/// copies the mock-project into temp.
/// this command only works the CURRENT DIRECTORY is unchchanged.
/// resource_archive: PathBuf - directory, relative to resources.
pub fn create_py_project_sample(
    resource_archive: &PathBuf,
    target_directory: &PathBuf,
) -> bool {
    if !Path::new(resource_archive).is_file() {
        eprintln!("resource archive does not exist");
    }

    if !Path::new(target_directory).is_dir() {
        eprintln!(
            "target_directory {} does not exist",
            target_directory.as_os_str().to_str().unwrap()
        );
    }

    // Copy mock project dir to target dir
    let copy_options = dir::CopyOptions::new();
    dir::copy(
        resource_archive.as_path(),
        target_directory.as_path(),
        &copy_options,
    )
    .unwrap();

    true
}
