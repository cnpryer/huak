use crate::{
    error::Error,
    releases::{Release, RELEASES},
    Version,
};
use std::{
    env::consts::{ARCH, OS},
    fmt::Display,
    str::FromStr,
};

/// Resolve a Python Release based on a resolution `Strategy`.
#[must_use]
pub fn resolve_release(strategy: &Strategy) -> Option<Release<'static>> {
    match strategy {
        Strategy::Default => resolve_release_with_options(&ReleaseOptions::default()),
        Strategy::Selection(options) => resolve_release_with_options(options),
    }
}

fn resolve_release_with_options(options: &ReleaseOptions) -> Option<Release<'static>> {
    let mut candidates = RELEASES
        .iter()
        .filter(|it| {
            options.kind.as_ref().map_or(false, |a| a.eq_str(it.kind))
                && options.os.as_ref().map_or(false, |a| a.eq_str(it.os))
                && options
                    .architecture
                    .as_ref()
                    .map_or(false, |a| a.eq_str(it.architecture))
                && options
                    .build_configuration
                    .as_ref()
                    .map_or(false, |a| a.eq_str(it.build_configuration))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        None
    } else {
        // Sort releases by version in descending order (latest releases at the beginning of the vector)
        candidates.sort_by(|a, b| b.version.cmp(&a.version));

        if let Some(ReleaseOption::Version(req)) = options.version.as_ref() {
            candidates
                .into_iter()
                .find(|it| req.matches_version(&it.version))
                .copied()
        } else {
            candidates.first().map(|it| **it)
        }
    }
}

/// The strategy used for resolving a Python releases.
#[derive(Default)]
pub enum Strategy {
    #[default]
    /// The default resolved release is the latest minor release offset by 1 (for example if the latest
    /// minor release available is 3.12 the default is 3.11).
    Default,
    /// `Selection` - Use some selection criteria to determine the Python release. Unused
    /// options criteria will resolve to *best possible defaults*.
    Selection(ReleaseOptions),
}

impl Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Strategy::Default => write!(f, "default"),
            Strategy::Selection(options) => write!(f, "{options:?}"),
        }
    }
}

/// Options criteria used for resolving Python releases.
#[derive(Debug)]
pub struct ReleaseOptions {
    pub kind: Option<ReleaseOption>,
    pub version: Option<ReleaseOption>, // TODO(cnpryer): Refactor to default as *latest available*
    pub os: Option<ReleaseOption>,
    pub architecture: Option<ReleaseOption>,
    pub build_configuration: Option<ReleaseOption>,
}

pub fn release_options_from_requested_version(
    version: RequestedVersion,
) -> Result<ReleaseOptions, Error> {
    Ok(ReleaseOptions {
        kind: Some(ReleaseOption::Kind(ReleaseKind::default())),
        version: Some(ReleaseOption::Version(version)),
        os: Some(ReleaseOption::Os(ReleaseOs::default())),
        architecture: Some(ReleaseOption::Architecture(ReleaseArchitecture::default())),
        build_configuration: Some(ReleaseOption::BuildConfiguration(
            ReleaseBuildConfiguration::default(),
        )),
    })
}

// TODO(cnpryer): Refactor
impl Default for ReleaseOptions {
    fn default() -> Self {
        Self {
            kind: Some(ReleaseOption::Kind(ReleaseKind::CPython)),
            version: None,
            os: Some(ReleaseOption::Os(ReleaseOs::default())),
            architecture: Some(ReleaseOption::Architecture(ReleaseArchitecture::default())),
            build_configuration: Some(ReleaseOption::BuildConfiguration(
                ReleaseBuildConfiguration::default(),
            )),
        }
    }
}

/// # Options
///
/// ## Kind
/// - "cpython"
///
/// ## Version
/// - major.minor.patch
/// - major.minor
///
/// ## Os
/// - "apple"
/// - "linux"
/// - "windows"
///
/// ## Architecture
/// - "`x86_64`"
/// - "aarch64"
/// - "i686"
///
/// ## Build Configuration
/// - "pgo+lto"
/// - "pgo
#[derive(Debug, Clone)]
pub enum ReleaseOption {
    Kind(ReleaseKind),
    Version(RequestedVersion),
    Os(ReleaseOs),
    Architecture(ReleaseArchitecture),
    BuildConfiguration(ReleaseBuildConfiguration),
}

impl ReleaseOption {
    fn eq_str(&self, s: &str) -> bool {
        match self {
            Self::Kind(ReleaseKind::CPython) if s == "cpython" => true,
            Self::Os(ReleaseOs::Apple) if s == "apple" => true, // TODO(cnpryer): Could handle macos, etc. here
            Self::Os(ReleaseOs::Linux) if s == "linux" => true,
            Self::Os(ReleaseOs::Windows) if s == "windows" => true,
            Self::Architecture(ReleaseArchitecture::X86_64) if s == "x86_64" => true,
            Self::Architecture(ReleaseArchitecture::Aarch64) if s == "aarch64" => true,
            Self::Architecture(ReleaseArchitecture::I686) if s == "i686" => true,
            Self::BuildConfiguration(ReleaseBuildConfiguration::PgoPlusLto) if s == "pgo+lto" => {
                true
            }
            Self::BuildConfiguration(ReleaseBuildConfiguration::Pgo) if s == "pgo" => true,
            _ => false,
        }
    }
}

impl FromStr for ReleaseOption {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let option = match s {
            "cpython" => ReleaseOption::Kind(ReleaseKind::CPython),
            "apple" => ReleaseOption::Os(ReleaseOs::Apple),
            "linux" => ReleaseOption::Os(ReleaseOs::Linux),
            "windows" => ReleaseOption::Os(ReleaseOs::Windows),
            "x86_64" => ReleaseOption::Architecture(ReleaseArchitecture::X86_64),
            "aarch64" => ReleaseOption::Architecture(ReleaseArchitecture::Aarch64),
            "i686" => ReleaseOption::Architecture(ReleaseArchitecture::I686),
            "pgo+lto" => ReleaseOption::BuildConfiguration(ReleaseBuildConfiguration::PgoPlusLto),
            "pgo" => ReleaseOption::BuildConfiguration(ReleaseBuildConfiguration::Pgo),
            _ => ReleaseOption::Version(RequestedVersion::from_str(s)?),
        };

        Ok(option)
    }
}

#[derive(Debug, Clone, Default)]
pub enum ReleaseKind {
    #[default]
    CPython,
}

impl Display for ReleaseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseKind::CPython => write!(f, "cpython"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReleaseOs {
    Apple,
    Linux,
    Windows,
    Unknown,
}

impl Default for ReleaseOs {
    fn default() -> Self {
        match OS {
            "macos" => ReleaseOs::Apple,
            "windows" => ReleaseOs::Windows,
            "linux" => ReleaseOs::Linux, // TODO(cnpryer)
            _ => ReleaseOs::Unknown,
        }
    }
}

impl Display for ReleaseOs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseOs::Apple => write!(f, "apple"),
            ReleaseOs::Linux => write!(f, "linux"),
            ReleaseOs::Windows => write!(f, "windows"),
            ReleaseOs::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReleaseArchitecture {
    X86_64,
    Aarch64,
    I686,
    Unknown,
}

impl Default for ReleaseArchitecture {
    fn default() -> Self {
        match ARCH {
            "x86_64" => ReleaseArchitecture::X86_64,
            "aarch64" => ReleaseArchitecture::Aarch64,
            "i686" => ReleaseArchitecture::I686,
            _ => ReleaseArchitecture::Unknown,
        }
    }
}

impl Display for ReleaseArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseArchitecture::X86_64 => write!(f, "x86_64"),
            ReleaseArchitecture::Aarch64 => write!(f, "aarch64"),
            ReleaseArchitecture::I686 => write!(f, "i686"),
            ReleaseArchitecture::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReleaseBuildConfiguration {
    PgoPlusLto,
    Pgo,
}

impl Default for ReleaseBuildConfiguration {
    fn default() -> Self {
        match OS {
            "windows" => ReleaseBuildConfiguration::Pgo,
            _ => ReleaseBuildConfiguration::PgoPlusLto,
        }
    }
}

impl Display for ReleaseBuildConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseBuildConfiguration::PgoPlusLto => write!(f, "pgo+lto"),
            ReleaseBuildConfiguration::Pgo => write!(f, "pgo"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestedVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: Option<u8>,
}

impl RequestedVersion {
    /// Evaluates if some Python release's version is what was requested.
    #[must_use]
    pub fn matches_version(&self, version: &Version) -> bool {
        self.major == version.major
            && self.minor == version.minor
            && self
                .patch
                .map_or(true, |it| it == version.patch.unwrap_or(it))
    }
}

impl From<Version> for RequestedVersion {
    fn from(value: Version) -> Self {
        requested_version_from_version(value)
    }
}

fn requested_version_from_version(version: Version) -> RequestedVersion {
    RequestedVersion {
        major: version.major,
        minor: version.minor,
        patch: version.patch,
    }
}

impl FromStr for RequestedVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.').map(str::parse);

        let Some(Ok(major)) = parts.next() else {
            return Err(Error::ParseRequestedVersionError(s.to_string()));
        };

        let Some(Ok(minor)) = parts.next() else {
            return Err(Error::ParseRequestedVersionError(s.to_string()));
        };

        let patch = match parts.next() {
            Some(Ok(it)) => Some(it),
            Some(Err(_e)) => return Err(Error::ParseRequestedVersionError(s.to_string())),
            _ => None,
        };

        Ok(RequestedVersion {
            major,
            minor,
            patch,
        })
    }
}

impl Display for RequestedVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;

        if let Some(patch) = self.patch {
            write!(f, ".{patch}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_latest() {
        let latest_default = resolve_release_with_options(&ReleaseOptions::default()).unwrap();
        let resolved_release = resolve_release(&Strategy::Default).unwrap();

        assert_eq!(resolved_release, latest_default);
    }

    #[test]
    fn test_selection() {
        let resolved_release = resolve_release(&Strategy::Selection(ReleaseOptions {
            kind: ReleaseOption::from_str("cpython").ok(),
            version: ReleaseOption::from_str("3.8").ok(),
            os: ReleaseOption::from_str("apple").ok(),
            architecture: ReleaseOption::from_str("aarch64").ok(),
            build_configuration: ReleaseOption::from_str("pgo+lto").ok(),
        }))
        .unwrap();

        assert_eq!(resolved_release.kind, "cpython");
        assert_eq!(resolved_release.version.major, 3);
        assert_eq!(resolved_release.version.minor, 8);
        assert_eq!(resolved_release.os, "apple");
        assert_eq!(resolved_release.architecture, "aarch64");
        assert_eq!(resolved_release.build_configuration, "pgo+lto");
    }
}
