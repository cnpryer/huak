use anyhow::Error;
use std::str::FromStr; // TODO(cnpryer): Library code should use thiserror

#[derive(Debug, Clone)]
pub struct RequestedVersion(String); // TODO(cnpryer)

impl FromStr for RequestedVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RequestedVersion(s.to_owned()))
    }
}
