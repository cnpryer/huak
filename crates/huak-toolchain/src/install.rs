use crate::{channel::DescriptorParts, Channel, Error, LocalToolchain};
use huak_python_manager::{
    install_with_target as install_python_with_target, release_options_from_requested_version,
    resolve_release as resolve_python_release, ReleaseOption, ReleaseOptions, RequestedVersion,
    Strategy as PythonStrategy,
};
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_file;
use std::{
    env::consts::OS,
    fs::{self, hard_link, read_link},
    path::{Path, PathBuf},
    str::FromStr,
};

pub fn install_toolchain_with_target(
    toolchain: &LocalToolchain,
    target: &PathBuf,
) -> Result<(), Error> {
    if target.exists() {
        Err(Error::LocalToolchainExistsError(target.clone()))
    } else {
        setup_toolchain(toolchain, target)
    }
}

fn setup_toolchain(toolchain: &LocalToolchain, path: &PathBuf) -> Result<(), Error> {
    fs::create_dir_all(path)?;

    let downloads = path.join("downloads");
    fs::create_dir_all(&downloads)?;

    let bin = path.join("bin");
    fs::create_dir_all(&bin)?;

    // Get the path to the installed interpreter.
    let py_path = maybe_exe(
        downloads
            .join("python")
            .join("install")
            .join(py_bin_name())
            .join("python3"),
    );

    install_python(toolchain, &downloads)?;
    create_proxy_file(py_path, bin.join("python"))?;

    // TODO(cnpryer): Rest of tools
    // todo!()
    Ok(())
}

fn py_bin_name() -> &'static str {
    #[cfg(unix)]
    let name = "bin";

    #[cfg(windows)]
    let name = "Scripts";

    name
}

fn install_python(toolchain: &LocalToolchain, target: &PathBuf) -> Result<(), Error> {
    let strategy = python_strategy_from_channel(toolchain.channel())?;

    let Some(release) = resolve_python_release(&strategy) else {
        return Err(Error::PythonInstallationError(format!(
            "could not resolve python with {strategy}"
        )));
    };

    Ok(install_python_with_target(&release, target)?)
}

fn python_strategy_from_channel(channel: &Channel) -> Result<PythonStrategy, Error> {
    let options = match channel {
        Channel::Default => ReleaseOptions::default(), // TODO(cnpryer): Is there ever a case where channel default doesn't yield python default?
        Channel::Version(version) => {
            release_options_from_requested_version(RequestedVersion::from(*version))?
        }
        Channel::Descriptor(desc) => python_options_from_descriptor(desc)?,
    };

    Ok(PythonStrategy::Selection(options))
}

fn python_options_from_descriptor(desc: &DescriptorParts) -> Result<ReleaseOptions, Error> {
    let mut options = ReleaseOptions::default();

    if let Some(kind) = desc.kind.as_ref() {
        options.kind = ReleaseOption::from_str(kind).ok();
    }

    if let Some(version) = desc.version.as_ref() {
        options.version = Some(ReleaseOption::from_str(&version.to_string())?);
    }

    if let Some(architecture) = desc.architecture.as_ref() {
        options.kind = ReleaseOption::from_str(architecture).ok();
    }

    if let Some(build_configuration) = desc.build_configuration.as_ref() {
        options.kind = ReleaseOption::from_str(build_configuration).ok();
    }

    Ok(options)
}

// TODO(cnpryer):
//   - More robust support
//   - Privileged action on windows https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/create-symbolic-links
//     - Get file metadata for priv check (linux too)
//     - Do symlinks not work on linux?
//   - https://github.com/cnpryer/huak/issues/809
//   - check if file is already a link, if not attempt to make it one.
fn create_proxy_file<T: AsRef<Path>>(original: T, link: T) -> Result<(), Error> {
    let original = original.as_ref();
    let link = link.as_ref();

    // If we can read the link we'll just make our own symlink of the original link's linked file
    if let Ok(it) = read_link(original) {
        // Do our best to get the path linked file
        let p = if it.is_absolute() {
            it
        } else if let Some(parent) = original.parent() {
            parent.join(it)
        } else {
            fs::canonicalize(it)?
        };

        // Attempt to create a symlink. If that doesn't work then we hardlink. If we can't hardlink we copy.
        if try_symlink(p.as_path(), link).is_ok() || hard_link(p.as_path(), link).is_ok() {
            Ok(())
        } else {
            // Last resort is to copy the Python interpreter
            println!(
                "  failed to link {} with {}",
                link.display(),
                original.display()
            );
            println!("  copying {} to {}", original.display(), link.display());
            let _ = fs::copy(p.as_path(), link)?;
            Ok(())
        }
    } else if try_symlink(original, link).is_ok() || hard_link(original, link).is_ok() {
        Ok(())
    } else {
        todo!()
    }
}

fn try_symlink<T: AsRef<Path>>(original: T, link: T) -> Result<(), Error> {
    #[cfg(unix)]
    let err = symlink(original, link);

    #[cfg(windows)]
    let err = symlink_file(original, link);

    Ok(err?)
}

fn maybe_exe(path: PathBuf) -> PathBuf {
    if OS == "windows" && path.extension().map_or(false, |it| it == "exe") {
        path.with_extension("exe")
    } else {
        path
    }
}
