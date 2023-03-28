use crate::{error::HuakResult, Error};
use git2::Repository;
use std::path::Path;

/// Initialize a directory on a local system as a git repository
/// and return the Repository.
pub fn init(path: impl AsRef<Path>) -> HuakResult<Repository> {
    Repository::init(path).map_err(Error::GitError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init() {
        let dir = tempdir().unwrap().into_path();
        init(&dir).unwrap();
        assert!(dir.join(".git").is_dir());
    }
}
