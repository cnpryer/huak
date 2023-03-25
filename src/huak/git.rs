use crate::{error::HuakResult, Error};
use git2::Repository;
use std::path::Path;

/// Initialize a directory on a local system as a git repository.
pub fn init(dir_path: impl AsRef<Path>) -> HuakResult<()> {
    Repository::init(dir_path).map_err(|e| Error::GitError(e))?;
    Ok(())
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
