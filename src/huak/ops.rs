///! This module implements various operations to interact with valid workspaces
///! existing on a system.
use crate::{
    default_entrypoint_string, default_init_file_contents,
    default_main_file_contents, default_test_file_contents,
    default_virtual_environment_name, env_path_values,
    error::HuakResult,
    find_venv_root,
    fs::{self, find_root_file_bottom_up},
    git, package_iter, python_paths,
    sys::{shell_name, Terminal, TerminalOptions},
    to_importable_package_name, to_package_cononical_name, BuildOptions,
    CleanOptions, Error, FormatOptions, InstallerOptions, LintOptions, Package,
    Project, PublishOptions, PyProjectToml, TestOptions, VirtualEnvironment,
    WorkspaceOptions,
};
use std::{env::consts::OS, path::PathBuf, process::Command, str::FromStr};
use termcolor::Color;

#[derive(Default)]
pub struct OperationConfig {
    pub workspace_root: PathBuf,
    pub terminal_options: TerminalOptions,
    pub workspace_options: Option<WorkspaceOptions>,
    pub build_options: Option<BuildOptions>,
    pub format_options: Option<FormatOptions>,
    pub lint_options: Option<LintOptions>,
    pub publish_options: Option<PublishOptions>,
    pub test_options: Option<TestOptions>,
    pub installer_options: Option<InstallerOptions>,
    pub clean_options: Option<CleanOptions>,
}

pub fn add_project_dependencies(
    dependencies: &[String],
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let manifest_path = manifest_path(config);
    let mut project = Project::from_manifest(&manifest_path)?;
    let packages = package_iter(dependencies)
        .filter(|item| {
            !project.contains_dependency(item.name()).unwrap_or_default()
        })
        .collect::<Vec<Package>>();
    if packages.is_empty() {
        return Ok(());
    }
    let venv = resolve_venv(config, &mut terminal)?;
    venv.install_packages(
        &packages,
        config.installer_options.as_ref(),
        &mut terminal,
    )?;
    for package in packages {
        project.add_dependency(&package.dependency_string())?;
    }
    project.pyproject_toml().write_file(&manifest_path)
}

pub fn add_project_optional_dependencies(
    dependencies: &[String],
    group: &str,
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let manifest_path = manifest_path(config);
    let mut project = Project::from_manifest(&manifest_path)?;
    let packages = package_iter(dependencies)
        .filter(|item| {
            !project
                .contains_optional_dependency(item.name(), group)
                .unwrap_or_default()
        })
        .collect::<Vec<Package>>();
    if packages.is_empty() {
        return Ok(());
    }
    let venv = resolve_venv(config, &mut terminal)?;
    venv.install_packages(
        &packages,
        config.installer_options.as_ref(),
        &mut terminal,
    )?;
    for package in packages {
        project.add_optional_dependency(&package.dependency_string(), group)?;
    }
    project.pyproject_toml().write_file(&manifest_path)
}

pub fn build_project(config: &OperationConfig) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let manifest_path = manifest_path(config);
    let mut project = Project::from_manifest(&manifest_path)?;
    let venv = resolve_venv(config, &mut terminal)?;
    if !venv.contains_module("build")? {
        venv.install_packages(
            &[Package::from_str("build")?],
            config.installer_options.as_ref(),
            &mut terminal,
        )?;
    }
    if !project.contains_dependency_any("build")? {
        project.add_optional_dependency("build", "dev")?;
        project.pyproject_toml().write_file(&manifest_path)?;
    }
    let mut cmd = Command::new(venv.python_path());
    let mut args = vec!["-m", "build"];
    if let Some(options) = config.build_options.as_ref() {
        if let Some(it) = options.args.as_ref() {
            args.extend(it.iter().map(|item| item.as_str()));
        }
    }
    make_venv_command(&mut cmd, &venv)?;
    cmd.args(args).current_dir(&config.workspace_root);
    terminal.run_command(&mut cmd)
}

pub fn clean_project(config: &OperationConfig) -> HuakResult<()> {
    if config.workspace_root.join("dist").exists() {
        std::fs::read_dir(config.workspace_root.join("dist"))?
            .filter_map(|x| x.ok().map(|item| item.path()))
            .for_each(|item| {
                if item.is_dir() {
                    std::fs::remove_dir_all(item).ok();
                } else if item.is_file() {
                    std::fs::remove_file(item).ok();
                }
            });
    }
    if let Some(options) = config.clean_options.as_ref() {
        if options.include_pycache {
            let pattern = format!(
                "{}",
                config
                    .workspace_root
                    .join("**")
                    .join("__pycache__")
                    .display()
            );
            glob::glob(&pattern)?.for_each(|item| {
                if let Ok(it) = item {
                    std::fs::remove_dir_all(it).ok();
                }
            })
        }
        if options.include_compiled_bytecode {
            let pattern = format!(
                "{}",
                config.workspace_root.join("**").join("*.pyc").display()
            );
            glob::glob(&pattern)?.for_each(|item| {
                if let Ok(it) = item {
                    std::fs::remove_file(it).ok();
                }
            })
        }
    }
    Ok(())
}

pub fn format_project(config: &OperationConfig) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let manifest_path = manifest_path(config);
    let mut project = Project::from_manifest(&manifest_path)?;
    let venv = resolve_venv(config, &mut terminal)?;
    let packages = ["black", "ruff"]
        .iter()
        .filter(|item| !venv.contains_module(item).unwrap_or_default())
        .filter_map(|item| Package::from_str(item).ok())
        .collect::<Vec<Package>>();
    if !packages.is_empty() {
        venv.install_packages(
            &packages,
            config.installer_options.as_ref(),
            &mut terminal,
        )?;
    }
    let packages = packages
        .into_iter()
        .filter(|item| {
            !project.contains_dependency(item.name()).unwrap_or_default()
                && !project
                    .contains_dependency_any(item.name())
                    .unwrap_or_default()
        })
        .collect::<Vec<Package>>();
    for package in &packages {
        {
            project.add_optional_dependency(package.name(), "dev")?;
        }
    }
    if !packages.is_empty() {
        project.pyproject_toml().write_file(manifest_path)?;
    }
    let mut cmd = Command::new(venv.python_path());
    let mut ruff_cmd = Command::new(venv.python_path());
    let mut ruff_args =
        vec!["-m", "ruff", "check", ".", "--select", "I001", "--fix"];
    make_venv_command(&mut cmd, &venv)?;
    make_venv_command(&mut ruff_cmd, &venv)?;
    let mut args = vec!["-m", "black", "."];
    if let Some(it) = config.format_options.as_ref() {
        if let Some(a) = it.args.as_ref() {
            args.extend(a.iter().map(|item| item.as_str()));
            if a.contains(&"--check".to_string()) {
                terminal.print_warning(
                    "this check will exit early if imports aren't sorted (see https://github.com/cnpryer/huak/issues/510)",
                )?;
                ruff_args.retain(|item| *item != "--fix")
            }
        }
    }
    ruff_cmd.args(ruff_args).current_dir(&config.workspace_root);
    terminal.run_command(&mut ruff_cmd)?;
    cmd.args(args).current_dir(&config.workspace_root);
    terminal.run_command(&mut cmd)
}

pub fn init_app_project(config: &OperationConfig) -> HuakResult<()> {
    init_lib_project(config)?;
    let mut pyproject_toml = PyProjectToml::from_path(manifest_path(config))?;
    let name = pyproject_toml.project_name().ok_or(Error::InternalError(
        "failed to read project name from toml".to_string(),
    ))?;
    pyproject_toml.add_script(
        &to_package_cononical_name(name)?,
        default_entrypoint_string(&to_importable_package_name(name)?).as_str(),
    )?;
    pyproject_toml.write_file(manifest_path(config))
}

pub fn init_lib_project(config: &OperationConfig) -> HuakResult<()> {
    let manifest_path = manifest_path(config);
    if manifest_path.exists() {
        return Err(Error::ProjectTomlExistsError);
    }
    if !config.workspace_root.join(".git").exists() {
        if let Some(options) = config.workspace_options.as_ref() {
            if options.uses_git {
                git::init(&config.workspace_root)?;
            }
        }
    }
    let mut pyproject_toml = PyProjectToml::new();
    let name = fs::last_path_component(config.workspace_root.as_path())?;
    pyproject_toml.set_project_name(&name);
    pyproject_toml.write_file(manifest_path)
}

pub fn install_project_dependencies(
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let project = Project::from_manifest(manifest_path(config))?;
    let dependencies = match project.dependencies() {
        Some(it) => it,
        None => return Ok(()),
    };
    let packages = package_iter(dependencies).collect::<Vec<Package>>();
    let venv = resolve_venv(config, &mut terminal)?;
    venv.install_packages(
        &packages,
        config.installer_options.as_ref(),
        &mut terminal,
    )
}

pub fn install_project_optional_dependencies(
    groups: &[String],
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let venv = resolve_venv(config, &mut terminal)?;
    let pyproject_toml = PyProjectToml::from_path(manifest_path(config))?;
    let mut packages = Vec::new();
    let binding = Vec::new();
    // If the group "all" is passed and isn't a valid optional dependency group
    // then install everything, disregarding other groups passed.
    if pyproject_toml.optional_dependencey_group("all").is_none()
        && groups.contains(&"all".to_string())
    {
        install_project_dependencies(config)?;
        if let Some(deps) = pyproject_toml.optional_dependencies() {
            for (_, vals) in deps {
                packages.extend(vals);
            }
        }
    } else {
        groups.iter().for_each(|item| {
            pyproject_toml
                .optional_dependencey_group(item)
                .unwrap_or(&binding)
                .iter()
                .for_each(|v| {
                    packages.push(v);
                });
        })
    }
    packages.dedup();
    let packages = package_iter(packages.iter()).collect::<Vec<Package>>();
    venv.install_packages(
        &packages,
        config.installer_options.as_ref(),
        &mut terminal,
    )
}

pub fn lint_project(config: &OperationConfig) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let manifest_path = manifest_path(config);
    let mut project = Project::from_manifest(&manifest_path)?;
    let venv = resolve_venv(config, &mut terminal)?;
    if !venv.contains_module("ruff")? {
        venv.install_packages(
            &[Package::from_str("ruff")?],
            config.installer_options.as_ref(),
            &mut terminal,
        )?;
    }
    if !project.contains_dependency_any("ruff")? {
        project.add_optional_dependency("ruff", "dev")?;
        project.pyproject_toml().write_file(&manifest_path)?;
    }
    let mut cmd = Command::new(venv.python_path());
    let mut args = vec!["-m", "ruff", "check", "."];
    if let Some(it) = config.lint_options.as_ref() {
        if let Some(a) = it.args.as_ref() {
            args.extend(a.iter().map(|item| item.as_str()));
        }
        if it.include_types {
            if !venv.contains_module("mypy")? {
                venv.install_packages(
                    &[Package::from_str("mypy")?],
                    config.installer_options.as_ref(),
                    &mut terminal,
                )?;
            }
            if !project.contains_dependency_any("mypy")? {
                project.add_optional_dependency("mypy", "dev")?;
                project.pyproject_toml().write_file(&manifest_path)?;
            }
            let mut mypy_cmd = Command::new(venv.python_path());
            make_venv_command(&mut mypy_cmd, &venv)?;
            mypy_cmd
                .args(vec![
                    "-m",
                    "mypy",
                    ".",
                    "--exclude",
                    venv.name()?.as_str(),
                ])
                .current_dir(&config.workspace_root);
            terminal.run_command(&mut mypy_cmd)?;
        }
    }
    make_venv_command(&mut cmd, &venv)?;
    cmd.args(args).current_dir(&config.workspace_root);
    terminal.run_command(&mut cmd)
}

pub fn new_app_project(config: &OperationConfig) -> HuakResult<()> {
    new_lib_project(config)?;
    let name = to_importable_package_name(
        fs::last_path_component(config.workspace_root.as_path())?.as_str(),
    )?;
    let mut pyproject_toml = PyProjectToml::from_path(manifest_path(config))?;
    let src_path = config.workspace_root.join("src");
    std::fs::write(
        src_path.join(&name).join("main.py"),
        default_main_file_contents(),
    )?;
    pyproject_toml.add_script(
        &to_package_cononical_name(name.as_str())?,
        default_entrypoint_string(&to_importable_package_name(&name)?).as_str(),
    )?;
    pyproject_toml.write_file(manifest_path(config))
}

pub fn new_lib_project(config: &OperationConfig) -> HuakResult<()> {
    create_workspace(config)?;
    let last_path_component =
        fs::last_path_component(config.workspace_root.as_path())?;
    let processed_name = to_importable_package_name(&last_path_component)?;
    if manifest_path(config).exists() {
        return Err(Error::ProjectTomlExistsError);
    }
    let mut pyproject_toml = PyProjectToml::new();
    pyproject_toml.set_project_name(&last_path_component);
    pyproject_toml.write_file(manifest_path(config))?;
    let src_path = config.workspace_root.join("src");
    std::fs::create_dir_all(src_path.join(&processed_name))?;
    std::fs::create_dir_all(config.workspace_root.join("tests"))?;
    std::fs::write(
        src_path.join(&processed_name).join("__init__.py"),
        default_init_file_contents(pyproject_toml.project_version().ok_or(
            Error::InternalError("failed to read project version".to_string()),
        )?),
    )?;
    std::fs::write(
        config.workspace_root.join("tests").join("test_version.py"),
        default_test_file_contents(&processed_name),
    )
    .map_err(Error::IOError)
}

pub fn publish_project(config: &OperationConfig) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let manifest_path = manifest_path(config);
    let mut project = Project::from_manifest(&manifest_path)?;
    let venv = resolve_venv(config, &mut terminal)?;
    if !venv.contains_module("twine")? {
        venv.install_packages(
            &[Package::from_str("twine")?],
            config.installer_options.as_ref(),
            &mut terminal,
        )?;
    }
    if !project.contains_dependency_any("twine")? {
        project.add_optional_dependency("twine", "dev")?;
        project.pyproject_toml().write_file(&manifest_path)?;
    }
    let mut cmd = Command::new(venv.python_path());
    let mut args = vec!["-m", "twine", "upload", "dist/*"];
    if let Some(it) = config.publish_options.as_ref() {
        if let Some(a) = it.args.as_ref() {
            args.extend(a.iter().map(|item| item.as_str()));
        }
    }
    make_venv_command(&mut cmd, &venv)?;
    cmd.args(args).current_dir(&config.workspace_root);
    terminal.run_command(&mut cmd)
}

pub fn remove_project_dependencies(
    dependency_names: &[String],
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let manifest_path = manifest_path(config);
    let mut project = Project::from_manifest(&manifest_path)?;
    let dependencies: Vec<String> = dependency_names
        .iter()
        .filter(|item| project.contains_dependency(item).unwrap_or_default())
        .cloned()
        .collect();
    if dependencies.is_empty() {
        return Ok(());
    }
    dependencies.iter().for_each(|item| {
        project.remove_dependency(item);
    });
    let venv =
        VirtualEnvironment::from_path(find_venv_root(&config.workspace_root)?)?;
    venv.uninstall_packages(
        &dependency_names
            .iter()
            .map(|item| item.as_str())
            .collect::<Vec<&str>>(),
        None,
        &mut terminal,
    )?;
    project.pyproject_toml().write_file(&manifest_path)
}

pub fn remove_project_optional_dependencies(
    dependency_names: &[String],
    group: &str,
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let mut project = Project::from_manifest(manifest_path(config))?;
    if project.optional_dependencey_group(group).is_none() {
        return Ok(());
    }
    let dependencies: Vec<String> = dependency_names
        .iter()
        .filter(|item| {
            project
                .contains_optional_dependency(item, group)
                .unwrap_or_default()
        })
        .cloned()
        .collect();
    if dependencies.is_empty() {
        return Ok(());
    }
    dependencies.iter().for_each(|item| {
        project.remove_optional_dependency(item, group);
    });
    let venv =
        VirtualEnvironment::from_path(find_venv_root(&config.workspace_root)?)?;
    venv.uninstall_packages(
        &dependency_names
            .iter()
            .map(|item| item.as_str())
            .collect::<Vec<&str>>(),
        None,
        &mut terminal,
    )?;
    project.pyproject_toml().write_file(manifest_path(config))
}

pub fn run_command_str(
    command: &str,
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let mut cmd = Command::new(shell_name()?);
    let flag = match OS {
        "windows" => "/C",
        _ => "-c",
    };
    let venv = resolve_venv(config, &mut terminal)?;
    make_venv_command(&mut cmd, &venv)?;
    cmd.args([flag, command])
        .current_dir(&config.workspace_root);
    terminal.run_command(&mut cmd)
}

pub fn test_project(config: &OperationConfig) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let manifest_path = manifest_path(config);
    let mut project = Project::from_manifest(&manifest_path)?;
    let venv = resolve_venv(config, &mut terminal)?;
    if !venv.contains_module("pytest")? {
        venv.install_packages(
            &[Package::from_str("pytest")?],
            config.installer_options.as_ref(),
            &mut terminal,
        )?;
    }
    if !project.contains_dependency_any("pytest")? {
        project.add_optional_dependency("pytest", "dev")?;
        project.pyproject_toml.write_file(&manifest_path)?;
    }
    let mut cmd = Command::new(venv.python_path());
    make_venv_command(&mut cmd, &venv)?;
    let python_path = if config.workspace_root.join("src").exists() {
        config.workspace_root.join("src")
    } else {
        config.workspace_root.clone()
    };
    let mut args = vec!["-m", "pytest"];
    if let Some(options) = config.lint_options.as_ref() {
        if let Some(it) = options.args.as_ref() {
            args.extend(it.iter().map(|item| item.as_str()));
        }
    }
    cmd.args(args).env("PYTHONPATH", python_path);
    terminal.run_command(&mut cmd)
}

pub fn display_project_version(config: &OperationConfig) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let project = Project::from_manifest(manifest_path(config))?;
    terminal.print_custom(
        "version",
        project
            .pyproject_toml()
            .project_version()
            .unwrap_or("no version found"),
        Color::Green,
        false,
    )
}

fn make_venv_command(
    cmd: &mut Command,
    venv: &VirtualEnvironment,
) -> HuakResult<()> {
    let mut paths = match env_path_values() {
        Some(it) => it,
        None => {
            return Err(Error::InternalError(
                "failed to parse PATH variable".to_string(),
            ))
        }
    };
    paths.insert(0, venv.executables_dir_path());
    cmd.env(
        "PATH",
        std::env::join_paths(paths)
            .map_err(|e| Error::InternalError(e.to_string()))?,
    )
    .env("VIRTUAL_ENV", venv.root());
    Ok(())
}

fn create_terminal(options: &TerminalOptions) -> Terminal {
    let mut terminal = Terminal::new();
    terminal.set_verbosity(options.verbosity);
    terminal
}

pub fn find_workspace() -> HuakResult<PathBuf> {
    let cwd = std::env::current_dir()?;
    let path = match find_root_file_bottom_up(
        "pyproject.toml",
        cwd,
        PathBuf::from("/"),
    ) {
        Ok(it) => it
            .ok_or(Error::ProjectFileNotFound)?
            .parent()
            .ok_or(Error::InternalError(
                "failed to parse parent directory".to_string(),
            ))?
            .to_path_buf(),
        Err(_) => return Err(Error::ProjectFileNotFound),
    };
    Ok(path)
}

fn create_workspace(config: &OperationConfig) -> HuakResult<()> {
    let path = config.workspace_root.as_path();
    let cwd = std::env::current_dir()?;
    if (path.exists() && path != cwd)
        || (path == cwd && path.read_dir()?.count() > 0)
    {
        return Err(Error::DirectoryExists(path.to_path_buf()));
    }
    std::fs::create_dir(path)?;
    if let Some(options) = config.workspace_options.as_ref() {
        if options.uses_git {
            git::init(&config.workspace_root)?;
        }
    }
    Ok(())
}

fn manifest_path(config: &OperationConfig) -> PathBuf {
    config.workspace_root.join("pyproject.toml")
}

/// Find a virtual enironment or create one at the workspace root.
fn resolve_venv(
    config: &OperationConfig,
    terminal: &mut Terminal,
) -> HuakResult<VirtualEnvironment> {
    let root = match find_venv_root(&config.workspace_root) {
        Ok(it) => it,
        Err(Error::VenvNotFoundError) => {
            create_virtual_environment(config, terminal)?;
            config
                .workspace_root
                .join(default_virtual_environment_name())
        }
        Err(e) => return Err(e),
    };
    VirtualEnvironment::from_path(root)
}

/// Create a new virtual environment at workspace root using the found Python interpreter.
fn create_virtual_environment(
    config: &OperationConfig,
    terminal: &mut Terminal,
) -> HuakResult<()> {
    // Use the first path found.
    let python_path = match python_paths().next() {
        Some(it) => it.0,
        None => return Err(Error::PythonNotFoundError),
    };
    let args = ["-m", "venv", default_virtual_environment_name()];
    let mut cmd = Command::new(python_path);
    cmd.args(args).current_dir(&config.workspace_root);
    terminal.run_command(&mut cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fs, test_resources_dir_path, PyProjectToml, Verbosity,
        VirtualEnvironment,
    };
    use tempfile::tempdir;

    #[test]
    fn test_add_project_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let deps = ["ruff".to_string()];
        let config = OperationConfig {
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };
        let venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        venv.uninstall_packages(
            &deps.iter().map(|item| item.as_str()).collect::<Vec<&str>>(),
            None,
            &mut terminal,
        )
        .unwrap();

        add_project_dependencies(&deps, &config).unwrap();

        let project = Project::from_manifest(manifest_path(&config)).unwrap();
        let ser_toml = PyProjectToml::from_path(
            dir.join("mock-project").join("pyproject.toml"),
        )
        .unwrap();

        assert!(venv.contains_module("ruff").unwrap());
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
        let deps = ["ruff".to_string()];
        let group = "dev";
        let config = OperationConfig {
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };
        let venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        venv.uninstall_packages(
            &deps.iter().map(|item| item.as_str()).collect::<Vec<&str>>(),
            None,
            &mut terminal,
        )
        .unwrap();

        add_project_optional_dependencies(&deps, group, &config).unwrap();

        let project = Project::from_manifest(manifest_path(&config)).unwrap();
        let ser_toml = PyProjectToml::from_path(
            dir.join("mock-project").join("pyproject.toml"),
        )
        .unwrap();

        assert!(venv.contains_module("ruff").unwrap());
        assert!(deps.iter().all(|item| project
            .optional_dependencey_group("dev")
            .unwrap()
            .contains(&item.to_string())));
        assert!(deps.iter().map(|item| item).all(|item| ser_toml
            .optional_dependencey_group("dev")
            .unwrap()
            .contains(&item.to_string())));
        assert!(deps.iter().map(|item| item).all(|item| project
            .pyproject_toml()
            .optional_dependencey_group("dev")
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
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
            workspace_root: dir.join("mock-project"),
            clean_options: Some(CleanOptions {
                include_pycache: true,
                include_compiled_bytecode: true,
            }),
            ..Default::default()
        };

        clean_project(&config).unwrap();

        let dist: Vec<PathBuf> = glob::glob(&format!(
            "{}",
            config.workspace_root.join("dist").join("*").display()
        ))
        .unwrap()
        .into_iter()
        .map(|item| item.unwrap())
        .collect();
        let pycaches: Vec<PathBuf> = glob::glob(&format!(
            "{}",
            config
                .workspace_root
                .join("**")
                .join("__pycache__")
                .display()
        ))
        .unwrap()
        .into_iter()
        .map(|item| item.unwrap())
        .collect();
        let bytecode: Vec<PathBuf> = glob::glob(&format!(
            "{}",
            config.workspace_root.join("**").join("*.pyc").display()
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };
        let project = Project::from_manifest(manifest_path(&config)).unwrap();
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
    fn test_init_lib_project() {
        let dir = tempdir().unwrap().into_path();
        std::fs::create_dir(dir.join("mock-project")).unwrap();
        let config = OperationConfig {
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };

        init_lib_project(&config).unwrap();

        let toml_path = manifest_path(&config);
        let ser_toml = PyProjectToml::from_path(toml_path).unwrap();
        let mut pyproject_toml = PyProjectToml::new();
        pyproject_toml.set_project_name("mock-project");

        assert_eq!(
            ser_toml.to_string_pretty().unwrap(),
            pyproject_toml.to_string_pretty().unwrap()
        );
    }

    #[test]
    fn test_init_app_project() {
        let dir = tempdir().unwrap().into_path();
        std::fs::create_dir(dir.join("mock-project")).unwrap();
        let config = OperationConfig {
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };

        init_app_project(&config).unwrap();

        let toml_path = manifest_path(&config);
        let ser_toml = PyProjectToml::from_path(toml_path).unwrap();
        let mut pyproject_toml = PyProjectToml::new();
        pyproject_toml.set_project_name("mock-project");

        assert_eq!(
            ser_toml.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock-project"
version = "0.0.1"
description = ""
dependencies = []

[project.scripts]
mock-project = "mock_project.main:main"
"#
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };
        let venv = VirtualEnvironment::from_path(".venv").unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(config.terminal_options.verbosity);
        venv.uninstall_packages(&["click"], None, &mut terminal)
            .unwrap();
        let package = Package::from_str("click").unwrap();
        let had_package = venv.contains_package(&package).unwrap();

        install_project_dependencies(&config).unwrap();

        assert!(!had_package);
        assert!(venv.contains_package(&package).unwrap());
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };
        let venv = VirtualEnvironment::from_path(".venv").unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(config.terminal_options.verbosity);
        venv.uninstall_packages(&["pytest"], None, &mut terminal)
            .unwrap();
        let had_package = venv.contains_module("pytest").unwrap();

        install_project_optional_dependencies(&["dev".to_string()], &config)
            .unwrap();

        assert!(!had_package);
        assert!(venv.contains_module("pytest").unwrap());
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            lint_options: Some(LintOptions {
                args: None,
                include_types: true,
            }),
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
            workspace_root: dir.join("mock-project"),
            lint_options: Some(LintOptions {
                args: Some(vec!["--fix".to_string()]),
                include_types: false,
            }),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };
        let project = Project::from_manifest(manifest_path(&config)).unwrap();
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

        lint_project(&config).unwrap();

        let post_fix_str = std::fs::read_to_string(&lint_fix_filepath).unwrap();

        assert_eq!(post_fix_str, expected);
    }

    #[test]
    fn test_new_lib_project() {
        let dir = tempdir().unwrap().into_path();
        let config = OperationConfig {
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };

        new_lib_project(&config).unwrap();

        let project = Project::from_manifest(manifest_path(&config)).unwrap();
        let test_file_filepath =
            project.root().join("tests").join("test_version.py");
        let test_file = std::fs::read_to_string(&test_file_filepath).unwrap();
        let expected_test_file = r#"from mock_project import __version__


def test_version():
    __version__
"#;
        let init_file_filepath = project
            .root()
            .join("src")
            .join("mock_project")
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };

        new_app_project(&config).unwrap();

        let project = Project::from_manifest(manifest_path(&config)).unwrap();
        let ser_toml = project.pyproject_toml();
        let main_file_filepath = project
            .root()
            .join("src")
            .join("mock_project")
            .join("main.py");
        let main_file = std::fs::read_to_string(&main_file_filepath).unwrap();
        let expected_main_file = r#"def main():
    print("Hello, World!")


if __name__ == "__main__":
    main()
"#;

        assert_eq!(
            ser_toml
                .inner
                .project
                .as_ref()
                .unwrap()
                .scripts
                .as_ref()
                .unwrap()["mock-project"],
            format!("{}.main:main", "mock_project")
        );
        assert_eq!(main_file, expected_main_file);

        assert!(ser_toml.inner.project.as_ref().unwrap().scripts.is_some());
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };
        let project = Project::from_manifest(manifest_path(&config)).unwrap();
        let venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let package = Package::from_str("click==8.1.3").unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(config.terminal_options.verbosity);
        let packages = [package.clone()];
        venv.install_packages(
            &packages,
            config.installer_options.as_ref(),
            &mut terminal,
        )
        .unwrap();
        let venv_had_package = venv.contains_package(&package).unwrap();
        let toml_had_package = project
            .pyproject_toml()
            .dependencies()
            .unwrap()
            .contains(&package.dependency_string());

        remove_project_dependencies(&["click".to_string()], &config).unwrap();

        let project = Project::from_manifest(manifest_path(&config)).unwrap();
        let venv_contains_package = venv.contains_package(&package).unwrap();
        let toml_contains_package = project
            .pyproject_toml()
            .dependencies()
            .unwrap()
            .contains(&package.dependency_string());
        venv.install_packages(
            &[package],
            config.installer_options.as_ref(),
            &mut terminal,
        )
        .unwrap();

        assert!(venv_had_package);
        assert!(toml_had_package);
        assert!(!venv_contains_package);
        assert!(!toml_contains_package);
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };
        let project = Project::from_manifest(manifest_path(&config)).unwrap();
        let venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let package = Package::from_str("black==22.8.0").unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(config.terminal_options.verbosity);
        let packages = [package.clone()];
        venv.uninstall_packages(&[package.name()], None, &mut terminal)
            .unwrap();
        venv.install_packages(
            &packages,
            config.installer_options.as_ref(),
            &mut terminal,
        )
        .unwrap();
        let venv_had_package = venv.contains_module(package.name()).unwrap();
        let toml_had_package = project
            .pyproject_toml()
            .optional_dependencey_group("dev")
            .unwrap()
            .contains(&package.dependency_string());

        remove_project_optional_dependencies(
            &["black".to_string()],
            "dev",
            &config,
        )
        .unwrap();

        let project = Project::from_manifest(manifest_path(&config)).unwrap();
        let venv_contains_package =
            venv.contains_module(package.name()).unwrap();
        let toml_contains_package = project
            .pyproject_toml()
            .dependencies()
            .unwrap()
            .contains(&package.dependency_string());
        venv.uninstall_packages(&[package.name()], None, &mut terminal)
            .unwrap();

        assert!(venv_had_package);
        assert!(toml_had_package);
        assert!(!venv_contains_package);
        assert!(!toml_contains_package);
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };
        let venv =
            VirtualEnvironment::from_path(PathBuf::from(".venv")).unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(config.terminal_options.verbosity);
        venv.uninstall_packages(&["black"], None, &mut terminal)
            .unwrap();
        let venv_had_package = venv.contains_module("black").unwrap();

        run_command_str("pip install black", &config).unwrap();

        let venv_contains_package = venv.contains_module("black").unwrap();
        venv.uninstall_packages(&["black"], None, &mut terminal)
            .unwrap();

        assert!(!venv_had_package);
        assert!(venv_contains_package);
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
            workspace_root: dir.join("mock-project"),
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };

        test_project(&config).unwrap();
    }
}
