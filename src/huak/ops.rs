///! This module implements various operations to interact with valid workspaces
///! existing on a system.
///
use indexmap::IndexMap;
use std::{env::consts::OS, path::Path, process::Command, str::FromStr};
use termcolor::Color;

use crate::{
    default_entrypoint_string, default_init_file_contents,
    default_main_file_contents, default_project_manifest_file_name,
    default_project_version_str, default_test_file_contents,
    default_virtual_environment_name, dependency_iter, env_path_values, fs,
    git::{self, default_python_gitignore},
    python_paths,
    sys::shell_name,
    Config, Dependency, Error, HuakResult, PackageInstallerOptions, Project,
    ProjectKind, PyProjectToml, PythonEnvironment, WorkspaceOptions,
};

pub struct AddOptions {
    pub args: Option<Vec<String>>,
    pub install_options: Option<InstallOptions>,
}
pub struct BuildOptions {
    pub args: Option<Vec<String>>,
    pub install_options: Option<InstallOptions>,
}
pub struct FormatOptions {
    pub args: Option<Vec<String>>,
    pub install_options: Option<InstallOptions>,
}
#[derive(Clone)]
pub struct InstallOptions {
    pub args: Option<Vec<String>>,
}
pub struct LintOptions {
    pub args: Option<Vec<String>>,
    pub include_types: bool,
    pub install_options: Option<InstallOptions>,
}

pub struct RemoveOptions {
    pub args: Option<Vec<String>>,
    pub install_options: Option<InstallOptions>,
}
pub struct PublishOptions {
    pub args: Option<Vec<String>>,
    pub install_options: Option<InstallOptions>,
}
pub struct TestOptions {
    pub args: Option<Vec<String>>,
    pub install_options: Option<InstallOptions>,
}
#[derive(Clone)]
pub struct UpdateOptions {
    pub args: Option<Vec<String>>,
    pub install_options: Option<InstallOptions>,
}
pub struct CleanOptions {
    pub include_pycache: bool,
    pub include_compiled_bytecode: bool,
}

pub fn activate_python_environment(config: &mut Config) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let python_env = workspace.current_python_environment()?;

    if python_env.is_active() {
        return Ok(());
    }

    #[cfg(unix)]
    let mut cmd = Command::new("bash");
    #[cfg(unix)]
    cmd.args([
        "--init-file",
        &format!(
            "{}",
            python_env.executables_dir_path().join("activate").display()
        ),
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
            python_env
                .executables_dir_path()
                .join("activate.ps1")
                .display()
        ),
    ]);

    config.terminal.run_command(&mut cmd)
}

pub fn add_project_dependencies(
    dependencies: &[String],
    config: &mut Config,
    options: Option<AddOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

    let deps = dependency_iter(dependencies)
        .filter(|dep| !project.contains_dependency(dep).unwrap_or_default())
        .collect::<Vec<Dependency>>();
    if deps.is_empty() {
        return Ok(());
    }

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let installer_options = match options.as_ref() {
        Some(it) => parse_installer_options(it.install_options.as_ref()),
        None => None,
    };
    python_env.install_packages(
        dependencies,
        installer_options.as_ref(),
        &mut config.terminal,
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
    let mut workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

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

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let installer_options = match options.as_ref() {
        Some(it) => parse_installer_options(it.install_options.as_ref()),
        None => None,
    };
    python_env.install_packages(
        dependencies,
        installer_options.as_ref(),
        &mut config.terminal,
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
    let mut workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let build_dep = Dependency::from_str("build")?;
    if !python_env.contains_module(&build_dep.name)? {
        let installer_options = match options.as_ref() {
            Some(it) => parse_installer_options(it.install_options.as_ref()),
            None => None,
        };
        python_env.install_packages(
            &[&build_dep],
            installer_options.as_ref(),
            &mut config.terminal,
        )?;
    }

    if !project.contains_dependency_any(&build_dep)? {
        project.add_optional_dependency(build_dep, "dev")?;
        project.write_manifest()?;
    }

    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "build"];
    if let Some(options) = options.as_ref() {
        if let Some(it) = options.args.as_ref() {
            args.extend(it.iter().map(|item| item.as_str()));
        }
    }
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(&workspace.root);

    config.terminal.run_command(&mut cmd)
}

pub fn clean_project(
    config: &mut Config,
    options: Option<CleanOptions>,
) -> HuakResult<()> {
    let workspace = config.workspace()?;

    if workspace.root.join("dist").exists() {
        std::fs::read_dir(workspace.root.join("dist"))?
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
            let pattern = format!(
                "{}",
                workspace.root.join("**").join("__pycache__").display()
            );
            glob::glob(&pattern)?.for_each(|item| {
                if let Ok(it) = item {
                    std::fs::remove_dir_all(it).ok();
                }
            })
        }
        if o.include_compiled_bytecode {
            let pattern = format!(
                "{}",
                workspace.root.join("**").join("*.pyc").display()
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

pub fn format_project(
    config: &mut Config,
    options: Option<FormatOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let format_deps = [
        Dependency::from_str("black")?,
        Dependency::from_str("ruff")?,
    ];
    let new_format_deps = format_deps
        .iter()
        .filter(|item| {
            !python_env.contains_module(&item.name).unwrap_or_default()
        })
        .collect::<Vec<&Dependency>>();
    if !new_format_deps.is_empty() {
        let installer_options = match options.as_ref() {
            Some(it) => parse_installer_options(it.install_options.as_ref()),
            None => None,
        };
        python_env.install_packages(
            &new_format_deps,
            installer_options.as_ref(),
            &mut config.terminal,
        )?;
    }

    let new_format_deps = format_deps
        .into_iter()
        .filter(|item| {
            !project.contains_dependency(item).unwrap_or_default()
                && !project.contains_dependency_any(item).unwrap_or_default()
        })
        .collect::<Vec<Dependency>>();
    if !new_format_deps.is_empty() {
        for dep in new_format_deps {
            {
                project.add_optional_dependency(dep, "dev")?;
            }
        }
        project.write_manifest()?;
    }

    let mut cmd = Command::new(python_env.python_path());
    let mut ruff_cmd = Command::new(python_env.python_path());
    let mut ruff_args =
        vec!["-m", "ruff", "check", ".", "--select", "I001", "--fix"];
    make_venv_command(&mut cmd, &python_env)?;
    make_venv_command(&mut ruff_cmd, &python_env)?;
    let mut args = vec!["-m", "black", "."];
    if let Some(it) = options.as_ref() {
        if let Some(a) = it.args.as_ref() {
            args.extend(a.iter().map(|item| item.as_str()));
            if a.contains(&"--check".to_string()) {
                config.terminal.print_warning(
                    "this check will exit early if imports aren't sorted (see https://github.com/cnpryer/huak/issues/510)",
                )?;
                ruff_args.retain(|item| *item != "--fix")
            }
        }
    }
    ruff_cmd.args(ruff_args).current_dir(&workspace.root);
    config.terminal.run_command(&mut ruff_cmd)?;
    cmd.args(args).current_dir(&workspace.root);
    config.terminal.run_command(&mut cmd)
}

pub fn init_app_project(
    config: &mut Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    init_lib_project(config, options)?;

    let workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

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
    let manifest_path = config
        .workspace_root
        .join(default_project_manifest_file_name());
    if manifest_path.exists() {
        return Err(Error::ProjectManifestExistsError);
    }

    let mut pyproject_toml = PyProjectToml::default();

    init_git(&config.workspace_root, options)?;
    let name = fs::last_path_component(&config.workspace_root)?;
    pyproject_toml.set_project_name(name);

    pyproject_toml.write_file(manifest_path)
}

pub fn install_project_dependencies(
    config: &mut Config,
    options: Option<InstallOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let project = Project::new(&workspace.root)?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };

    let dependencies = match project.dependencies() {
        Some(it) => it,
        None => return Ok(()),
    };

    python_env.install_packages(
        dependencies,
        parse_installer_options(options.as_ref()).as_ref(),
        &mut config.terminal,
    )
}

pub fn install_project_optional_dependencies(
    groups: &[String],
    config: &mut Config,
    options: Option<InstallOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let project = Project::new(&workspace.root)?;

    let binding = Vec::new(); // TODO
    let mut dependencies = Vec::new();
    // If the group "all" is passed and isn't a valid optional dependency group
    // then install everything, disregarding other groups passed.
    if project.optional_dependencey_group("all").is_none()
        && groups.contains(&"all".to_string())
    {
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

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    python_env.install_packages(
        &dependencies,
        parse_installer_options(options.as_ref()).as_ref(),
        &mut config.terminal,
    )
}

pub fn lint_project(
    config: &mut Config,
    options: Option<LintOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };

    let ruff_dep = Dependency::from_str("ruff")?;
    if !python_env.contains_module("ruff")? {
        let installer_options = match options.as_ref() {
            Some(it) => parse_installer_options(it.install_options.as_ref()),
            None => None,
        };
        python_env.install_packages(
            &[&ruff_dep],
            installer_options.as_ref(),
            &mut config.terminal,
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
            let installer_options = match options.as_ref() {
                Some(it) => {
                    parse_installer_options(it.install_options.as_ref())
                }
                None => None,
            };
            if !python_env.contains_module("mypy")? {
                python_env.install_packages(
                    &[&mypy_dep],
                    installer_options.as_ref(),
                    &mut config.terminal,
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
                .current_dir(&workspace.root);
            config.terminal.run_command(&mut mypy_cmd)?;
        }
    }
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(&workspace.root);
    config.terminal.run_command(&mut cmd)?;

    if write_manifest {
        project.write_manifest()?;
    }
    Ok(())
}

pub fn list_python(config: &mut Config) -> HuakResult<()> {
    python_paths().enumerate().for_each(|(i, item)| {
        config
            .terminal
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

    let workspace = config.workspace()?;
    let mut project = workspace.current_project()?;
    project.kind = ProjectKind::Application;

    let manifest = &mut project.manifest;
    manifest.name = fs::last_path_component(workspace.root.as_path())?;
    let as_dep = Dependency::from_str(&manifest.name)?;

    let src_path = workspace.root.join("src");
    std::fs::write(
        src_path.join(as_dep.importable_name()?).join("main.py"),
        default_main_file_contents(),
    )?;
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

pub fn new_lib_project(
    config: &mut Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    let manifest_path = config
        .workspace_root
        .join(default_project_manifest_file_name());
    if manifest_path.exists() {
        return Err(Error::ProjectManifestExistsError);
    }

    let mut pyproject_toml = PyProjectToml::default();

    create_workspace(&config.workspace_root, config, options)?;

    let name = &fs::last_path_component(&config.workspace_root)?;
    pyproject_toml.set_project_name(name.to_string());
    pyproject_toml.write_file(manifest_path)?;

    let as_dep = Dependency::from_str(name)?;
    let src_path = config.workspace_root.join("src");
    std::fs::create_dir_all(src_path.join(as_dep.importable_name()?))?;
    std::fs::create_dir_all(config.workspace_root.join("tests"))?;
    std::fs::write(
        src_path.join(as_dep.importable_name()?).join("__init__.py"),
        default_init_file_contents(default_project_version_str()),
    )?;
    std::fs::write(
        config.workspace_root.join("tests").join("test_version.py"),
        default_test_file_contents(&as_dep.importable_name()?),
    )
    .map_err(Error::IOError)
}

pub fn publish_project(
    config: &mut Config,
    options: Option<PublishOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let pub_dep = Dependency::from_str("twine")?;
    let installer_options = match options.as_ref() {
        Some(it) => parse_installer_options(it.install_options.as_ref()),
        None => None,
    };
    if !python_env.contains_module(&pub_dep.name)? {
        python_env.install_packages(
            &[&pub_dep],
            installer_options.as_ref(),
            &mut config.terminal,
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
    cmd.args(args).current_dir(&workspace.root);
    config.terminal.run_command(&mut cmd)
}

pub fn remove_project_dependencies(
    dependencies: &[String],
    config: &mut Config,
    options: Option<RemoveOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

    let deps = dependency_iter(dependencies)
        .filter(|item| project.contains_dependency(item).unwrap_or_default())
        .collect::<Vec<_>>();
    if deps.is_empty() {
        return Ok(());
    }

    for dep in &deps {
        project.remove_dependency(dep)?;
    }

    project.write_manifest()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => return Ok(()),
        Err(e) => return Err(e),
    };
    let installer_options = match options.as_ref() {
        Some(it) => parse_installer_options(it.install_options.as_ref()),
        None => None,
    };
    python_env.uninstall_packages(
        &deps,
        installer_options.as_ref(),
        &mut config.terminal,
    )
}

pub fn remove_project_optional_dependencies(
    dependencies: &[String],
    group: &str,
    config: &mut Config,
    options: Option<RemoveOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

    if project.optional_dependencey_group(group).is_none() {
        return Ok(());
    }

    let deps: Vec<Dependency> = dependency_iter(dependencies)
        .filter(|item| {
            project
                .contains_optional_dependency(item, group)
                .unwrap_or_default()
        })
        .collect();
    if deps.is_empty() {
        return Ok(());
    }

    for dep in &deps {
        project.remove_optional_dependency(dep, group)?;
    }

    project.write_manifest()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => return Ok(()),
        Err(e) => return Err(e),
    };
    let installer_options = match options {
        Some(it) => parse_installer_options(it.install_options.as_ref()),
        None => None,
    };
    python_env.uninstall_packages(
        &deps,
        installer_options.as_ref(),
        &mut config.terminal,
    )
}

pub fn run_command_str(command: &str, config: &mut Config) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let python_env = workspace.current_python_environment()?;

    let mut cmd = Command::new(shell_name()?);
    let flag = match OS {
        "windows" => "/C",
        _ => "-c",
    };
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args([flag, command]).current_dir(&workspace.root);
    config.terminal.run_command(&mut cmd)
}

pub fn test_project(
    config: &mut Config,
    options: Option<TestOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let mut project = workspace.current_project()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let test_dep = Dependency::from_str("pytest")?;
    if !python_env.contains_module(&test_dep.name)? {
        let installer_options = match options.as_ref() {
            Some(it) => parse_installer_options(it.install_options.as_ref()),
            None => None,
        };
        python_env.install_packages(
            &[&test_dep],
            installer_options.as_ref(),
            &mut config.terminal,
        )?;
    }

    if !project.contains_dependency_any(&test_dep)? {
        project.add_optional_dependency(test_dep, "dev")?;
        project.write_manifest()?;
    }

    let mut cmd = Command::new(python_env.python_path());
    make_venv_command(&mut cmd, &python_env)?;
    let python_path = if workspace.root.join("src").exists() {
        workspace.root.join("src")
    } else {
        workspace.root.clone()
    };
    let mut args = vec!["-m", "pytest"];
    if let Some(o) = options.as_ref() {
        if let Some(it) = o.args.as_ref() {
            args.extend(it.iter().map(|item| item.as_str()));
        }
    }
    cmd.args(args).env("PYTHONPATH", python_path);
    config.terminal.run_command(&mut cmd)
}

pub fn update_project_dependencies(
    dependencies: Option<Vec<String>>,
    config: &mut Config,
    options: Option<UpdateOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let mut project = Project::new(&workspace.root)?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };

    if let Some(it) = dependencies.as_ref() {
        let deps = dependency_iter(it)
            .filter_map(|item| {
                if project.contains_dependency(&item).unwrap_or_default() {
                    Some(item)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if deps.is_empty() {
            return Ok(());
        }
        let installer_options = match options {
            Some(it) => parse_installer_options(it.install_options.as_ref()),
            None => None,
        };
        python_env.update_packages(
            &deps,
            installer_options.as_ref(),
            &mut config.terminal,
        )?;
    } else if let Some(deps) = project.dependencies() {
        let installer_options = match options {
            Some(it) => parse_installer_options(it.install_options.as_ref()),
            None => None,
        };
        python_env.update_packages(
            &deps.iter().map(|item| &item.name).collect::<Vec<_>>(),
            installer_options.as_ref(),
            &mut config.terminal,
        )?;
    }

    let mut write = false;
    let packages = python_env.installed_packages()?;
    for pkg in packages {
        let dep = Dependency::from_str(&pkg.to_string())?;
        if project.contains_dependency(&dep)? {
            project.remove_dependency(&dep)?;
            project.add_dependency(dep)?;
            write = true;
        }

        if write {
            project.write_manifest()?;
        }
    }

    Ok(())
}

pub fn update_project_optional_dependencies(
    dependencies: Option<Vec<String>>,
    group: &str,
    config: &mut Config,
    options: Option<UpdateOptions>,
) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let mut project = Project::new(&workspace.root)?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFoundError) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };

    if let Some(it) = dependencies.as_ref() {
        let deps = dependency_iter(it)
            .filter_map(|item| {
                if project
                    .contains_optional_dependency(&item, group)
                    .unwrap_or_default()
                {
                    Some(item)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if deps.is_empty() {
            return Ok(());
        }
        let installer_options = match options {
            Some(it) => parse_installer_options(it.install_options.as_ref()),
            None => None,
        };
        python_env.update_packages(
            &deps,
            installer_options.as_ref(),
            &mut config.terminal,
        )?;
    } else {
        let mut deps = Vec::new();
        let binding = Vec::new(); // TODO

        // If the group "all" is passed and isn't a valid optional dependency group
        // then install everything, disregarding other groups passed.
        if project.optional_dependencey_group("all").is_none() && group == "all"
        {
            if let Some(it) = project.optional_dependencies() {
                for (_, vals) in it {
                    deps.extend(vals);
                }
            }
        } else {
            project
                .optional_dependencey_group(group)
                .unwrap_or(&binding)
                .iter()
                .for_each(|item| {
                    deps.push(item);
                });
        }

        deps.dedup();
        let installer_options = match options {
            Some(it) => parse_installer_options(it.install_options.as_ref()),
            None => None,
        };
        python_env.update_packages(
            &deps,
            installer_options.as_ref(),
            &mut config.terminal,
        )?;
    }

    let mut write = false;
    let packages = python_env.installed_packages()?;
    for pkg in packages {
        let dep = Dependency::from_str(&pkg.to_string())?;
        if project.contains_dependency(&dep)? && group == "all" {
            project.remove_dependency(&dep)?;
            project.add_dependency(dep)?;
            write = true;
        } else if project.contains_optional_dependency(&dep, group)? {
            project.remove_optional_dependency(&dep, group)?;
            project.add_optional_dependency(dep, group)?;
            write = true;
        }

        if write {
            project.write_manifest()?;
        }
    }

    Ok(())
}

pub fn use_python(version: &str, config: &mut Config) -> HuakResult<()> {
    let path = match python_paths()
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
        Some(it) => it,
        None => return Err(Error::PythonNotFoundError),
    };

    if let Ok(workspace) = config.workspace().as_mut() {
        match workspace.current_python_environment() {
            Ok(it) => std::fs::remove_dir_all(it.root)?,
            Err(Error::PythonEnvironmentNotFoundError) => (),
            Err(e) => return Err(e),
        };
    }

    let mut cmd = Command::new(path);
    cmd.args(["-m", "venv", default_virtual_environment_name()])
        .current_dir(&config.workspace_root);
    config.terminal.run_command(&mut cmd)
}

pub fn display_project_version(config: &mut Config) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let project = Project::new(workspace.root)?;

    config.terminal.print_custom(
        "version",
        project
            .manifest
            .version
            .unwrap_or("no version found".to_string()),
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

fn create_workspace<T: AsRef<Path>>(
    path: T,
    config: &Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    let root = path.as_ref();

    if (root.exists() && root != config.cwd)
        || (root == config.cwd && root.read_dir()?.count() > 0)
    {
        return Err(Error::DirectoryExists(root.to_path_buf()));
    }

    std::fs::create_dir(root)?;

    init_git(root, options)
}

fn init_git<T: AsRef<Path>>(
    path: T,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    let root = path.as_ref();
    if let Some(o) = options.as_ref() {
        if o.uses_git {
            if !root.join(".git").exists() {
                git::init(root)?;
            }
            let gitignore_path = root.join(".gitignore");
            if !gitignore_path.exists() {
                std::fs::write(gitignore_path, default_python_gitignore())?;
            }
        }
    }
    Ok(())
}

fn parse_installer_options(
    options: Option<&InstallOptions>,
) -> Option<PackageInstallerOptions> {
    options.map(|it| PackageInstallerOptions::Pip {
        args: it.args.clone(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{
        fs, sys::Terminal, test_resources_dir_path, Package, PyProjectToml,
        Verbosity,
    };
    use pep440_rs::Version as Version440;
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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let venv = PythonEnvironment::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        venv.uninstall_packages(
            &deps.iter().map(|item| item.as_str()).collect::<Vec<&str>>(),
            None,
            &mut terminal,
        )
        .unwrap();

        add_project_dependencies(&deps, &mut config, None).unwrap();

        let project = Project::new(config.workspace_root).unwrap();
        let ser_toml =
            PyProjectToml::new(dir.join("mock-project").join("pyproject.toml"))
                .unwrap();
        let dep = Dependency::from_str("ruff").unwrap();

        assert!(venv.contains_module("ruff").unwrap());
        assert!(project.contains_dependency(&dep).unwrap());
        assert!(deps.iter().all(|item| ser_toml
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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let venv = PythonEnvironment::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        venv.uninstall_packages(
            &deps.iter().map(|item| item.as_str()).collect::<Vec<&str>>(),
            None,
            &mut terminal,
        )
        .unwrap();

        add_project_optional_dependencies(&deps, group, &mut config, None)
            .unwrap();

        let project = Project::new(config.workspace_root).unwrap();
        let ser_toml =
            PyProjectToml::new(dir.join("mock-project").join("pyproject.toml"))
                .unwrap();
        let dep = Dependency::from_str("ruff").unwrap();

        assert!(venv.contains_module("ruff").unwrap());
        assert!(project.contains_optional_dependency(&dep, "dev").unwrap());
        assert!(deps.iter().all(|item| ser_toml
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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };

        build_project(&mut config, None).unwrap();
    }

    #[test]
    fn test_clean_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            test_resources_dir_path().join("mock-project"),
            dir.join("mock-project"),
        )
        .unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let options = Some(CleanOptions {
            include_pycache: true,
            include_compiled_bytecode: true,
        });

        clean_project(&mut config, options).unwrap();

        let dist = glob::glob(&format!(
            "{}",
            config.workspace_root.join("dist").join("*").display()
        ))
        .unwrap()
        .map(|item| item.unwrap())
        .collect::<Vec<_>>();
        let pycaches = glob::glob(&format!(
            "{}",
            config
                .workspace_root
                .join("**")
                .join("__pycache__")
                .display()
        ))
        .unwrap()
        .map(|item| item.unwrap())
        .collect::<Vec<_>>();
        let bytecode = glob::glob(&format!(
            "{}",
            config.workspace_root.join("**").join("*.pyc").display()
        ))
        .unwrap()
        .map(|item| item.unwrap())
        .collect::<Vec<_>>();

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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let project = Project::new(&config.workspace_root).unwrap();
        let fmt_filepath = project
            .root()
            .unwrap()
            .join("src")
            .join("mock_project")
            .join("fmt_me.py");
        let pre_fmt_str = r#"
def fn( ):
    pass"#;
        std::fs::write(&fmt_filepath, pre_fmt_str).unwrap();

        format_project(&mut config, None).unwrap();

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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };

        init_lib_project(&mut config, None).unwrap();

        let ser_toml =
            PyProjectToml::new(config.workspace_root.join("pyproject.toml"))
                .unwrap();
        let mut pyproject_toml =
            PyProjectToml::new(config.workspace_root.join("pyproject.toml"))
                .unwrap();
        pyproject_toml.set_project_name("mock-project".to_string());

        assert_eq!(
            ser_toml.to_string_pretty().unwrap(),
            pyproject_toml.to_string_pretty().unwrap()
        );
    }

    #[test]
    fn test_init_app_project() {
        let dir = tempdir().unwrap().into_path();
        std::fs::create_dir(dir.join("mock-project")).unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };

        init_app_project(&mut config, None).unwrap();

        let ser_toml =
            PyProjectToml::new(config.workspace_root.join("pyproject.toml"))
                .unwrap();
        let mut pyproject_toml = PyProjectToml::default();
        pyproject_toml.set_project_name("mock-project".to_string());

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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let venv = PythonEnvironment::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();
        venv.uninstall_packages(&["click"], None, &mut config.terminal)
            .unwrap();
        let package = Package {
            name: String::from("click"),
            canonical_name: String::from("click"),
            version: Version440::from_str("0.0.0").unwrap(),
        };
        let had_package = venv.contains_package(&package);

        install_project_dependencies(&mut config, None).unwrap();

        assert!(!had_package);
        assert!(venv.contains_package(&package));
    }

    #[test]
    fn test_install_project_optional_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let venv = PythonEnvironment::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();
        venv.uninstall_packages(&["pytest"], None, &mut config.terminal)
            .unwrap();
        let had_package = venv.contains_module("pytest").unwrap();

        install_project_optional_dependencies(
            &["dev".to_string()],
            &mut config,
            None,
        )
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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let options = Some(LintOptions {
            args: None,
            include_types: true,
            install_options: None,
        });

        lint_project(&mut config, options).unwrap();
    }

    #[test]
    fn test_fix_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let options = Some(LintOptions {
            args: Some(vec!["--fix".to_string()]),
            include_types: false,
            install_options: None,
        });
        let project = Project::new(&config.workspace_root).unwrap();
        let lint_fix_filepath = project
            .root()
            .unwrap()
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

        lint_project(&mut config, options).unwrap();

        let post_fix_str = std::fs::read_to_string(&lint_fix_filepath).unwrap();

        assert_eq!(post_fix_str, expected);
    }

    #[test]
    fn test_new_lib_project() {
        let dir = tempdir().unwrap().into_path();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };

        new_lib_project(&mut config, None).unwrap();

        let project = Project::new(config.workspace_root).unwrap();
        let test_file_filepath = project
            .root()
            .unwrap()
            .join("tests")
            .join("test_version.py");
        let test_file = std::fs::read_to_string(test_file_filepath).unwrap();
        let expected_test_file = r#"from mock_project import __version__


def test_version():
    __version__
"#;
        let init_file_filepath = project
            .root()
            .unwrap()
            .join("src")
            .join("mock_project")
            .join("__init__.py");
        let init_file = std::fs::read_to_string(init_file_filepath).unwrap();
        let expected_init_file = "__version__ = \"0.0.1\"
";

        assert!(project.manifest.scripts.is_none());
        assert_eq!(test_file, expected_test_file);
        assert_eq!(init_file, expected_init_file);
    }

    #[test]
    fn test_new_app_project() {
        let dir = tempdir().unwrap().into_path();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };

        new_app_project(&mut config, None).unwrap();

        let project = Project::new(config.workspace_root).unwrap();
        let main_file_filepath = project
            .root()
            .unwrap()
            .join("src")
            .join("mock_project")
            .join("main.py");
        let main_file = std::fs::read_to_string(main_file_filepath).unwrap();
        let expected_main_file = r#"def main():
    print("Hello, World!")


if __name__ == "__main__":
    main()
"#;

        assert_eq!(
            project.manifest.scripts.as_ref().unwrap()["mock-project"],
            format!("{}.main:main", "mock_project")
        );
        assert_eq!(main_file, expected_main_file);
    }

    #[test]
    fn test_remove_project_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let project = Project::new(&config.workspace_root).unwrap();
        let venv = PythonEnvironment::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();
        let package = Package {
            name: "click".to_string(),
            canonical_name: String::from("click"),
            version: Version440::from_str("8.1.3").unwrap(),
        };
        let dep = Dependency::from_str("click==8.1.3").unwrap();
        venv.install_packages(&[&dep], None, &mut config.terminal)
            .unwrap();
        let venv_had_package = venv.contains_package(&package);
        let toml_had_package = project.dependencies().unwrap().contains(&dep);

        remove_project_dependencies(&["click".to_string()], &mut config, None)
            .unwrap();

        let project = Project::new(&config.workspace_root).unwrap();
        let venv_contains_package = venv.contains_package(&package);
        let toml_contains_package =
            project.dependencies().unwrap().contains(&dep);
        venv.install_packages(&[&dep], None, &mut config.terminal)
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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let project = Project::new(&config.workspace_root).unwrap();
        let venv = PythonEnvironment::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();
        let package = Package {
            name: "black".to_string(),
            canonical_name: String::from("black"),
            version: Version440::from_str("22.8.0").unwrap(),
        };
        let dep = Dependency::from_str("black==22.8.0").unwrap();
        venv.uninstall_packages(&[package.name()], None, &mut config.terminal)
            .unwrap();
        venv.install_packages(&[&dep], None, &mut config.terminal)
            .unwrap();
        let venv_had_package = venv.contains_module(package.name()).unwrap();
        let toml_had_package = project
            .optional_dependencey_group("dev")
            .unwrap()
            .contains(&dep);

        remove_project_optional_dependencies(
            &["black".to_string()],
            "dev",
            &mut config,
            None,
        )
        .unwrap();

        let project = Project::new(&config.workspace_root).unwrap();
        let venv_contains_package =
            venv.contains_module(package.name()).unwrap();
        let toml_contains_package =
            project.dependencies().unwrap().contains(&dep);
        venv.uninstall_packages(&[package.name()], None, &mut config.terminal)
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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };
        let venv = PythonEnvironment::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();
        venv.uninstall_packages(&["black"], None, &mut config.terminal)
            .unwrap();
        let venv_had_package = venv.contains_module("black").unwrap();

        run_command_str("pip install black", &mut config).unwrap();

        let venv_contains_package = venv.contains_module("black").unwrap();
        venv.uninstall_packages(&["black"], None, &mut config.terminal)
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
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };

        update_project_dependencies(None, &mut config, None).unwrap();
    }

    #[test]
    fn test_update_project_optional_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };

        update_project_optional_dependencies(None, "dev", &mut config, None)
            .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_use_python() {
        let dir = tempdir().unwrap().into_path();
        let version = python_paths().max().unwrap().0.unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.to_path_buf(),
            cwd: dir,
            terminal,
        };
        use_python(&version.to_string(), &mut config).unwrap();
    }

    #[test]
    fn test_test_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let mut terminal = Terminal::new();
        terminal.set_verbosity(Verbosity::Quiet);
        let mut config = Config {
            workspace_root: dir.join("mock-project"),
            cwd: std::env::current_dir().unwrap(),
            terminal,
        };

        test_project(&mut config, None).unwrap();
    }
}
