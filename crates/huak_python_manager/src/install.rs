use crate::{
    resolve::{get_release, Strategy},
    version::RequestedVersion,
};
use anyhow::{bail, Context, Error, Ok}; // TODO(cnpryer): Use thiserror in library code.
use huak_home::huak_home_dir;
use std::{fs::File, path::PathBuf};
use tar::Archive;
use tempfile::TempDir;
use zstd::decode_all;

pub(crate) fn install_to_home(
    version: &RequestedVersion,
    strategy: &Strategy,
) -> Result<(), Error> {
    let release = get_release(version, strategy).context("requested release data")?;
    let tmp_dir = TempDir::new()?;
    let tmp_name = "tmp.tar.zst";
    let tmp_path = tmp_dir.path().join(tmp_name);
    let target_dir = huak_home_dir()
        .context("requested huak's home directory")?
        .join("bin");

    download_file(release.url, &tmp_path)?;

    let mut archive = File::open(tmp_path)?;
    let decoded = decode_all(&mut archive)?;

    let mut archive = Archive::new(decoded.as_slice());
    archive.unpack(target_dir)?;

    Ok(())
}

fn download_file(url: &str, to: &PathBuf) -> Result<(), Error> {
    let mut response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        bail!("failed to download file from {url}");
    }

    let mut dest_file = File::create(to)?;
    response.copy_to(&mut dest_file)?;

    Ok(())
}
