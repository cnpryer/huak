use crate::error::HuakResult;
use std::path::Path;

/// Initialize a directory on a local system as a git repository.
pub fn init(dir_path: impl AsRef<Path>) -> HuakResult<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        todo!()
    }
}
