use crate::releases::{Release, Version, RELEASES};
use anyhow::Error; // TODO(cnpryer): Library code should use thiserror
use std::{
    env::consts::{ARCH, OS},
    fmt::Display,
    str::FromStr,
};

/// Resolve a Python Release based on a resolution `Strategy`.
pub(crate) fn resolve_release(strategy: &Strategy) -> Option<Release<'static>> {
    match strategy {
        Strategy::Latest => resolve_release_with_options(&Options::default()),
        Strategy::Selection(options) => resolve_release_with_options(options),
    }
}

fn resolve_release_with_options(options: &Options) -> Option<Release<'static>> {
    let mut candidates = RELEASES
        .iter()
        .filter(|it| {
            it.kind == options.kind
                && it.os == options.os
                && it.architecture == options.architecture
                && it.build_configuration == options.build_configuration
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        None
    } else {
        // Sort releases by version in descending order (latest releases at the beginning of the vector)
        candidates.sort_by(|a, b| b.version.cmp(&a.version));

        if let Some(req) = options.version.as_ref() {
            candidates
                .into_iter()
                .find(|it| req.matches_version(it.version))
                .copied()
        } else {
            candidates.first().map(|it| **it)
        }
    }
}

#[derive(Default)]
/// The strategy used for resolving a Python releases.
pub(crate) enum Strategy {
    #[default]
    /// Resolve with the latest possible Python release version for the current environment.
    Latest,
    /// `Selection` - Use some selection criteria to determine the Python release. Unused
    /// options criteria will resolve to *best possible defaults*.
    Selection(Options),
}

#[derive(Default, Debug)]
/// Options criteria used for resolving Python releases.
pub(crate) struct Options {
    pub kind: ReleaseKind,
    pub version: Option<RequestedVersion>, // TODO(cnpryer): Can this default to something like *Latest*?
    pub os: ReleaseOS,
    pub architecture: ReleaseArchitecture,
    pub build_configuration: ReleaseBuildConfiguration,
}

#[derive(Debug)]
pub(crate) struct ReleaseKind(String);

impl Default for ReleaseKind {
    fn default() -> Self {
        Self(String::from("cpython"))
    }
}

impl PartialEq<str> for ReleaseKind {
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
    }
}

impl PartialEq<ReleaseKind> for &str {
    fn eq(&self, other: &ReleaseKind) -> bool {
        self == &other.0.as_str()
    }
}

#[derive(Debug)]
pub(crate) struct ReleaseOS(String);

impl Default for ReleaseOS {
    fn default() -> Self {
        Self(String::from(match OS {
            "macos" => "apple",
            "windows" => "windows",
            _ => "linux",
        }))
    }
}

impl PartialEq<str> for ReleaseOS {
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
    }
}

impl PartialEq<ReleaseOS> for &str {
    fn eq(&self, other: &ReleaseOS) -> bool {
        self == &other.0.as_str()
    }
}

#[derive(Debug)]
pub(crate) struct ReleaseArchitecture(String);

impl Default for ReleaseArchitecture {
    fn default() -> Self {
        Self(String::from(match ARCH {
            "x86_64" => "x86_64",
            "aarch64" => "aarch64",
            "x86" => "i686", // TODO(cnpryer): Need to look at other windows releases.
            _ => unimplemented!(),
        }))
    }
}

impl PartialEq<str> for ReleaseArchitecture {
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
    }
}

impl PartialEq<ReleaseArchitecture> for &str {
    fn eq(&self, other: &ReleaseArchitecture) -> bool {
        self == &other.0.as_str()
    }
}

#[derive(Debug)]
pub(crate) struct ReleaseBuildConfiguration(String);

impl Default for ReleaseBuildConfiguration {
    fn default() -> Self {
        Self(String::from(match OS {
            "windows" => "pgo",
            _ => "pgo+lto",
        }))
    }
}

impl PartialEq<str> for ReleaseBuildConfiguration {
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
    }
}

impl PartialEq<ReleaseBuildConfiguration> for &str {
    fn eq(&self, other: &ReleaseBuildConfiguration) -> bool {
        self == &other.0.as_str()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RequestedVersion {
    pub(crate) major: Option<u8>,
    pub(crate) minor: Option<u8>,
    pub(crate) patch: Option<u8>,
}

impl RequestedVersion {
    /// Evaluates if some Python release's version is what was requested.
    pub(crate) fn matches_version(&self, version: Version) -> bool {
        self.major.map_or(true, |it| it == version.major)
            && self.minor.map_or(true, |it| it == version.minor)
            && self.patch.map_or(true, |it| it == version.patch)
    }
}

impl FromStr for RequestedVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s
            .split('.')
            .map(|it| it.parse::<u8>().expect("parsed requested version part"));

        Ok(RequestedVersion {
            major: parts.next(),
            minor: parts.next(),
            patch: parts.next(),
        })
    }
}

impl Display for RequestedVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(major) = self.major {
            write!(f, "{major}")?;
        }
        if let Some(minor) = self.minor {
            write!(f, ".{minor}")?;
        }
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
        let latest_default = resolve_release_with_options(&Options::default()).unwrap();
        let resolved_release = resolve_release(&Strategy::Latest).unwrap();

        assert_eq!(resolved_release, latest_default);
    }

    #[test]
    fn test_selection() {
        let resolved_release = resolve_release(&Strategy::Selection(Options {
            kind: ReleaseKind("cpython".to_string()),
            version: Some(RequestedVersion::from_str("3.8").unwrap()),
            os: ReleaseOS("apple".to_string()),
            architecture: ReleaseArchitecture("aarch64".to_string()),
            build_configuration: ReleaseBuildConfiguration("pgo+lto".to_string()),
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
