pub mod errors;
pub mod pyproject;
pub mod utils;

/// Struct containing dependency information.
#[derive(Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}
