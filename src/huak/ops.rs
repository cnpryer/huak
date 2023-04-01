///! This module implements various operations to interact with valid workspaces
///! existing on a system.
///
use indexmap::IndexMap;
use std::{env::consts::OS, path::PathBuf, process::Command, str::FromStr};
use termcolor::Color;

use crate::{
    default_entrypoint_string, default_init_file_contents,
    default_main_file_contents, default_project_version_str,
    default_test_file_contents, default_virtual_environment_name,
    dependency_iter, env_path_values, fs,
    git::{self, default_python_gitignore},
    python_paths,
    sys::Terminal,
    Config, Dependency, Error, HuakResult, PackageInstallerOptions,
    PythonEnvironment, Workspace, WorkspaceOptions,
};

pub struct AddOptions {
    pub args: Option<Vec<String>>,
    pub installer_options: Option<PackageInstallerOptions>,
}
pub struct BuildOptions {
    pub args: Option<Vec<String>>,
    pub installer_options: Option<PackageInstallerOptions>,
}
pub struct FormatOptions {
    pub args: Option<Vec<String>>,
    pub installer_options: Option<PackageInstallerOptions>,
}
pub struct LintOptions {
    pub args: Option<Vec<String>>,
    pub include_types: bool,
    pub installer_options: Option<PackageInstallerOptions>,
}
pub struct PublishOptions {
    pub args: Option<Vec<String>>,
    pub installer_options: Option<PackageInstallerOptions>,
}
pub struct TestOptions {
    pub args: Option<Vec<String>>,
}
pub struct CleanOptions {
    pub include_pycache: bool,
    pub include_compiled_bytecode: bool,
}

pub fn activate_venv(config: &mut Config) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let venv = ws.current_python_environment(config)?;

    #[cfg(unix)]
    let mut cmd = Command::new("bash");
    #[cfg(unix)]
    cmd.args([
        "--init-file",
        &format!("{}", venv.executables_dir_path().join("activate").display()),
        "-i",
    ]);
    #[cfg(windows)]
    let mut cmd = Command::new("powershell");
    #[cfg(windows)]
    cmd.args([
        "-executionpolicy",
        "bypass",
        "-NoExit",
        "-NoLogo",
        "-File",
        &format!(
            "{}",
            venv.executables_dir_path().join("activate.ps1").display()
        ),
    ]);

    config.terminal().run_command(&mut cmd)
}

pub fn add_project_dependencies(
    dependencies: &[String],
    config: &mut Config,
    options: Option<AddOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let mut project = ws.current_project(config)?;

    let deps = dependency_iter(dependencies)
        .filter(|dep| !project.contains_dependency(dep).unwrap_or_default())
        .collect::<Vec<Dependency>>();
    if deps.is_empty() {
        return Ok(());
    }

    let python_env = match ws.current_python_environment(config) {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            create_venv(&mut &ws, config)?
        }
        Err(e) => return Err(e),
    };
    python_env.install_packages(
        &dependencies,
        if let Some(o) = options {
            o.installer_options.as_ref()
        } else {
            None
        },
        &mut config.terminal(),
    )?;

    for dep in deps {
        project.add_dependency(dep)?;
    }
    project.write_manifest()
}

pub fn add_project_optional_dependencies(
    dependencies: &[String],
    group: &str,
    config: &mut Config,
    options: Option<AddOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let mut project = ws.current_project(config)?;

    let deps = dependency_iter(dependencies)
        .filter(|dep| {
            !project
                .contains_optional_dependency(dep, group)
                .unwrap_or_default()
        })
        .collect::<Vec<Dependency>>();
    if deps.is_empty() {
        return Ok(());
    };

    let python_env = match ws.current_python_environment(config) {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            create_venv(&mut &ws, config)?
        }
        Err(e) => return Err(e),
    };
    python_env.install_packages(
        &dependencies,
        if let Some(o) = options {
            o.installer_options.as_ref()
        } else {
            None
        },
        &mut config.terminal(),
    )?;

    for dep in deps {
        project.add_optional_dependency(dep, group)?;
    }
    project.write_manifest()
}

pub fn build_project(
    config: &mut Config,
    options: Option<BuildOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let mut project = ws.current_project(config)?;

    let python_env = match ws.current_python_environment(config) {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            create_venv(&mut &ws, config)?
        }
        Err(e) => return Err(e),
    };
    let build_dep = Dependency::from_str("build")?;
    if !python_env.contains_module(&build_dep.name)? {
        python_env.install_packages(
            &[build_dep],
            if let Some(o) = options {
                o.installer_options.as_ref()
            } else {
                None
            },
            &mut config.terminal(),
        )?;
    }

    if !project.contains_dependency_any(&build_dep)? {
        project.add_optional_dependency(build_dep, "dev")?;
        project.write_manifest();
    }

    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "build"];
    if let Some(options) = options.as_ref() {
        if let Some(it) = options.args.as_ref() {
            args.extend(it.iter().map(|item| item.as_str()));
        }
    }
    make_venv_command(&mut cmd, python_env)?;
    cmd.args(args).current_dir(&ws.root);

    config.terminal().run_command(&mut cmd)
}

pub fn clean_project(
    config: &mut Config,
    options: Option<CleanOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;

    if ws.root.join("dist").exists() {
        std::fs::read_dir(ws.root.join("dist"))?
            .filter_map(|x| x.ok().map(|item| item.path()))
            .for_each(|item| {
                if item.is_dir() {
                    std::fs::remove_dir_all(item).ok();
                } else if item.is_file() {
                    std::fs::remove_file(item).ok();
                }
            });
    }
    if let Some(o) = options.as_ref() {
        if o.include_pycache {
            let pattern =
                format!("{}", ws.root.join("**").join("__pycache__").display());
            glob::glob(&pattern)?.for_each(|item| {
                if let Ok(it) = item {
                    std::fs::remove_dir_all(it).ok();
                }
            })
        }
        if o.include_compiled_bytecode {
            let pattern =
                format!("{}", ws.root.join("**").join("*.pyc").display());
            glob::glob(&pattern)?.for_each(|item| {
                if let Ok(it) = item {
                    std::fs::remove_file(it).ok();
                }
            })
        }
    }
    Ok(())
}

pub fn format_project(
    config: &mut Config,
    options: Option<FormatOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let mut project = ws.current_project(config)?;

    let python_env = match ws.current_python_environment(config) {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            create_venv(&mut &ws, config)?
        }
        Err(e) => return Err(e),
    };
    let format_deps = [
        Dependency::from_str("black")?,
        Dependency::from_str("ruff")?,
    ]
    .iter()
    .filter(|item| !python_env.contains_module(&item.name).unwrap_or_default())
    .collect::<Vec<&Dependency>>();
    if !format_deps.is_empty() {
        python_env.install_packages(
            &format_deps,
            if let Some(o) = options {
                o.installer_options.as_ref()
            } else {
                None
            },
            &mut config.terminal(),
        )?;
    }

    let format_deps = format_deps
        .into_iter()
        .filter(|item| {
            !project.contains_dependency(item).unwrap_or_default()
                && !project.contains_dependency_any(item).unwrap_or_default()
        })
        .collect::<Vec<&Dependency>>();
    for dep in format_deps {
        {
            project.add_optional_dependency(*dep, "dev")?;
        }
    }
    if !format_deps.is_empty() {
        project.write_manifest()?;
    }

    let mut cmd = Command::new(python_env.python_path());
    let mut ruff_cmd = Command::new(python_env.python_path());
    let mut ruff_args =
        vec!["-m", "ruff", "check", ".", "--select", "I001", "--fix"];
    make_venv_command(&mut cmd, &python_env)?;
    make_venv_command(&mut ruff_cmd, &python_env)?;
    let mut args = vec!["-m", "black", "."];
    let terminal = config.terminal();
    if let Some(it) = options.as_ref() {
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
    ruff_cmd.args(ruff_args).current_dir(ws.root);
    terminal.run_command(&mut ruff_cmd)?;
    cmd.args(args).current_dir(ws.root);
    terminal.run_command(&mut cmd)
}

pub fn init_app_project(
    config: &mut Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    init_lib_project(config, options)?;

    let mut ws = config.workspace()?;
    let mut project = ws.current_project(config)?;
    let manifest = &mut project.manifest;
    let as_dep = Dependency::from_str(&manifest.name)?;
    let entry_point = default_entrypoint_string(&as_dep.importable_name()?);
    if let Some(scripts) = manifest.scripts.as_mut() {
        if !scripts.contains_key(&as_dep.canonical_name) {
            scripts.insert(as_dep.canonical_name, entry_point);
        }
    } else {
        manifest.scripts =
            Some(IndexMap::from_iter([(as_dep.canonical_name, entry_point)]));
    }

    project.write_manifest()
}

pub fn init_lib_project(
    config: &mut Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let mut project = ws.current_project(config)?;

    if project.manifest_path.exists() {
        return Err(Error::ProjectManifestExistsError);
    }

    init_git(&ws, options)?;
    let manifest = &mut project.manifest;
    let name = fs::last_path_component(ws.root)?;
    manifest.name = name;
    project.write_manifest()
}

pub fn install_project_dependencies(
    config: &mut Config,
    options: Option<PackageInstallerOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let project = ws.current_project(config)?;

    let python_env = match ws.current_python_environment(config) {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            create_venv(&mut &ws, config)?
        }
        Err(e) => return Err(e),
    };

    let dependencies = match project.dependencies() {
        Some(it) => it,
        None => return Ok(()),
    };
    python_env.install_packages(
        &dependencies,
        options.as_ref(),
        &mut config.terminal(),
    )
}

pub fn install_project_optional_dependencies(
    groups: &[String],
    config: &mut Config,
    options: Option<PackageInstallerOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let project = ws.current_project(config)?;

    let binding = Vec::new(); // TODO
    let mut dependencies = Vec::new();
    // If the group "all" is passed and isn't a valid optional dependency group
    // then install everything, disregarding other groups passed.
    if project.optional_dependencey_group("all").is_none()
        && groups.contains(&"all".to_string())
    {
        install_project_dependencies(config, options)?;
        if let Some(deps) = project.optional_dependencies() {
            for (_, vals) in deps {
                dependencies.extend(vals);
            }
        }
    } else {
        groups.iter().for_each(|item| {
            project
                .optional_dependencey_group(item)
                .unwrap_or(&binding)
                .iter()
                .for_each(|v| {
                    dependencies.push(v);
                });
        })
    }
    dependencies.dedup();

    let python_env = match ws.current_python_environment(config) {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            create_venv(&mut &ws, config)?
        }
        Err(e) => return Err(e),
    };
    python_env.install_packages(
        &dependencies,
        options.as_ref(),
        &mut config.terminal(),
    )
}

pub fn lint_project(
    config: &mut Config,
    options: Option<LintOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let mut project = ws.current_project(config)?;

    let python_env = match ws.current_python_environment(config) {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            create_venv(&mut &ws, config)?
        }
        Err(e) => return Err(e),
    };

    let ruff_dep = Dependency::from_str("ruff")?;
    if !python_env.contains_module("ruff")? {
        python_env.install_packages(
            &[ruff_dep],
            if let Some(o) = options {
                o.installer_options.as_ref()
            } else {
                None
            },
            &mut config.terminal(),
        )?;
    }

    let mut write_manifest = false;
    if !project.contains_dependency_any(&ruff_dep)? {
        project.add_optional_dependency(ruff_dep, "dev")?;
        write_manifest = true;
    }

    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "ruff", "check", "."];
    if let Some(it) = options.as_ref() {
        if let Some(a) = it.args.as_ref() {
            args.extend(a.iter().map(|item| item.as_str()));
        }
        if it.include_types {
            let mypy_dep = Dependency::from_str("mypy")?;
            if !python_env.contains_module("mypy")? {
                python_env.install_packages(
                    &[mypy_dep],
                    if let Some(o) = options {
                        o.installer_options.as_ref()
                    } else {
                        None
                    },
                    &mut config.terminal(),
                )?;
            }
            if !project.contains_dependency_any(&mypy_dep)? {
                project.add_optional_dependency(mypy_dep, "dev")?;
                write_manifest = true;
            }
            let mut mypy_cmd = Command::new(python_env.python_path());
            make_venv_command(&mut mypy_cmd, &python_env)?;
            mypy_cmd
                .args(vec![
                    "-m",
                    "mypy",
                    ".",
                    "--exclude",
                    python_env.name()?.as_str(),
                ])
                .current_dir(ws.root);
            config.terminal().run_command(&mut mypy_cmd)?;
        }
    }
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(ws.root);
    config.terminal().run_command(&mut cmd)?;

    if write_manifest {
        project.write_manifest()?;
    }
    Ok(())
}

pub fn list_python(config: &mut Config) -> HuakResult<()> {
    python_paths().enumerate().for_each(|(i, item)| {
        config
            .terminal()
            .print_custom(i + 1, item.1.display(), Color::Blue, false)
            .ok();
    });
    Ok(())
}

pub fn new_app_project(
    config: &mut Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    new_lib_project(config, options)?;

    let mut ws = config.workspace()?;
    let mut project = ws.current_project(config)?;
    let manifest = &mut project.manifest;
    manifest.name = fs::last_path_component(ws.root.as_path())?;
    let as_dep = Dependency::from_str(&manifest.name)?;

    let src_path = ws.root.join("src");
    std::fs::write(
        src_path.join(as_dep.importable_name()?).join("main.py"),
        default_main_file_contents(),
    )?;
    let entry_point = default_entrypoint_string(&as_dep.importable_name()?);
    if let Some(scripts) = manifest.scripts.as_ref() {
        if !scripts.contains_key(&as_dep.canonical_name) {
            scripts.insert(as_dep.canonical_name, entry_point);
        }
    } else {
        manifest.scripts =
            Some(IndexMap::from_iter([(as_dep.canonical_name, entry_point)]));
    }

    project.write_manifest()
}

pub fn new_lib_project(
    config: &mut Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let project = ws.current_project(config)?;

    if project.manifest_path.exists() {
        return Err(Error::ProjectManifestExistsError);
    }

    create_workspace(&ws, config, options)?;

    let name = &fs::last_path_component(ws.root)?;
    let as_dep = Dependency::from_str(name)?;
    let manifest = &mut project.manifest;
    manifest.name = *name;
    project.write_manifest()?;

    let src_path = ws.root.join("src");
    std::fs::create_dir_all(src_path.join(as_dep.importable_name()?))?;
    std::fs::create_dir_all(ws.root.join("tests"))?;
    std::fs::write(
        src_path.join(as_dep.importable_name()?).join("__init__.py"),
        default_init_file_contents(default_project_version_str()),
    )?;
    std::fs::write(
        ws.root.join("tests").join("test_version.py"),
        default_test_file_contents(&as_dep.importable_name()?),
    )
    .map_err(Error::IOError)
}

pub fn publish_project(
    config: &mut Config,
    options: Option<PublishOptions>,
) -> HuakResult<()> {
    let mut ws = config.workspace()?;
    let mut project = ws.current_project(config)?;

    let python_env = match ws.current_python_environment(config) {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            create_venv(&mut &ws, config)?
        }
        Err(e) => return Err(e),
    };
    let pub_dep = Dependency::from_str("twine")?;
    if !python_env.contains_module(&pub_dep.name)? {
        python_env.install_packages(
            &[pub_dep],
            if let Some(o) = options {
                o.installer_options.as_ref()
            } else {
                None
            },
            &mut config.terminal(),
        )?;
    }

    if !project.contains_dependency_any(&pub_dep)? {
        project.add_optional_dependency(pub_dep, "dev")?;
        project.write_manifest()?;
    }

    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "twine", "upload", "dist/*"];
    if let Some(it) = options.as_ref() {
        if let Some(a) = it.args.as_ref() {
            args.extend(a.iter().map(|item| item.as_str()));
        }
    }
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(ws.root);
    config.terminal().run_command(&mut cmd)
}

pub fn remove_project_dependencies(
    dependencies: &[String],
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let manifest_path = manifest_path(config);
    let mut project = Project::from_manifest(&manifest_path)?;
    let deps: Vec<String> = dependencies
        .iter()
        .filter(|item| project.contains_dependency(item).unwrap_or_default())
        .cloned()
        .collect();
    if deps.is_empty() {
        return Ok(());
    }
    deps.iter().for_each(|item| {
        project.remove_dependency(item);
    });
    let venv = Venv::from_path(find_venv_root(&config.workspace_root)?)?;
    venv.uninstall_packages(
        &deps.iter().map(|item| item.as_str()).collect::<Vec<&str>>(),
        config.installer_options.as_ref(),
        &mut terminal,
    )?;
    project.pyproject_toml().write_file(&manifest_path)
}

pub fn remove_project_optional_dependencies(
    dependencies: &[String],
    group: &str,
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let mut project = Project::from_manifest(manifest_path(config))?;
    if project.optional_dependencey_group(group).is_none() {
        return Ok(());
    }
    let deps: Vec<String> = dependencies
        .iter()
        .filter(|item| {
            project
                .contains_optional_dependency(item, group)
                .unwrap_or_default()
        })
        .cloned()
        .collect();
    if deps.is_empty() {
        return Ok(());
    }
    deps.iter().for_each(|item| {
        project.remove_optional_dependency(item, group);
    });
    let venv = Venv::from_path(find_venv_root(&config.workspace_root)?)?;
    venv.uninstall_packages(
        &deps.iter().map(|item| item.as_str()).collect::<Vec<&str>>(),
        config.installer_options.as_ref(),
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

pub fn update_project_dependencies(
    dependencies: Option<Vec<String>>,
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let project = Project::from_manifest(manifest_path(config))?;
    let venv = resolve_venv(config, &mut terminal)?;
    if let Some(it) = dependencies.as_ref() {
        let deps = it
            .iter()
            .filter_map(|item| {
                if project.contains_dependency(item).unwrap_or_default() {
                    Some(item.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if deps.is_empty() {
            return Ok(());
        }
        venv.update_packages(
            &deps,
            config.installer_options.as_ref(),
            &mut terminal,
        )?;
        return Ok(());
    }
    if let Some(it) = project.dependencies() {
        venv.update_packages(
            &it.iter().map(|item| item.as_str()).collect::<Vec<_>>(),
            config.installer_options.as_ref(),
            &mut terminal,
        )?;
    }
    Ok(())
}

pub fn update_project_optional_dependencies(
    dependencies: Option<Vec<String>>,
    group: &String,
    config: &OperationConfig,
) -> HuakResult<()> {
    let mut terminal = create_terminal(&config.terminal_options);
    let project = Project::from_manifest(manifest_path(config))?;
    let venv = resolve_venv(config, &mut terminal)?;
    if let Some(it) = dependencies.as_ref() {
        let deps = it
            .iter()
            .filter_map(|item| {
                if project
                    .contains_optional_dependency(item, group)
                    .unwrap_or_default()
                {
                    Some(item.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if deps.is_empty() {
            return Ok(());
        }
        venv.update_packages(
            &deps,
            config.installer_options.as_ref(),
            &mut terminal,
        )?;
        return Ok(());
    }
    let mut packages = Vec::new();
    let binding = Vec::new();
    // If the group "all" is passed and isn't a valid optional dependency group
    // then install everything, disregarding other groups passed.
    if project
        .pyproject_toml
        .optional_dependencey_group("all")
        .is_none()
        && *group == "all"
    {
        update_project_dependencies(dependencies, config)?;
        if let Some(deps) = project.pyproject_toml.optional_dependencies() {
            for (_, vals) in deps {
                packages.extend(vals.iter().map(|item| item.as_str()));
            }
        }
    } else {
        project
            .pyproject_toml
            .optional_dependencey_group(group)
            .unwrap_or(&binding)
            .iter()
            .for_each(|v| {
                packages.push(v.as_str());
            });
    }
    packages.dedup();
    venv.update_packages(
        &packages,
        config.installer_options.as_ref(),
        &mut terminal,
    )
}

pub fn use_python(version: String, config: &OperationConfig) -> HuakResult<()> {
    if let Some(path) = python_paths()
        .filter_map(|item| {
            if let Some(version) = item.0 {
                Some((version, item.1))
            } else {
                None
            }
        })
        .find(|item| item.0.to_string() == version)
        .map(|item| item.1)
    {
        if let Ok(venv) = Venv::from_path(
            config
                .workspace_root
                .join(default_virtual_environment_name()),
        ) {
            std::fs::remove_dir_all(venv.root())?;
        }
        let mut terminal = create_terminal(&config.terminal_options);
        let mut cmd = Command::new(path);
        cmd.args(["-m", "venv", default_virtual_environment_name()])
            .current_dir(&config.workspace_root);
        terminal.run_command(&mut cmd)
    } else {
        Err(Error::PythonNotFoundError)
    }
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
    venv: &PythonEnvironment,
) -> HuakResult<()> {
    let mut paths = match env_path_values() {
        Some(it) => it,
        None => {
            return Err(Error::InternalError(
                "failed to parse PATH variable".to_string(),
            ))
        }
    };
    paths.insert(0, venv.executables_dir_path().clone());
    cmd.env(
        "PATH",
        std::env::join_paths(paths)
            .map_err(|e| Error::InternalError(e.to_string()))?,
    )
    .env("VIRTUAL_ENV", venv.root());
    Ok(())
}

fn create_workspace(
    ws: &Workspace,
    config: &Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    if (ws.root.exists() && ws.root != config.cwd)
        || (ws.root == config.cwd && ws.root.read_dir()?.count() > 0)
    {
        return Err(Error::DirectoryExists(ws.root.to_path_buf()));
    }
    std::fs::create_dir(ws.root)?;
    init_git(ws, options)
}

fn init_git(
    ws: &Workspace,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    if let Some(o) = options.as_ref() {
        if o.uses_git {
            if !ws.root.join(".git").exists() {
                git::init(ws.root)?;
            }
            let gitignore_path = ws.root.join(".gitignore");
            if !gitignore_path.exists() {
                std::fs::write(gitignore_path, default_python_gitignore())?;
            }
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
) -> HuakResult<Venv> {
    let root = match find_venv_root(&config.workspace_root) {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            create_venv(config, terminal)?;
            config
                .workspace_root
                .join(default_virtual_environment_name())
        }
        Err(e) => return Err(e),
    };
    Venv::from_path(root)
}

/// Create a new Python environment at the workspace root.
/// found on the PATH environment variable.
/// TODO: Allow version selection.
fn create_venv<'a>(
    ws: &'a mut Workspace,
    config: &'a mut Config,
) -> HuakResult<&'a PythonEnvironment> {
    let python_path = match python_paths().next() {
        Some(it) => it.1,
        None => return Err(Error::PythonNotFoundError),
    };
    let name = default_virtual_environment_name();
    let args = ["-m", "venv", name];
    let mut cmd = Command::new(python_path);
    cmd.args(args).current_dir(ws.root);
    let mut terminal = config.terminal;
    terminal.run_command(&mut cmd)?;
    let path = ws.root.join(name);
    let env = PythonEnvironment::new(path)?;
    ws.python_environments.envs.insert(path.to_path_buf(), env);
    Ok(&env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fs, test_resources_dir_path, PyProjectToml, Venv, Verbosity};
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
        let venv = Venv::from_path(PathBuf::from(".venv")).unwrap();
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
        let venv = Venv::from_path(PathBuf::from(".venv")).unwrap();
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
        let venv = Venv::from_path(".venv").unwrap();
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
        let venv = Venv::from_path(".venv").unwrap();
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
        let venv = Venv::from_path(PathBuf::from(".venv")).unwrap();
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
        let venv = Venv::from_path(PathBuf::from(".venv")).unwrap();
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
        let venv = Venv::from_path(PathBuf::from(".venv")).unwrap();
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
    fn test_update_project_dependencies() {
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

        update_project_dependencies(None, &config).unwrap();
    }

    #[test]
    fn test_update_project_optional_dependencies() {
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

        update_project_optional_dependencies(None, &"dev".to_string(), &config)
            .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_use_python() {
        let dir = tempdir().unwrap().into_path();
        let version = python_paths().max().unwrap().0.unwrap();
        let config = OperationConfig {
            workspace_root: dir,
            terminal_options: TerminalOptions {
                verbosity: Verbosity::Quiet,
            },
            ..Default::default()
        };

        use_python(version.to_string(), &config).unwrap();

        let venv = Venv::from_path(
            config
                .workspace_root
                .join(default_virtual_environment_name()),
        )
        .unwrap();
        let version_string = version.to_string().replace(".", "");

        assert_eq!(
            venv.python_version().unwrap().to_string().replace(".", "")
                [..version_string.len()],
            version_string
        );
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
