use crate::{
    releases::Release,
    resolve::{resolve_release, Strategy},
};
use anyhow::{bail, Context, Error, Ok}; // TODO(cnpryer): Use thiserror in library code.
use huak_home::huak_home_dir;
use std::{fs::File, path::PathBuf};
use tar::Archive;
use tempfile::TempDir;
use zstd::decode_all;

/// Install a Python release to `~/.huak/bin/`.
pub(crate) fn install_to_home(strategy: &Strategy) -> Result<(), Error> {
    let release = resolve_release(strategy).context("requested release data")?;
    let tmp_dir = TempDir::new()?;
    let tmp_name = "tmp.tar.zst";
    let tmp_path = tmp_dir.path().join(tmp_name);
    let target_dir = huak_home_dir()
        .context("requested huak's home directory")?
        .join("bin");

    download_release(&release, &tmp_path)?;

    let mut archive = File::open(tmp_path)?;
    let decoded = decode_all(&mut archive)?;

    let mut archive = Archive::new(decoded.as_slice());
    Ok(archive.unpack(target_dir)?)
}

/// Download the release to a temporary archive file (`to`).
fn download_release(release: &Release, to: &PathBuf) -> Result<(), Error> {
    validate_release(release)?;

    let mut response = reqwest::blocking::get(release.url)?;

    if !response.status().is_success() {
        bail!("failed to download file from {}", release.url);
    }

    let mut file = File::create(to)?;
    response.copy_to(&mut file)?;

    Ok(())
}

/// Validation for release installation. The following is verified prior to installation:
/// - checksum
fn validate_release(_release: &Release) -> Result<(), Error> {
    todo!()
}
