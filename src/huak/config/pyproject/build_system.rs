use pyproject_toml::BuildSystem;

const HUAK_REQUIRES: &str = "hatchling";
const HUAK_BUILD_BACKEND: &str = "hatchling.build";

#[derive(Clone)]
/// Builder struct to create load default BuildSystem data into pyproject-toml-rs.
pub struct BuildSystemBuilder {}

impl BuildSystemBuilder {
    pub fn default() -> BuildSystem {
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
    use toml_edit;

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
        assert_eq!(data.build_backend.as_ref().unwrap(), &backend);
        assert_eq!(toml_edit::ser::to_string(&data).unwrap(), string);
    }
}
