use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use crate::errors::HuakResult;

/// Package distribtion info stored in the site-packages directory adjacent to the
/// installed package artifact.
/// https://peps.python.org/pep-0376/#one-dist-info-directory-per-installed-distribution
pub struct DistInfo {
    /// File containing the name of the tool used to install the package.
    installer_file: File,
    /// File containing the package's license information.
    license_file: Option<File>,
    /// File containing metadata about the package.
    /// See
    ///   https://peps.python.org/pep-0345/
    ///   https://peps.python.org/pep-0314/
    ///   https://peps.python.org/pep-0241/
    metadata_file: File,
    /// File containing each file isntalled as part of the package's installation.
    /// See https://peps.python.org/pep-0376/#record
    record_file: File,
    /// File added to the .dist-info directory of the installed distribution if the package
    /// was explicitly requested.
    /// See https://peps.python.org/pep-0376/#requested
    requested_file: Option<File>,
    /// File containing metadata about the archive.
    wheel_file: Option<File>,
}

impl DistInfo {
    /// Construct the disttribution info data from the package's dist-info path.
    pub fn from_path(path: &Path) -> HuakResult<DistInfo> {
        let installer_file = File::open(path.join("INSTALLER"))?;
        let metadata_file = File::open(path.join("METADATA"))?;
        let license_file = File::open(path.join("LICENSE")).ok();
        let record_file = File::open(path.join("RECORD"))?;
        let requested_file = File::open(path.join("REQUESTED")).ok();
        let wheel_file = File::open(path.join("WHEEL")).ok();

        Ok(DistInfo {
            installer_file,
            license_file,
            metadata_file,
            record_file,
            requested_file,
            wheel_file,
        })
    }

    /// Get the name of the installer listed in the INSTALLER file.
    pub fn installer_name(&self) -> HuakResult<String> {
        let mut buf_reader = BufReader::new(&self.installer_file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        Ok(contents)
    }

    /// Get the LICENSE File if one exists.
    pub fn license_file(&self) -> Option<&File> {
        self.license_file.as_ref()
    }

    /// Get the METADATA File.
    pub fn metadata_file(&self) -> &File {
        &self.metadata_file
    }

    /// Get the LICENSE File if one exists.
    pub fn record_file(&self) -> &File {
        &self.record_file
    }

    /// Get all records from the RECORD File.
    pub fn records(&self) -> RecordData {
        todo!()
    }

    /// Get the REQUESTED File if one exists.
    pub fn requested_file(&self) -> Option<&File> {
        self.requested_file.as_ref()
    }

    /// Get the WHEEL File if one exists.
    pub fn wheel_file(&self) -> Option<&File> {
        self.wheel_file.as_ref()
    }
}

/// A record of an installed file associated with the installation of a Python package.
/// The data from each record is part of a CSV file's contents, so originally the row's
/// data is separated by CSV delimiters. This struct stores the row's contents parsed.
/// The contents include the Path to the file recorded, the hash string either empty
/// or containing the algorithm signature=hash-of-contents, and finally the size of the
/// file in bytes.
/// See https://peps.python.org/pep-0376/#record
pub struct RecordData(Vec<RecordRow>);

struct RecordRow(PathBuf, HashString, u32);

/// Hash string containing the [algo]=Hash.
struct HashString(HashAlgo, char, String);

#[allow(dead_code)]
enum HashAlgo {
    SHA256,
}

#[allow(dead_code)]
impl HashAlgo {
    fn as_str(&self) -> &'static str {
        match self {
            HashAlgo::SHA256 => "sha256",
        }
    }
}
