use crate::{config::pyproject::toml::Toml, project::Project};

/// Create project toml.
// TODO: Config implementations?
pub fn create_toml(project: &Project) -> Result<Toml, anyhow::Error> {
    let mut toml = Toml::default();
    let name = crate::utils::path::parse_filename(&project.root)?.to_string();
    toml.project.name = name;

    Ok(toml)
}
