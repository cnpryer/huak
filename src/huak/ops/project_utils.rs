use crate::{config::pyproject::toml::Toml, project::Project};

pub fn create_toml(project: &Project) -> Result<Toml, anyhow::Error> {
    let mut toml = Toml::default();
    let name = crate::utils::path::parse_filename(&project.root)?.to_string();
    toml.tool.huak.name = name.clone();

    Ok(toml)
}
