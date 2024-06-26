use crate::{Error, HuakResult, LocalManifest};
use huak_pyproject_toml::PyProjectToml;
use lazy_static::lazy_static;
use pep440_rs::{Operator, Version, VersionSpecifiers};
use regex::Regex;
use std::{borrow::Cow, fmt::Display, str::FromStr};

const VERSION_OPERATOR_CHARACTERS: [char; 5] = ['=', '~', '!', '>', '<'];

lazy_static! {
    static ref PACKAGE_REGEX: Regex = Regex::new("[-_. ]+").expect("hyphen-underscore regex");
}

/// The `Package` contains data about a Python `Package`.
///
/// A `Package` contains information like the project's name, its version, authors,
/// its dependencies, etc.
///
/// ```
/// use huak_package_manager::Package;
/// use pep440_rs::Version;
///
/// let mut package = Package::from_str("my-project == 0.0.1").unwrap();
///
/// assert_eq!(package.version, Version::from_str("0.0.1").unwrap()));
/// ```
pub struct Package {
    /// Information used to identify the `Package`.
    id: PackageId,
    /// The `Package`'s manifest data (TODO(cnpryer): Make just core)
    manifest_data: PyProjectToml,
}

impl Package {
    /// Get a reference to the `Package`'s name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.id.name
    }

    /// Get a reference to the PEP 440 `Version` of the `Package`.
    #[must_use]
    pub fn version(&self) -> &Version {
        &self.id.version
    }

    /// Get a reference to the `Package`'s manifest data.
    #[must_use]
    pub fn manifest_data(&self) -> &PyProjectToml {
        &self.manifest_data
    }

    pub fn try_from_manifest(manifest: &LocalManifest) -> HuakResult<Self> {
        let Some(name) = manifest.manifest_data().project_name() else {
            return Err(Error::InternalError("missing project name".to_string()));
        };

        let Some(version) = manifest.manifest_data().project_version() else {
            return Err(Error::InternalError("missing project version".to_string()));
        };

        Ok(Self {
            id: PackageId {
                name,
                version: Version::from_str(&version)
                    .map_err(|e| Error::InvalidVersionString(e.to_string()))?,
            },
            manifest_data: manifest.manifest_data().clone(),
        })
    }

    // TODO: I want this implemented with `FromStr`.
    /// Initialize a `Package` from a `&str`.
    ///
    /// ```
    /// use huak_package_manager::Package;
    ///
    /// let package = Package::from_str("my-package == 0.0.1").unwrap();
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str<T: AsRef<str>>(s: T) -> HuakResult<Package> {
        // A naive approach to parsing the name and `VersionSpecifiers` from the `&str`.
        // Find the first character of the `VersionSpecifiers`. Everything prior is considered
        // the name.
        let s = s.as_ref();
        let spec_str =
            parse_version_specifiers_str(s).ok_or(Error::InvalidVersionString(s.to_string()))?;
        let name = s.strip_suffix(spec_str).unwrap_or(s).to_string();
        let version_specifiers = VersionSpecifiers::from_str(spec_str)?;

        // Since we only want to define `Package`s as having a specific `Version`,
        // a `Package` cannot be initialized with multiple `VersionSpecifier`s.
        if version_specifiers.len() > 1 {
            return Err(Error::InvalidVersionString(format!(
                "{s} can only contain one version specifier"
            )));
        }
        let version_specifer = version_specifiers.first().unwrap();
        if version_specifer.operator() != &Operator::Equal {
            return Err(Error::InvalidVersionString(format!(
                "{s} must contain {} specifier",
                Operator::Equal
            )));
        }

        let id = PackageId {
            name: canonical_package_name(&name).into_owned(),
            version: version_specifer.version().to_owned(),
        };

        let mut manifest_data = PyProjectToml::default();
        manifest_data.set_project_name(&name);

        let package = Package { id, manifest_data };

        Ok(package)
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=={}", self.name(), self.version())
    }
}

/// Two `Package`s are currently considered partially equal if their names are the same.
/// NOTE: This may change in the future.
impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Package {}

/// The `PackageId` struct is used to contain `Package`-identifying data.
#[derive(Clone)]
struct PackageId {
    /// The `Package` name.
    name: String,
    /// The `Package` PEP 440 `Version`.
    version: Version,
}

/// Parse the version specifiers component of a `Package` `&str`.
///
/// The first character of the version specififers component indicates the end of
/// the `Package` name.
fn parse_version_specifiers_str(s: &str) -> Option<&str> {
    let found = s
        .chars()
        .enumerate()
        .find(|x| VERSION_OPERATOR_CHARACTERS.contains(&x.1));

    let spec = match found {
        Some(it) => &s[it.0..],
        None => return None,
    };

    Some(spec)
}

/// Convert a name to an importable version of the name.
pub fn importable_package_name(name: &str) -> HuakResult<String> {
    let canonical_name = canonical_package_name(name);
    Ok(canonical_name.replace('-', "_"))
}

/// Normalize a name to a distributable and packagable name.
fn canonical_package_name(name: &str) -> Cow<str> {
    PACKAGE_REGEX.replace_all(name, "-")
}
