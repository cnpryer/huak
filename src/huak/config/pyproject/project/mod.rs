use serde_derive::{Deserialize, Serialize};
use toml::{value::Map, Value};

/// Struct containing dependency information.
/// ```toml
/// name = version
/// ```
#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
pub(crate) struct Dependency {
    pub(crate) name: String,
    pub(crate) version: String,
}

/// Project table data.
/// ```toml
/// [project]
/// name = "Project"
/// version = "0.0.1"
/// description = ""
/// authors = []
/// # ...
/// ```
#[derive(Serialize, Deserialize)]
pub(crate) struct Project {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) description: String,
    pub(crate) authors: Vec<String>,
    pub(crate) dependencies: Map<String, Value>,
    #[serde(rename = "dev-dependencies")]
    pub(crate) dev_dependencies: Map<String, Value>,
}

impl Default for Project {
    fn default() -> Project {
        Project {
            name: "".to_string(),
            version: "0.0.1".to_string(),
            description: "".to_string(),
            authors: vec![],
            dependencies: Map::new(),
            dev_dependencies: Map::new(),
        }
    }
}
