use huak_python_manager::{RequestedVersion, Version};
use huak_toolchain::{Channel, LocalTool, LocalToolchain};
use pep508_rs::Requirement;

use super::toolchain::{add_tool_to_toolchain, install_minimal_toolchain};
use crate::{Config, Error, HuakResult};

// TODO(cnpryer): <https://github.com/cnpryer/huak/issues/850>
pub fn install(
    package: &Requirement,
    python_version: Option<RequestedVersion>,
    _package_index_url: &str,
    config: &Config,
) -> HuakResult<()> {
    // TODO(cnpryer): Since we're treating the bin dir as a toolchain that'd mean Huak home is
    //   the root of that toolchain (given toolchain bins are standard at roots).
    let Some(home) = config.home.as_ref() else {
        return Err(Error::HuakHomeNotFound);
    };

    // TODO(cnpryer): Smarter installs
    if home.join("bin").join(&package.name).exists() {
        return config
            .terminal()
            .print_warning(format!("'{}' is already installed", &package.name));
    }

    if !home.join("bin").exists() {
        std::fs::create_dir_all(home)?;

        // TODO(cnpryer): https://github.com/cnpryer/huak/issues/871
        let channel = python_version
            .map(|it| {
                Channel::Version(Version {
                    major: it.major,
                    minor: it.minor,
                    patch: it.patch,
                })
            })
            .unwrap_or_default();

        install_minimal_toolchain(home, channel, config)?;
    }

    // TODO(cnpryer): Toolchains have names. The bin directory is used as a toolchain
    //   but there's no intention behind a toolchain named 'bin'.
    let bin = LocalToolchain::new(home);
    let package = LocalTool::from_spec(package.name.clone(), package.to_string());

    add_tool_to_toolchain(&package, &bin, config)
}
