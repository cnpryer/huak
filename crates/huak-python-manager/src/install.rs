use crate::{error::Error, releases::Release};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tar::Archive;
use zstd::stream::read::Decoder;

/// An API for interacting with the release directory.
///
/// Python Standalone Builds is a source for Python distributions. The location of the installed
/// content can vary depending on the platform. At the time of writing this, this problem is
/// being further researched and more detail can be found here: <https://github.com/cnpryer/huak/issues/844>.
///
/// [WIP]
/// # Unix
///
/// python
/// ├── build
/// ├── licenses
/// └── install
///   ├── bin
///   │ ├── python
///   │ ├── python3
///   │ └── python3.11
///   ├── include
///   └── lib
///   └── share
///
/// # Windows
///
/// python
/// ├── build
/// ├── licenses
/// └── install
///   ├── Scripts
///   ├── DLLs
///   ├── include
///   ├── libs
///   ├── tcl
///   ├── python.exe
///   └── pythonw.exe
///
/// # Installing the release
///
/// The install directory is meant to be used as the installation's home. So when changing where
/// these files are accessible from would need to consider the directory structure.
///
/// ## The "Bin"
///
/// The path of the directory's *bin* -- containing executable programs to use from the
/// environment -- will depend on the platform it's installed to.
///
/// On Windows it's root/install/Scripts/.
/// On Unix platforms it's root/install/bin/.
///
/// These examples aren't comprehensive, but `PythonReleaseDir` is used to provide one
/// API for interacting with installed Python releases.
///
/// ## Python Path
///
/// The path to the directory's Python interpreter.
///
/// On Windows it's root/install/python.exe.
/// On Unix platforms it's root/install/bin/python.
///
/// ## "Installing" The Release
///
/// Note that the *Python Path* on Windows is not in the "Bin" directory. On Unix
/// platforms it's located in the "Bin" with the rest of the installed modules.
pub struct PythonReleaseDir {
    /// Python is installed to some directory. The installation directory has some root.
    root: PathBuf,
}

impl PythonReleaseDir {
    pub fn new<T: AsRef<Path>>(root: T) -> Self {
        PythonReleaseDir {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// The installed python or python.exe path.
    #[must_use]
    pub fn python_path(&self, release: Option<&Release>) -> PathBuf {
        if cfg!(unix) {
            if let Some(it) = release {
                self.bin_path()
                    .join(format!("python{}.{}", it.version.major, it.version.minor))
            } else {
                self.bin_path().join("python3")
            }
        } else if cfg!(windows) {
            self.root.join("install").join("python.exe")
        } else {
            unimplemented!()
        }
    }

    /// The "Bin" directory containing installed Python modules
    #[must_use]
    pub fn bin_path(&self) -> PathBuf {
        #[cfg(unix)]
        let path = self.root.join("install").join("bin");

        #[cfg(windows)]
        let path = self.root.join("install").join("Scripts");

        path
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Write};
    use tempfile::TempDir;

    #[cfg(unix)]
    #[test]
    fn test_python_build_standalone_helper_unix() {
        // python/install/bin/
        let dir = TempDir::new().unwrap();
        let dir = dir.path();
        let bin = dir.join("install").join("bin");
        let py = bin.join("python");
        let py3 = bin.join("python3");
        let py312 = bin.join("python3.12");
        let pythons = [py.clone(), py3, py312];
        let module = bin.join("module");

        std::fs::create_dir_all(&bin).unwrap();

        for file in pythons.iter().chain([&module]) {
            let mut file = File::create(file).unwrap();
            file.write_all(&[]).unwrap();
        }

        let release_dir = PythonReleaseDir::new(dir);

        let release_bin = release_dir.bin_path();
        let release_py = release_bin.join("python");

        assert_eq!(bin, release_bin);
        assert_eq!(py, release_py);
        assert_eq!(module, release_bin.join("module"));
    }

    #[cfg(windows)]
    #[test]
    fn test_python_build_standalone_helper_windows() {
        // python/install/
        let dir = TempDir::new().unwrap();
        let dir = dir.path();
        let parent = dir.join("install");
        let bin = parent.join("Scripts");
        let py = parent.join("python.exe");
        let pythons = [py.clone()];
        let module = bin.join("module.exe");

        std::fs::create_dir_all(&bin).unwrap();

        for file in pythons.iter().chain([&module]) {
            let mut file = File::create(file).unwrap();
            file.write_all(&[]).unwrap();
        }

        let release_dir = PythonReleaseDir::new(dir);

        let release_bin = release_dir.bin_path();
        let release_py = release_dir.python_path(None);

        assert_eq!(bin, release_bin);
        assert_eq!(py, release_py);
        assert_eq!(module, release_bin.join("module.exe"));
    }
}
