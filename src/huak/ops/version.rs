use crate::{Config, Error, HuakResult};
use termcolor::Color;

pub fn display_project_version(config: &Config) -> HuakResult<()> {
    let workspace = config.workspace();
    let package = workspace.current_package()?;

    let version = match package.metadata().project_version() {
        Some(it) => it,
        None => return Err(Error::PackageVersionNotFound),
    };

    config
        .terminal()
        .print_custom("version", version, Color::Green, false)
}
