use crate::Error;
use std::path::Path;

pub(crate) fn name_from_path(path: &Path) -> Result<&str, Error> {
    let Some(name) = path
        .components()
        .last()
        .map(std::path::Component::as_os_str)
        .and_then(|name| name.to_str())
    else {
        return Err(Error::InvalidToolchain(
            "could not parse name from path".to_string(),
        ));
    };

    Ok(name)
}
