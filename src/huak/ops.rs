///! This module implements various operations to interact with valid workspaces
///! existing on a system.
use crate::{
    error::HuakResult,
    find_venv_root, git,
    sys::{self, get_shell_name, Terminal, Verbosity},
    Error, Installer, Package, Project, ProjectType, PyProjectToml,
    VirtualEnvironment,
};
use std::{path::PathBuf, process::Command, str::FromStr};

#[derive(Default)]
pub struct OperationConfig {
    pub root: PathBuf,
    pub project_options: Option<ProjectOptions>,
    pub build_options: Option<BuildOptions>,
    pub format_options: Option<FormatOptions>,
    pub lint_options: Option<LintOptions>,
    pub publish_options: Option<PublishOptions>,
    pub installer_options: Option<InstallerOptions>,
    pub terminal_options: Option<TerminalOptions>,
    pub clean_options: Option<CleanOptions>,
}

pub struct ProjectOptions {
    pub uses_git: bool,
}
pub struct BuildOptions;
pub struct FormatOptions;
pub struct LintOptions;
pub struct PublishOptions;
pub struct InstallerOptions;
pub struct TerminalOptions {
    pub verbosity: Verbosity,
}
pub struct CleanOptions {
    include_pycache: bool,
    include_compiled_bytecode: bool,
}

/// Activate a Python virtual environment.
pub fn activate_venv(config: &OperationConfig) -> HuakResult<()> {
    let venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut terminal = Terminal::new();
    venv.activate_with_terminal(&mut terminal)
}

/// Add Python packages as a dependencies to a Python project.
pub fn add_project_dependencies(
    dependencies: &[&str],
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut terminal = Terminal::new();
    // TODO: Propagate installer configuration (potentially per-package)
    let mut installer = Installer::new();
    installer.set_config(crate::InstallerConfig {
        path: venv.executables_dir_path().join("pip"),
    });
    venv.set_installer(installer);
    let packages: HuakResult<Vec<Package>> = dependencies
        .iter()
        .map(|item| Package::from_str(item))
        .collect();
    let packages = venv.resolve_packages(&packages?)?;
    venv.install_packages(&packages, &mut terminal)?;
    let manifest_path = config.root.join("pyproject.toml");
    let mut project = Project::from_manifest(&manifest_path)?;
    for package in packages {
        project.add_dependency(&package.dependency_string());
    }
    project.pyproject_toml().write_file(&manifest_path)
}

/// Add Python packages as optional dependencies to a Python project.
pub fn add_project_optional_dependencies(
    dependencies: &[&str],
    group: &str,
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut terminal = Terminal::new();
    // TODO: Propagate installer configuration (potentially per-package)
    let mut installer = Installer::new();
    installer.set_config(crate::InstallerConfig {
        path: venv.executables_dir_path().join("pip"),
    });
    venv.set_installer(installer);
    let packages: HuakResult<Vec<Package>> = dependencies
        .iter()
        .map(|item| Package::from_str(item))
        .collect();
    let packages = venv.resolve_packages(&packages?)?;
    venv.install_packages(&packages, &mut terminal)?;
    let manifest_path = config.root.join("pyproject.toml");
    let mut project = Project::from_manifest(&manifest_path)?;
    for package in packages {
        project.add_optional_dependency(&package.dependency_string(), group);
    }
    project.pyproject_toml().write_file(&manifest_path)
}

/// Build the Python project as installable package.
pub fn build_project(config: &OperationConfig) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut terminal = Terminal::new();
    terminal.set_verbosity(Verbosity::Quiet);
    venv.install_packages(&[Package::from_str("build")?], &mut terminal)?;
    let mut cmd = Command::new(get_shell_name()?);
    let cmd = command_with_venv_context(&mut cmd, &venv)?;
    #[cfg(unix)]
    let cmd = cmd.arg("-c");
    #[cfg(windows)]
    let cmd = cmd.arg("/C");
    // TODO: Propagate CLI config
    let cmd = cmd.arg("python -m build").current_dir(&config.root);
    terminal.run_command(cmd)
}

/// Clean the dist directory.
pub fn clean_project(config: &OperationConfig) -> HuakResult<()> {
    let paths: Result<Vec<PathBuf>, std::io::Error> =
        std::fs::read_dir(config.root.join("dist"))?
            .into_iter()
            .map(|x| x.map(|item| item.path().to_path_buf()))
            .collect();
    let mut paths = paths?;
    if let Some(options) = config.clean_options.as_ref() {
        if options.include_compiled_bytecode {
            let pattern =
                format!("{}", config.root.join("**").join("*.pyc").display());
            glob::glob(&pattern)?
                .into_iter()
                .for_each(|item| match item {
                    Ok(it) => paths.push(it),
                    Err(_) => return (),
                })
        }
        if options.include_pycache {
            let pattern = format!(
                "{}",
                config.root.join("**").join("__pycache__").display()
            );
            glob::glob(&pattern)?
                .into_iter()
                .for_each(|item| match item {
                    Ok(it) => paths.push(it),
                    Err(_) => return (),
                })
        }
    }
    for path in &paths {
        if !path.exists() {
            continue;
        }
        if path.is_file() {
            std::fs::remove_file(path)?;
        }
        if path.is_dir() {
            std::fs::remove_dir(path)?;
        }
    }
    Ok(())
}

/// Format the Python project's source code.
pub fn format_project(config: &OperationConfig) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut terminal = Terminal::new();
    terminal.set_verbosity(Verbosity::Quiet);
    venv.install_packages(&[Package::from_str("black")?], &mut terminal)?;
    let mut cmd = Command::new(get_shell_name()?);
    let cmd = command_with_venv_context(&mut cmd, &venv)?;
    #[cfg(unix)]
    let cmd = cmd.arg("-c");
    #[cfg(windows)]
    let cmd = cmd.arg("/C");
    // TODO: Propagate CLI config
    let cmd = cmd.arg("python -m black .").current_dir(&config.root);
    terminal.run_command(cmd)
}

/// Initilize an existing Python project.
pub fn init_project(config: &OperationConfig) -> HuakResult<()> {
    let manifest_path = config.root.join("pyproject.toml");
    let pyproject_toml = PyProjectToml::default();
    pyproject_toml.write_file(manifest_path)
}

/// Install a Python project's dependencies to an environment.
pub fn install_project_dependencies(
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let project = Project::from_manifest(config.root.join("pyproject.toml"))?;
    let mut terminal = Terminal::new();
    // TODO: Propagate installer configuration (potentially per-package)
    let mut installer = Installer::new();
    installer.set_config(crate::InstallerConfig {
        path: venv.executables_dir_path().join("pip"),
    });
    venv.set_installer(installer);
    let packages: HuakResult<Vec<Package>> = project
        .dependencies()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|item| Package::from_str(item))
        .collect();
    let packages = venv.resolve_packages(&packages?)?;
    venv.install_packages(&packages, &mut terminal)
}

/// Install groups of a Python project's optional dependencies to an environment.
pub fn install_project_optional_dependencies(
    group: &str,
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let project = Project::from_manifest(config.root.join("pyproject.toml"))?;
    let mut terminal = Terminal::new();
    // TODO: Propagate installer configuration (potentially per-package)
    let mut installer = Installer::new();
    installer.set_config(crate::InstallerConfig {
        path: venv.executables_dir_path().join("pip"),
    });
    venv.set_installer(installer);
    let packages: HuakResult<Vec<Package>> = project
        .optional_dependencey_group(group)
        .unwrap_or(&Vec::new())
        .iter()
        .map(|item| Package::from_str(item))
        .collect();
    let packages = venv.resolve_packages(&packages?)?;
    venv.install_packages(&packages, &mut terminal)
}

/// Lint a Python project's source code.
pub fn lint_project(config: &OperationConfig) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut terminal = Terminal::new();
    terminal.set_verbosity(Verbosity::Quiet);
    venv.install_packages(&[Package::from_str("ruff")?], &mut terminal)?;
    let mut cmd = Command::new(get_shell_name()?);
    let cmd = command_with_venv_context(&mut cmd, &venv)?;
    #[cfg(unix)]
    let cmd = cmd.arg("-c");
    #[cfg(windows)]
    let cmd = cmd.arg("/C");
    // TODO: Propagate CLI config (including --fix)
    let cmd = cmd.arg("python -m ruff .").current_dir(&config.root);
    terminal.run_command(cmd)
}

/// Create a new library-like Python project on the system.
pub fn create_new_lib_project(config: &OperationConfig) -> HuakResult<()> {
    let project = Project::from(ProjectType::Library);
    project.write_project(&config.root)?;
    if let Some(options) = config.project_options.as_ref() {
        if options.uses_git {
            git::init(&config.root)?;
        }
    }
    Ok(())
}

/// Create a new application-like Python project on the system.
pub fn create_new_app_project(config: &OperationConfig) -> HuakResult<()> {
    let project = Project::from(ProjectType::Application);
    project.write_project(&config.root)?;
    if let Some(options) = config.project_options.as_ref() {
        if options.uses_git {
            git::init(&config.root)?;
        }
    }
    Ok(())
}

/// Publish the Python project as to a registry.
pub fn publish_project(config: &OperationConfig) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut terminal = Terminal::new();
    terminal.set_verbosity(Verbosity::Quiet);
    venv.install_packages(&[Package::from_str("twine")?], &mut terminal)?;
    let mut cmd = Command::new(get_shell_name()?);
    let cmd = command_with_venv_context(&mut cmd, &venv)?;
    #[cfg(unix)]
    let cmd = cmd.arg("-c");
    #[cfg(windows)]
    let cmd = cmd.arg("/C");
    // TODO: Propagate CLI config
    let cmd = cmd
        .arg("python -m twine upload dist/*")
        .current_dir(&config.root);
    terminal.run_command(cmd)
}

/// Remove a dependency from a Python project.
pub fn remove_project_dependencies(
    dependency_names: &[&str],
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut project =
        Project::from_manifest(config.root.join("pyproject.toml"))?;
    for dependency in dependency_names {
        project.remove_dependency(dependency);
    }
    let mut terminal = terminal_from_options(config.terminal_options.as_ref());
    venv.uninstall_packages(dependency_names, &mut terminal)?;
    project
        .pyproject_toml()
        .write_file(config.root.join("pyproject.toml"))
}

/// Remove a dependency from a Python project.
pub fn remove_project_optional_dependencies(
    dependency_names: &[&str],
    group: &str,
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut project =
        Project::from_manifest(config.root.join("pyproject.toml"))?;
    for dependency in dependency_names {
        project.remove_optional_dependency(dependency, group);
    }
    let mut terminal = terminal_from_options(config.terminal_options.as_ref());
    venv.uninstall_packages(dependency_names, &mut terminal)?;
    project
        .pyproject_toml()
        .write_file(config.root.join("pyproject.toml"))
}

/// Run a command from within a Python project's context.
pub fn run_command_str(
    command: &str,
    config: &OperationConfig,
) -> HuakResult<()> {
    let venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut terminal = Terminal::new();
    let mut cmd = Command::new(get_shell_name()?);
    let mut cmd = command_with_venv_context(&mut cmd, &venv)?;
    #[cfg(unix)]
    let cmd = cmd.arg("-c");
    #[cfg(windows)]
    let cmd = cmd.arg("/C");
    let cmd = cmd.arg(command).current_dir(&config.root);
    terminal.run_command(cmd)
}

/// Run a Python project's tests.
pub fn test_project(config: &OperationConfig) -> HuakResult<()> {
    let mut venv = VirtualEnvironment::from_path(find_venv_root()?)?;
    let mut terminal = Terminal::new();
    terminal.set_verbosity(Verbosity::Quiet);
    venv.install_packages(&[Package::from_str("pytest")?], &mut terminal)?;
    let mut cmd = Command::new(get_shell_name()?);
    let mut cmd = command_with_venv_context(&mut cmd, &venv)?;
    #[cfg(unix)]
    let cmd = cmd.arg("-c");
    #[cfg(windows)]
    let cmd = cmd.arg("/C");
    // TODO: Propagate CLI config
    let cmd = cmd.arg("python -m pytest");
    terminal.run_command(cmd)
}

/// Display the version of the Python project.
pub fn display_project_version(config: &OperationConfig) -> HuakResult<()> {
    let project = Project::from_manifest(config.root.join("pyproject.toml"))?;
    let mut terminal = Terminal::new();
    let verbosity = match config.terminal_options.as_ref() {
        Some(it) => it.verbosity,
        None => Verbosity::default(),
    };
    terminal.set_verbosity(verbosity);
    terminal.status(
        "",
        project
            .pyproject_toml()
            .project_version()
            .unwrap_or("no version found"),
    )
}

/// Modify a process::Command to use a virtual environment's context.
fn command_with_venv_context<'a>(
    cmd: &'a mut Command,
    venv: &VirtualEnvironment,
) -> HuakResult<&'a mut Command> {
    let mut paths = sys::env_path_values();
    paths.insert(0, venv.executables_dir_path().to_path_buf());
    let cmd = cmd
        .env(
            "PATH",
            std::env::join_paths(paths)
                .map_err(|e| Error::InternalError(e.to_string()))?,
        )
        .env("VIRTUAL_ENV", venv.root().clone());
    Ok(cmd)
}

/// Creates a Terminal from OpertionConfig TerminalOptions.
fn terminal_from_options(options: Option<&TerminalOptions>) -> Terminal {
    let mut terminal = Terminal::new();
    if let Some(it) = options {
        terminal.set_verbosity(it.verbosity);
    } else {
        terminal.set_verbosity(Verbosity::Normal);
    }
    terminal
}

/// NOTE: Operations are meant to be executed on projects and environments.
///       See https://github.com/cnpryer/huak/issues/123
///       To run some of these tests a .venv must be available at the project's root.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fs, test_resources_dir_path, PyProjectToml, VirtualEnvironment,
    };
    use tempfile::tempdir;

    #[ignore = "currently untestable"]
    #[test]
    fn test_activate_venv() {
        todo!()
    }

    #[test]
    fn test_add_project_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let deps = ["ruff"];
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };

        add_project_dependencies(&deps, &config).unwrap();

        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let ser_toml = PyProjectToml::from_path(
            dir.join("mock-project").join("pyproject.toml"),
        )
        .unwrap();

        assert!(venv.find_site_packages_package("ruff").is_some());
        assert!(deps.iter().all(|item| project
            .dependencies()
            .unwrap()
            .contains(&item.to_string())));
        assert!(deps.iter().map(|item| item).all(|item| ser_toml
            .dependencies()
            .unwrap()
            .contains(&item.to_string())));
        assert!(deps.iter().map(|item| item).all(|item| project
            .pyproject_toml()
            .dependencies()
            .unwrap()
            .contains(&item.to_string())));
    }

    #[test]
    fn test_add_optional_project_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let deps = ["ruff"];
        let group = "test";
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };

        add_project_optional_dependencies(&deps, group, &config).unwrap();

        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let ser_toml = PyProjectToml::from_path(
            dir.join("mock-project").join("pyproject.toml"),
        )
        .unwrap();

        assert!(venv.find_site_packages_package("ruff").is_some());
        assert!(deps.iter().all(|item| project
            .optional_dependencey_group("test")
            .unwrap()
            .contains(&item.to_string())));
        assert!(deps.iter().map(|item| item).all(|item| ser_toml
            .optional_dependencey_group("test")
            .unwrap()
            .contains(&item.to_string())));
        assert!(deps.iter().map(|item| item).all(|item| project
            .pyproject_toml()
            .optional_dependencey_group("test")
            .unwrap()
            .contains(&item.to_string())));
    }

    #[test]
    fn test_build_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };

        build_project(&config).unwrap();
    }

    #[test]
    fn test_clean_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            test_resources_dir_path().join("mock-project"),
            dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            clean_options: Some(CleanOptions {
                include_pycache: true,
                include_compiled_bytecode: true,
            }),
            ..Default::default()
        };

        clean_project(&config).unwrap();

        let dist: Vec<PathBuf> = glob::glob(&format!(
            "{}",
            config.root.join("dist").join("*").display()
        ))
        .unwrap()
        .into_iter()
        .map(|item| item.unwrap())
        .collect();
        let pycaches: Vec<PathBuf> = glob::glob(&format!(
            "{}",
            config.root.join("**").join("__pycache__").display()
        ))
        .unwrap()
        .into_iter()
        .map(|item| item.unwrap())
        .collect();
        let bytecode: Vec<PathBuf> = glob::glob(&format!(
            "{}",
            config.root.join("**").join("*.pyc").display()
        ))
        .unwrap()
        .into_iter()
        .map(|item| item.unwrap())
        .collect();

        assert!(dist.is_empty());
        assert!(pycaches.is_empty());
        assert!(bytecode.is_empty());
    }

    #[test]
    fn test_format_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };
        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let fmt_filepath = project
            .root()
            .join("src")
            .join("mock_project")
            .join("fmt_me.py");
        let pre_fmt_str = r#"
def fn( ):
    pass"#;
        std::fs::write(&fmt_filepath, pre_fmt_str).unwrap();

        format_project(&config).unwrap();

        let post_fmt_str = std::fs::read_to_string(&fmt_filepath).unwrap();

        assert_eq!(
            post_fmt_str,
            r#"def fn():
    pass
"#
        );
    }

    #[test]
    fn test_init_project() {
        let dir = tempdir().unwrap().into_path();
        let config = OperationConfig {
            root: dir,
            ..Default::default()
        };

        init_project(&config).unwrap();

        let toml_path = config.root.join("pyproject.toml");
        let ser_toml = PyProjectToml::from_path(toml_path).unwrap();

        assert_eq!(
            ser_toml.to_string_pretty().unwrap(),
            crate::default_pyproject_toml_contents()
        );
    }

    #[test]
    fn test_install_project_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };
        let mut venv = VirtualEnvironment::from_path(".venv").unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        venv.uninstall_packages(&["black"], &mut terminal).unwrap();
        let had_black = venv.find_site_packages_package("black").is_some();

        install_project_dependencies(&config).unwrap();

        assert!(!had_black);
        assert!(venv.find_site_packages_package("black").is_some());
    }

    #[test]
    fn test_install_project_optional_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };
        let mut venv = VirtualEnvironment::from_path(".venv").unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        venv.uninstall_packages(&["pytest"], &mut terminal).unwrap();
        let had_pytest = venv.find_site_packages_package("pytest").is_some();

        install_project_optional_dependencies("test", &config).unwrap();

        assert!(!had_pytest);
        assert!(venv.find_site_packages_package("pytest").is_some());
    }

    #[test]
    fn test_lint_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };

        lint_project(&config).unwrap();
    }

    #[test]
    fn test_fix_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };
        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let lint_fix_filepath = project
            .root()
            .join("src")
            .join("mock_project")
            .join("fix_me.py");
        let pre_fix_str = r#"
import json # this gets removed(autofixed)


def fn():
    pass
"#;
        let expected = r#"


def fn():
    pass
"#;
        std::fs::write(&lint_fix_filepath, pre_fix_str).unwrap();

        // TODO: Add --fix to LintOptions
        lint_project(&config).unwrap();

        let post_fix_str = std::fs::read_to_string(&lint_fix_filepath).unwrap();

        assert_eq!(post_fix_str, expected);
    }

    #[test]
    fn test_new_lib_project() {
        let dir = tempdir().unwrap().into_path();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };

        create_new_lib_project(&config).unwrap();

        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let test_file_filepath =
            project.root().join("tests").join("test_version.py");
        let test_file = std::fs::read_to_string(&test_file_filepath).unwrap();
        let expected_test_file = format!(
            r#"from {} import __version__


def test_version():
    __version__
"#,
            project.pyproject_toml().project_name().unwrap()
        );
        let init_file_filepath = project
            .root()
            .join("src")
            .join("project")
            .join("__init__.py");
        let init_file = std::fs::read_to_string(&init_file_filepath).unwrap();
        let expected_init_file = "__version__ = \"0.0.1\"
";

        assert!(project
            .pyproject_toml()
            .inner
            .project
            .as_ref()
            .unwrap()
            .scripts
            .is_none());
        assert_eq!(test_file, expected_test_file);
        assert_eq!(init_file, expected_init_file);
    }

    #[test]
    fn test_new_app_project() {
        let dir = tempdir().unwrap().into_path();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };

        create_new_app_project(&config).unwrap();

        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let ser_toml = project.pyproject_toml();
        let main_file_filepath =
            project.root().join("src").join("project").join("main.py");
        let main_file = std::fs::read_to_string(&main_file_filepath).unwrap();
        let expected_main_file = "\
def main():
print(\"Hello, World!\")


if __name__ == \"__main__\":
main()
";

        assert_eq!(
            ser_toml
                .inner
                .project
                .as_ref()
                .unwrap()
                .scripts
                .as_ref()
                .unwrap()[ser_toml.project_name().unwrap()],
            format!("{}.main:main", ser_toml.project_name().unwrap())
        );
        assert_eq!(main_file, expected_main_file);

        assert!(ser_toml.inner.project.as_ref().unwrap().scripts.is_some());
    }

    #[ignore = "currently untestable"]
    #[test]
    fn test_publish_project() {
        todo!()
    }

    #[test]
    fn test_remove_project_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };
        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let mut venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let package = Package::from_str("click==8.1.3").unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let packages = [package.clone()];
        venv.install_packages(&packages, &mut terminal).unwrap();
        let venv_had_package = venv.has_package(&package).unwrap();
        let toml_had_package = project
            .pyproject_toml()
            .dependencies()
            .unwrap()
            .contains(&package.dependency_string());

        remove_project_dependencies(&["click"], &config).unwrap();

        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let venv_has_package = venv.has_package(&package).unwrap();
        let toml_has_package = project
            .pyproject_toml()
            .dependencies()
            .unwrap()
            .contains(&package.dependency_string());
        venv.install_packages(&[package], &mut terminal).unwrap();

        assert!(venv_had_package);
        assert!(toml_had_package);
        assert!(!venv_has_package);
        assert!(!toml_has_package);
    }

    #[test]
    fn test_remove_project_optional_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };
        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let mut venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let package = Package::from_str("black==22.8.0").unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let packages = [package.clone()];
        venv.install_packages(&packages, &mut terminal).unwrap();
        let venv_had_package = venv.has_module(package.name()).unwrap();
        let toml_had_package = project
            .pyproject_toml()
            .optional_dependencey_group("dev")
            .unwrap()
            .contains(&package.dependency_string());

        remove_project_optional_dependencies(&["black"], "dev", &config)
            .unwrap();

        let project =
            Project::from_manifest(config.root.join("pyproject.toml")).unwrap();
        let venv_has_package = venv.has_module(package.name()).unwrap();
        let toml_has_package = project
            .pyproject_toml()
            .dependencies()
            .unwrap()
            .contains(&package.dependency_string());
        venv.install_packages(&[package], &mut terminal).unwrap();

        assert!(venv_had_package);
        assert!(toml_had_package);
        assert!(!venv_has_package);
        assert!(!toml_has_package);
    }

    #[test]
    fn test_run_command_str() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };
        let venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let venv_had_xlcsv = venv.find_site_packages_package("xlcsv").is_some();

        run_command_str("pip install xlcsv", &config).unwrap();

        let mut venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let venv_has_xlcsv = venv.find_site_packages_package("xlcsv").is_some();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        venv.uninstall_packages(&["xlcsv"], &mut terminal).unwrap();

        assert!(!venv_had_xlcsv);
        assert!(venv_has_xlcsv);
    }

    #[test]
    fn test_test_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let config = OperationConfig {
            root: dir.join("mock-project"),
            ..Default::default()
        };

        test_project(&config).unwrap();
    }
}
