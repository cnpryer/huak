use anyhow::Error; // TODO(cnpryer): Library code should use thiserror
use std::str::FromStr;

use crate::releases::Version;

#[derive(Debug, Clone)]
pub(crate) struct RequestedVersion {
    pub(crate) major: Option<u8>,
    pub(crate) minor: Option<u8>,
    pub(crate) patch: Option<u8>,
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

impl RequestedVersion {
    /// Evaluates if some Python release's version is what was requested.
    pub(crate) fn matches_version(&self, version: Version) -> bool {
        self.major.map_or(true, |it| it == version.major)
            && self.minor.map_or(true, |it| it == version.minor)
            && self.patch.map_or(true, |it| it == version.patch)
    }
}
