use serde_derive::{Deserialize, Serialize};
use toml::{value::Map, Value};

/// Struct containing dependency information.
#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug)]
pub(crate) struct Dependency {
    pub(crate) name: String,
    pub(crate) version: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Huak {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) description: String,
    pub(crate) authors: Vec<String>,
    pub(crate) dependencies: Map<String, Value>,
    #[serde(rename = "dev-dependencies")]
    pub(crate) dev_dependencies: Map<String, Value>,
}

impl Default for Huak {
    fn default() -> Huak {
        Huak {
            name: "".to_string(),
            version: "0.0.1".to_string(),
            description: "".to_string(),
            authors: vec![],
            dependencies: Map::new(),
            dev_dependencies: Map::new(),
        }
    }
}
