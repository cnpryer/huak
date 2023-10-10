use huak_package_manager::{Config, Error, HuakResult};
use termcolor::Color;

#[allow(clippy::module_name_repetitions)]
pub fn display_project_version(config: &Config) -> HuakResult<()> {
    let workspace = config.workspace();
    let package = workspace.current_package()?;

    let Some(version) = package.metadata().project_version() else {
        return Err(Error::PackageVersionNotFound);
    };

    config
        .terminal()
        .print_custom("version", version, Color::Green, false)
}
