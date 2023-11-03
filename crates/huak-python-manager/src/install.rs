use crate::{error::Error, releases::Release};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tar::Archive;
use zstd::stream::read::Decoder;

pub fn install_with_target<T: Into<PathBuf>>(release: &Release, target: T) -> Result<(), Error> {
    let buffer = download_release(release)?;
    validate_checksum(&buffer, release.checksum)?;

    // TODO(cnpryer): Support more archive formats.
    let decoded =
        Decoder::with_buffer(buffer.as_slice()).map_err(|e| Error::ZstdError(e.to_string()))?;

    let mut archive = Archive::new(decoded);

    archive
        .unpack(target.into())
        .map_err(|e| Error::TarError(e.to_string()))
}

pub(crate) fn download_release(release: &Release) -> Result<Vec<u8>, Error> {
    let mut response = reqwest::blocking::get(release.url)?;

    if !response.status().is_success() {
        return Err(Error::RequestError(format!(
            "failed to download file from {}",
            release.url
        )));
    }

    let mut contents = Vec::new();
    response.copy_to(&mut contents)?;

    Ok(contents)
}

pub(crate) fn validate_checksum(bytes: &[u8], checksum: &str) -> Result<(), Error> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);

    if hex::encode(hasher.finalize()).eq_ignore_ascii_case(checksum) {
        Ok(())
    } else {
        Err(Error::RequestError(
            "failed to validate checksum".to_string(),
        ))
    }
}
