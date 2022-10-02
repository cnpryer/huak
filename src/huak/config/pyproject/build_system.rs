use serde_derive::{Deserialize, Serialize};
use pyproject_toml::{BuildSystem};

const HUAK_REQUIRES: &str = "hatchling";
const HUAK_BUILD_BACKEND: &str = "hatchling.build";

/// Build System data.
/// ```toml
/// [tool.build-system]
/// # ...
/// ```
/*
#[derive(Serialize, Deserialize)]
pub(crate) struct BuildSystem {
    pub(crate) requires: Vec<String>,
    #[serde(rename = "build-backend")]
    pub(crate) backend: String,
}
*/
impl Default for BuildSystem {
    fn default() -> BuildSystem {
        BuildSystem {
            requires: vec![HUAK_REQUIRES.to_string()],
            build_backend: Some(HUAK_BUILD_BACKEND.to_string()),
            backend_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn build_system() {
        let requires = vec![];
        let backend = "".to_string();
        let string = r#"requires = []
build-backend = ""
"#;

        let data = BuildSystem {
            requires: requires.clone(),
            build_backend: Some(backend.clone()),
            backend_path: None,
        };

        assert_eq!(data.requires, requires);
        assert_eq!(data.backend, backend);
        assert_eq!(toml::to_string(&data).unwrap(), string);
    }
}
