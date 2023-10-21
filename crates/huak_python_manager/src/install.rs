use crate::{
    releases::Release,
    resolve::{resolve_release, Strategy},
};
use anyhow::{bail, Context, Error, Ok}; // TODO(cnpryer): Use thiserror in library code.
use huak_home::huak_home_dir;
use sha2::{Digest, Sha256};
use tar::Archive;
use zstd::stream::read::Decoder;

pub(crate) fn install_to_home(strategy: &Strategy) -> Result<(), Error> {
    let release = resolve_release(strategy).context("requested release data")?;
    let target_dir = huak_home_dir()
        .context("requested huak's home directory")?
        .join("toolchains")
        .join(format!("huak-{}-{}", release.kind, release.version));

    let buffer = download_release(&release)?;
    validate_checksum(&buffer, release.checksum)?;

    // TODO(cnpryer): Support more archive formats.
    let decoded = Decoder::with_buffer(buffer.as_slice())?;
    let mut archive = Archive::new(decoded);

    Ok(archive.unpack(target_dir)?)
}

fn download_release(release: &Release) -> Result<Vec<u8>, Error> {
    let mut response = reqwest::blocking::get(release.url)?;

    if !response.status().is_success() {
        bail!("failed to download file from {}", release.url);
    }

    let mut contents = Vec::new();
    response.copy_to(&mut contents)?;

    Ok(contents)
}

fn validate_checksum(bytes: &[u8], checksum: &str) -> Result<(), Error> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);

    if !hex::encode(hasher.finalize()).eq_ignore_ascii_case(checksum) {
        bail!("failed to validate checksum");
    }

    Ok(())
}
