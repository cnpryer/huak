use thiserror::Error as ThisError;

#[allow(clippy::enum_variant_names)]
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    TOMLEditError(#[from] toml_edit::TomlError),
    #[error("a problem with utf-8 parsing occurred: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}
