use std::path::Path;

use anyhow::{self, Error};

/// Return the filename from a `Path`.
pub fn parse_filename(path: &Path) -> Result<&str, Error> {
    // Attempt to convert OsStr to str.
    let name = match path.file_name() {
        Some(f) => f.to_str(),
        _ => return Err(anyhow::format_err!("failed to read name from path")),
    };

    // If a str was failed to be parsed error.
    if name.is_none() {
        return Err(anyhow::format_err!(
            "failed to convert filename from {} to string",
            path.display()
        ));
    }

    Ok(name.unwrap())
}

/// Convert a `Path` to a &str.
pub fn to_string(path: &Path) -> Result<&str, anyhow::Error> {
    let pip_path = match path.to_str() {
        Some(s) => s,
        None => {
            return Err(anyhow::format_err!(
                "failed to convert {} to a string",
                path.display()
            ))
        }
    };

    Ok(pip_path)
}
