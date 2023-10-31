use thiserror::Error as ThisError;

// TODO(cnpryer):
//   - Either more specific variants or convert from upstream.
//   - Don't reference actual crates.
#[allow(clippy::enum_variant_names)]
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("a problem occurred attempting to parse a requested version: {0}")]
    ParseRequestedVersionError(String),
    #[error("a version is invalid: {0}")]
    InvalidVersion(String),
    #[error("a problem occurred with a request: {0}")]
    RequestError(String),
    #[error("a problem with reqwest occurred: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("a problem with tar occurred: {0}")]
    TarError(String),
    #[error("a problem with zstd occurred: {0}")]
    ZstdError(String),
}
