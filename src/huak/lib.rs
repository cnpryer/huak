use serde_derive::Deserialize;

pub mod errors;
pub mod pyproject;
pub mod utils;

/// Struct containing dependency information.
#[derive(Clone, Deserialize, Debug)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}
