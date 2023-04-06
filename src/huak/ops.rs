///! This module implements various operations to interact with valid workspaces
///! existing on a system.
///
use indexmap::IndexMap;
use std::{env::consts::OS, path::Path, process::Command, str::FromStr};
use termcolor::Color;

use crate::{
    dependency_iter,
    error::{Error, HuakResult},
    fs, git, importable_package_name, sys, Config, Dependency, Environment,
    InstallOptions, LocalMetdata, Metadata, PyProjectToml, PythonEnvironment,
    ToDepStr, WorkspaceOptions,
};

pub struct AddOptions {
    pub install_options: InstallOptions,
}
pub struct BuildOptions {
    /// An values vector of build options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}
pub struct FormatOptions {
    /// An values vector of format options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}

pub struct LintOptions {
    /// An values vector of lint options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub include_types: bool,
    pub install_options: InstallOptions,
}

pub struct RemoveOptions {
    pub install_options: InstallOptions,
}
pub struct PublishOptions {
    /// An values vector of publish options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}
pub struct TestOptions {
    /// An values vector of test options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}
#[derive(Clone)]
pub struct UpdateOptions {
    pub install_options: InstallOptions,
}
pub struct CleanOptions {
    pub include_pycache: bool,
    pub include_compiled_bytecode: bool,
}

pub fn activate_python_environment(config: &Config) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let python_env = workspace.current_python_environment()?;

    if python_env.active() {
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

    config.terminal().run_command(&mut cmd)
}

pub fn add_project_dependencies(
    dependencies: &[String],
    config: &Config,
    options: &AddOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    let deps = dependency_iter(dependencies)
        .filter(|dep| {
            !metadata
                .metadata
                .contains_dependency(dep)
                .unwrap_or_default()
        })
        .collect::<Vec<Dependency>>();
    if deps.is_empty() {
        return Ok(());
    }

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    python_env.install_packages(
        deps.into_iter(),
        &options.install_options,
        config,
    )?;

    let packages = python_env.installed_packages()?;
    for pkg in packages.iter().filter(|pkg| {
        deps.iter().any(|dep| {
            pkg.name() == dep.name() && dep.version_specifiers.is_none()
        })
    }) {
        let dep = Dependency::from_str(&pkg.to_dep_str())?;
        metadata.metadata.add_dependency(dep);
    }

    for dep in deps
        .into_iter()
        .filter(|dep| dep.version_specifiers.is_none())
    {
        metadata.metadata.add_dependency(dep);
    }

    if package.metadata != metadata.metadata {
        metadata.write_file()?;
    }

    Ok(())
}

pub fn add_project_optional_dependencies(
    dependencies: &[String],
    group: &str,
    config: &Config,
    options: &AddOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    let deps = dependency_iter(dependencies)
        .filter(|dep| {
            !metadata
                .metadata
                .contains_optional_dependency(dep, group)
                .unwrap_or_default()
        })
        .collect::<Vec<Dependency>>();
    if deps.is_empty() {
        return Ok(());
    };

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    python_env.install_packages(
        deps.into_iter(),
        &options.install_options,
        config,
    )?;

    let packages = python_env.installed_packages()?;
    for pkg in packages.iter().filter(|pkg| {
        deps.iter().any(|dep| {
            pkg.name() == dep.name() && dep.version_specifiers.is_none()
        })
    }) {
        let dep = Dependency::from_str(&pkg.to_dep_str())?;
        metadata.metadata.add_optional_dependency(dep, group);
    }

    for dep in deps
        .into_iter()
        .filter(|dep| dep.version_specifiers.is_none())
    {
        metadata.metadata.add_optional_dependency(dep, group);
    }

    if package.metadata != metadata.metadata {
        metadata.write_file()?;
    }

    Ok(())
}

pub fn build_project(
    config: &Config,
    options: &BuildOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let mut metadata = workspace.current_local_metadata()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let build_dep = Dependency::from_str("build")?;
    if !python_env.contains_module(build_dep.name())? {
        python_env.install_packages(
            [build_dep].into_iter(),
            &options.install_options,
            &config,
        )?;
    }

    if !metadata.metadata.contains_dependency_any(&build_dep)? {
        metadata.metadata.add_optional_dependency(build_dep, "dev");
        metadata.write_file();
    }

    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "build"];
    if let Some(it) = options.values.as_ref() {
        args.extend(it.iter().map(|item| item.as_str()));
    }
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(&workspace.root);

    config.terminal().run_command(&mut cmd)
}

pub fn clean_project(
    config: &Config,
    options: &CleanOptions,
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
    if options.include_pycache {
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
    if options.include_compiled_bytecode {
        let pattern =
            format!("{}", workspace.root.join("**").join("*.pyc").display());
        glob::glob(&pattern)?.for_each(|item| {
            if let Ok(it) = item {
                std::fs::remove_file(it).ok();
            }
        })
    }
    Ok(())
}

pub fn format_project(
    config: &Config,
    options: &FormatOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_package()?;
    let metadata = workspace.current_local_metadata()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let format_deps = [
        Dependency::from_str("black")?,
        Dependency::from_str("ruff")?,
    ];
    let new_format_deps = format_deps
        .into_iter()
        .filter(|item| {
            !python_env.contains_module(item.name()).unwrap_or_default()
        })
        .collect::<Vec<Dependency>>();
    if !new_format_deps.is_empty() {
        python_env.install_packages(
            new_format_deps.into_iter(),
            &options.install_options,
            config,
        )?;
    }

    let new_format_deps = format_deps
        .into_iter()
        .filter(|item| {
            !metadata
                .metadata
                .contains_dependency(item)
                .unwrap_or_default()
                && !metadata
                    .metadata
                    .contains_dependency_any(item)
                    .unwrap_or_default()
        })
        .collect::<Vec<Dependency>>();
    if !new_format_deps.is_empty() {
        for dep in new_format_deps {
            {
                metadata.metadata.add_optional_dependency(dep, "dev");
            }
        }
    }
    if package.metadata != metadata.metadata {
        metadata.write_file()?;
    }

    let mut terminal = config.terminal();
    let mut cmd = Command::new(python_env.python_path());
    let mut ruff_cmd = Command::new(python_env.python_path());
    let mut ruff_args =
        vec!["-m", "ruff", "check", ".", "--select", "I001", "--fix"];
    make_venv_command(&mut cmd, &python_env)?;
    make_venv_command(&mut ruff_cmd, &python_env)?;
    let mut args = vec!["-m", "black", "."];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(|item| item.as_str()));
        if v.contains(&"--check".to_string()) {
            terminal.print_warning(
                    "this check will exit early if imports aren't sorted (see https://github.com/cnpryer/huak/issues/510)",
                )?;
            ruff_args.retain(|item| *item != "--fix")
        }
    }
    ruff_cmd.args(ruff_args).current_dir(&workspace.root);
    terminal.run_command(&mut ruff_cmd)?;
    cmd.args(args).current_dir(&workspace.root);
    terminal.run_command(&mut cmd)
}

pub fn init_app_project(
    config: &Config,
    options: &WorkspaceOptions,
) -> HuakResult<()> {
    init_lib_project(config, options)?;

    let workspace = config.workspace()?;
    let mut metadata = workspace.current_local_metadata()?;

    let as_dep = Dependency::from_str(&metadata.metadata.project_name())?;
    let entry_point =
        default_entrypoint_string(importable_package_name(as_dep.name()));
    if let Some(scripts) = metadata.metadata.project.scripts.as_mut() {
        if !scripts.contains_key(as_dep.name()) {
            scripts.insert(as_dep.name().to_string(), entry_point);
        }
    } else {
        metadata.metadata.project.scripts =
            Some(IndexMap::from_iter([(as_dep.name(), entry_point)]));
    }

    metadata.write_file()
}

pub fn init_lib_project(
    config: &Config,
    options: &WorkspaceOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let mut metadata = match workspace.current_local_metadata() {
        Ok(_) => return Err(Error::MetadataFileFound),
        Err(_) => LocalMetdata {
            metadata: Metadata {
                project: PyProjectToml::default().project.unwrap(),
                tool: None,
            },
            path: workspace.root.join("pyproject.toml"),
        },
    };

    init_git(&config.workspace_root, options)?;

    let name = fs::last_path_component(&config.workspace_root)?;
    metadata.metadata.set_project_name(name);
    metadata.write_file()
}

pub fn install_project_dependencies(
    config: &Config,
    options: &InstallOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_package()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };

    let dependencies = match package.metadata.dependencies() {
        Some(it) => it,
        None => return Ok(()),
    };

    if dependencies.is_empty() {
        return Ok(());
    }

    python_env.install_packages(
        dependencies.iter().map(|req| Dependency::from(req)),
        options,
        config,
    )
}

pub fn install_project_optional_dependencies(
    groups: &[String],
    config: &Config,
    options: &InstallOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_local_metadata()?;

    let binding = Vec::new(); // TODO
    let mut dependencies = Vec::new();
    // If the group "all" is passed and isn't a valid optional dependency group
    // then install everything, disregarding other groups passed.
    if package.metadata.optional_dependencey_group("all").is_none()
        && groups.contains(&"all".to_string())
    {
        if let Some(deps) = package.metadata.optional_dependencies() {
            for (_, vals) in deps {
                dependencies
                    .extend(vals.iter().map(|req| Dependency::from(req)));
            }
        }
    } else {
        groups.iter().for_each(|item| {
            package
                .metadata
                .optional_dependencey_group(item)
                .unwrap_or(&binding)
                .iter()
                .for_each(|req| {
                    dependencies.push(Dependency::from(req));
                });
        })
    }
    dependencies.dedup();

    if dependencies.is_empty() {
        return Ok(());
    }

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    python_env.install_packages(dependencies.into_iter(), options, config)
}

pub fn lint_project(config: &Config, options: &LintOptions) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let project = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };

    let ruff_dep = Dependency::from_str("ruff")?;
    if !python_env.contains_module("ruff")? {
        python_env.install_packages(
            [ruff_dep].into_iter(),
            &options.install_options,
            config,
        )?;
    }

    if !metadata.metadata.contains_dependency_any(&ruff_dep)? {
        metadata.metadata.add_optional_dependency(ruff_dep, "dev");
    }

    let mut terminal = config.terminal();
    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "ruff", "check", "."];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(|item| item.as_str()));
    }
    if options.include_types {
        let mypy_dep = Dependency::from_str("mypy")?;
        if !python_env.contains_module("mypy")? {
            python_env.install_packages(
                [mypy_dep],
                &options.install_options,
                config,
            )?;
        }
        if !metadata.metadata.contains_dependency_any(&mypy_dep)? {
            metadata.metadata.add_optional_dependency(mypy_dep, "dev");
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
        terminal.run_command(&mut mypy_cmd)?;
    }
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(&workspace.root);
    terminal.run_command(&mut cmd)?;

    if project.metadata != metadata.metadata {
        metadata.write_file()?;
    }

    Ok(())
}

pub fn list_python(config: &Config) -> HuakResult<()> {
    let env = Environment::new();
    env.python_paths().enumerate().for_each(|(i, path)| {
        config
            .terminal()
            .print_custom(i + 1, path.display(), Color::Blue, false)
            .ok();
    });

    Ok(())
}

pub fn new_app_project(
    config: &Config,
    options: &WorkspaceOptions,
) -> HuakResult<()> {
    new_lib_project(config, options)?;

    let workspace = config.workspace()?;
    let mut metadata = workspace.current_local_metadata()?;

    let name = fs::last_path_component(workspace.root.as_path())?;
    let as_dep = Dependency::from_str(&name)?;
    metadata.metadata.set_project_name(name);

    let src_path = workspace.root.join("src");
    let importable_name = importable_package_name(as_dep.name())?;
    std::fs::write(
        src_path.join(importable_name).join("main.py"),
        default_main_file_contents(),
    )?;
    let entry_point = default_entrypoint_string(&importable_name);
    metadata.metadata.project.scripts =
        Some(IndexMap::from_iter([(as_dep.name(), entry_point)]));

    metadata.write_file()
}

pub fn new_lib_project(
    config: &Config,
    options: &WorkspaceOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let mut metadata = match workspace.current_local_metadata() {
        Ok(_) => return Err(Error::ProjectFound),
        Err(_) => LocalMetdata {
            metadata: Metadata {
                project: PyProjectToml::default().project.unwrap(),
                tool: None,
            },
            path: workspace.root.join("pyproject.toml"),
        },
    };

    create_workspace(&config.workspace_root, config, options)?;

    let name = &fs::last_path_component(&config.workspace_root)?;
    metadata.metadata.set_project_name(name.to_string());
    metadata.write_file()?;

    let as_dep = Dependency::from_str(name)?;
    let src_path = config.workspace_root.join("src");
    let importable_name = importable_package_name(as_dep.name())?;
    std::fs::create_dir_all(src_path.join(importable_name))?;
    std::fs::create_dir_all(config.workspace_root.join("tests"))?;
    std::fs::write(
        src_path.join(importable_name).join("__init__.py"),
        default_init_file_contents(),
    )?;
    std::fs::write(
        config.workspace_root.join("tests").join("test_version.py"),
        default_test_file_contents(&importable_name),
    )
    .map_err(Error::IOError)
}

pub fn publish_project(
    config: &Config,
    options: &PublishOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let mut metadata = workspace.current_local_metadata()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let pub_dep = Dependency::from_str("twine")?;
    if !python_env.contains_module(pub_dep.name())? {
        python_env.install_packages(
            [pub_dep],
            &options.install_options,
            config,
        )?;
    }

    if !metadata.metadata.contains_dependency_any(&pub_dep)? {
        metadata.metadata.add_optional_dependency(pub_dep, "dev");
        metadata.write_file()?;
    }

    let mut cmd = Command::new(python_env.python_path());
    let mut args = vec!["-m", "twine", "upload", "dist/*"];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(|item| item.as_str()));
    }
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args(args).current_dir(&workspace.root);
    config.terminal().run_command(&mut cmd)
}

pub fn remove_project_dependencies(
    dependencies: &[String],
    config: &Config,
    options: &RemoveOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    let deps = dependency_iter(dependencies)
        .filter(|item| {
            metadata
                .metadata
                .contains_dependency(item)
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    if deps.is_empty() {
        return Ok(());
    }

    for dep in &deps {
        metadata.metadata.remove_dependency(dep);
    }

    if package.metadata != metadata.metadata {
        metadata.write_file()?;
    }

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => return Ok(()),
        Err(e) => return Err(e),
    };
    python_env.uninstall_packages(deps, &options.install_options, config)
}

pub fn remove_project_optional_dependencies(
    dependencies: &[String],
    group: &str,
    config: &Config,
    options: &RemoveOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    if metadata
        .metadata
        .optional_dependencey_group(group)
        .is_none()
    {
        return Ok(());
    }

    let deps: Vec<Dependency> = dependency_iter(dependencies)
        .filter(|item| {
            metadata
                .metadata
                .contains_optional_dependency(item, group)
                .unwrap_or_default()
        })
        .collect();
    if deps.is_empty() {
        return Ok(());
    }

    for dep in &deps {
        metadata.metadata.remove_optional_dependency(dep, group);
    }

    if package.metadata != metadata.metadata {
        metadata.write_file()?;
    }

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => return Ok(()),
        Err(e) => return Err(e),
    };
    python_env.uninstall_packages(deps, &options.install_options, config)
}

pub fn run_command_str(command: &str, config: &Config) -> HuakResult<()> {
    let mut workspace = config.workspace()?;
    let python_env = workspace.current_python_environment()?;

    let mut cmd = Command::new(sys::shell_name()?);
    let flag = match OS {
        "windows" => "/C",
        _ => "-c",
    };
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args([flag, command]).current_dir(&workspace.root);
    config.terminal().run_command(&mut cmd)
}

pub fn test_project(config: &Config, options: &TestOptions) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let mut metadata = workspace.current_local_metadata()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };
    let test_dep = Dependency::from_str("pytest")?;
    if !python_env.contains_module(test_dep.name())? {
        python_env.install_packages(
            [test_dep],
            &options.install_options,
            config,
        )?;
    }

    if !metadata.metadata.contains_dependency_any(&test_dep)? {
        metadata.metadata.add_optional_dependency(test_dep, "dev");
        metadata.write_file()?;
    }

    let mut cmd = Command::new(python_env.python_path());
    make_venv_command(&mut cmd, &python_env)?;
    let python_path = if workspace.root.join("src").exists() {
        workspace.root.join("src")
    } else {
        workspace.root.clone()
    };
    let mut args = vec!["-m", "pytest"];
    if let Some(v) = options.values.as_ref() {
        args.extend(v.iter().map(|item| item.as_str()));
    }
    cmd.args(args).env("PYTHONPATH", python_path);
    config.terminal().run_command(&mut cmd)
}

pub fn update_project_dependencies(
    dependencies: Option<Vec<String>>,
    config: &Config,
    options: &UpdateOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };

    if let Some(it) = dependencies.as_ref() {
        let deps = dependency_iter(it)
            .filter_map(|item| {
                if metadata
                    .metadata
                    .contains_dependency(&item)
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
        python_env.update_packages(deps, &options.install_options, config)?;
    } else if let Some(deps) = metadata.metadata.dependencies() {
        python_env.update_packages(
            deps.iter().map(|req| Dependency::from(req)),
            &options.install_options,
            config,
        )?;
    }

    let packages = python_env.installed_packages()?;
    for pkg in packages {
        let dep = Dependency::from_str(&pkg.to_dep_str())?;
        if metadata.metadata.contains_dependency(&dep)? {
            metadata.metadata.remove_dependency(&dep);
            metadata.metadata.add_dependency(dep);
        }

        if package.metadata != metadata.metadata {
            metadata.write_file()?;
        }
    }

    Ok(())
}

pub fn update_project_optional_dependencies(
    dependencies: Option<Vec<String>>,
    group: &str,
    config: &Config,
    options: &UpdateOptions,
) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;

    let python_env = match workspace.current_python_environment() {
        Ok(it) => it,
        Err(Error::PythonEnvironmentNotFound) => {
            workspace.new_python_environment()?
        }
        Err(e) => return Err(e),
    };

    if let Some(it) = dependencies.as_ref() {
        let deps = dependency_iter(it)
            .filter_map(|item| {
                if metadata
                    .metadata
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
        python_env.update_packages(deps, &options.install_options, config)?;
    } else {
        let mut deps = Vec::new();
        let binding = Vec::new(); // TODO

        // If the group "all" is passed and isn't a valid optional dependency group
        // then install everything, disregarding other groups passed.
        if metadata
            .metadata
            .optional_dependencey_group("all")
            .is_none()
            && group == "all"
        {
            if let Some(it) = metadata.metadata.optional_dependencies() {
                for (_, vals) in it {
                    deps.extend(vals.iter().map(|req| Dependency::from(req)));
                }
            }
        } else {
            metadata
                .metadata
                .optional_dependencey_group(group)
                .unwrap_or(&binding)
                .iter()
                .for_each(|req| {
                    deps.push(Dependency::from(req));
                });
        }

        deps.dedup();
        python_env.update_packages(deps, &options.install_options, config)?;
    }

    let packages = python_env.installed_packages()?;
    let mut groups = Vec::new();

    if group == "all"
        && metadata
            .metadata
            .optional_dependencey_group("all")
            .is_none()
    {
        if let Some(it) = metadata.metadata.optional_dependencies() {
            groups.extend(it.keys().map(|key| key.to_string()));
        }
    } else {
        groups.push(group.to_string());
    }

    for pkg in packages {
        for g in groups.iter() {
            let dep = Dependency::from_str(&pkg.to_dep_str())?;
            if metadata.metadata.contains_optional_dependency(&dep, g)? {
                metadata.metadata.remove_optional_dependency(&dep, g);
                metadata.metadata.add_optional_dependency(dep, g);
            }
        }
    }

    if package.metadata != metadata.metadata {
        metadata.write_file()?;
    }

    Ok(())
}

pub fn use_python(version: &str, config: &Config) -> HuakResult<()> {
    let env = Environment::new();
    let interpreters = env.resolve_python_interpreters();

    let path = match interpreters
        .interpreters
        .iter()
        .find(|py| py.version.to_string() == version)
        .map(|py| py.path)
    {
        Some(it) => it,
        None => return Err(Error::PythonNotFound),
    };

    if let Ok(workspace) = config.workspace() {
        match workspace.current_python_environment() {
            Ok(it) => std::fs::remove_dir_all(it.root)?,
            Err(Error::PythonEnvironmentNotFound) => (),
            Err(e) => return Err(e),
        };
    }

    let mut cmd = Command::new(path);
    cmd.args(["-m", "venv", ".venv"])
        .current_dir(&config.workspace_root);
    config.terminal().run_command(&mut cmd)
}

pub fn display_project_version(config: &Config) -> HuakResult<()> {
    let workspace = config.workspace()?;
    let package = workspace.current_package()?;

    let version = match package.metadata.project_version() {
        Some(it) => it,
        None => return Err(Error::PackageVersionNotFound),
    };

    config
        .terminal()
        .print_custom("version", version, Color::Green, false)
}

fn make_venv_command(
    cmd: &Command,
    venv: &PythonEnvironment,
) -> HuakResult<()> {
    let mut paths = crate::env_path_values().unwrap_or(Vec::new());

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
    options: &WorkspaceOptions,
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
    options: &WorkspaceOptions,
) -> HuakResult<()> {
    let root = path.as_ref();
    if options.uses_git {
        if !root.join(".git").exists() {
            git::init(root)?;
        }
        let gitignore_path = root.join(".gitignore");
        if !gitignore_path.exists() {
            std::fs::write(gitignore_path, default_python_gitignore())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{
        fs,
        sys::{TerminalOptions, Verbosity},
        test_resources_dir_path, Package, PyProjectToml,
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
        let deps = [Dependency::from_str("ruff").unwrap()];
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let ws = config.workspace().unwrap();
        let package = ws.current_package().unwrap();
        let venv = PythonEnvironment::new(cwd.join(".venv")).unwrap();
        let options = AddOptions {
            install_options: InstallOptions { values: None },
        };
        venv.uninstall_packages(deps, &options.install_options, &config)
            .unwrap();

        add_project_dependencies(&[String::from("ruff")], &config, &options)
            .unwrap();

        let dep = Dependency::from_str("ruff").unwrap();

        assert!(venv.contains_module("ruff").unwrap());
        assert!(package.metadata.contains_dependency(&dep).unwrap());
    }

    #[test]
    fn test_add_optional_project_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let deps = [Dependency::from_str("ruff").unwrap()];
        let group = "dev";
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let ws = config.workspace().unwrap();
        let package = ws.current_package().unwrap();
        let venv = PythonEnvironment::new(cwd.join(".venv")).unwrap();
        let options = AddOptions {
            install_options: InstallOptions { values: None },
        };
        venv.uninstall_packages(deps, &options.install_options, &config)
            .unwrap();

        add_project_optional_dependencies(
            &[String::from("ruff")],
            group,
            &config,
            &options,
        )
        .unwrap();

        let dep = Dependency::from_str("ruff").unwrap();

        assert!(venv.contains_module("ruff").unwrap());
        assert!(package
            .metadata
            .contains_optional_dependency(&dep, "dev")
            .unwrap());
    }

    #[test]
    fn test_build_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = BuildOptions {
            values: None,
            install_options: InstallOptions { values: None },
        };

        build_project(&config, &options).unwrap();
    }

    #[test]
    fn test_clean_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            test_resources_dir_path().join("mock-project"),
            dir.join("mock-project"),
        )
        .unwrap();
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = CleanOptions {
            include_pycache: true,
            include_compiled_bytecode: true,
        };

        clean_project(&config, &options).unwrap();

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
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let ws = config.workspace().unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let fmt_filepath = metadata
            .path
            .parent()
            .unwrap()
            .join("src")
            .join("mock_project")
            .join("fmt_me.py");
        let pre_fmt_str = r#"
def fn( ):
    pass"#;
        std::fs::write(&fmt_filepath, pre_fmt_str).unwrap();
        let options = FormatOptions {
            values: None,
            install_options: InstallOptions { values: None },
        };
        let options = FormatOptions {
            values: None,
            install_options: InstallOptions { values: None },
        };

        format_project(&config, &options).unwrap();

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
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = WorkspaceOptions { uses_git: false };

        init_lib_project(&config, &options).unwrap();

        let ws = config.workspace().unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let mut pyproject_toml = PyProjectToml::default();
        pyproject_toml.project.unwrap().name = String::from("mock-project");

        assert_eq!(
            metadata.to_string_pretty().unwrap(),
            toml::ser::to_string_pretty(&pyproject_toml).unwrap()
        );
    }

    #[test]
    fn test_init_app_project() {
        let dir = tempdir().unwrap().into_path();
        std::fs::create_dir(dir.join("mock-project")).unwrap();
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = WorkspaceOptions { uses_git: false };

        init_app_project(&config, &options).unwrap();

        let ws = config.workspace().unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let mut pyproject_toml = PyProjectToml::default();
        pyproject_toml.project.unwrap().name = String::from("mock-project");

        assert_eq!(
            metadata.to_string_pretty().unwrap(),
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
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = InstallOptions { values: None };
        let venv = PythonEnvironment::new(cwd.join(".venv")).unwrap();
        let test_package = Package::from_str("click").unwrap();
        venv.uninstall_packages([test_package], &options, &config)
            .unwrap();
        let had_package = venv.contains_package(&test_package);

        install_project_dependencies(&config, &options).unwrap();

        assert!(!had_package);
        assert!(venv.contains_package(&test_package));
    }

    #[test]
    fn test_install_project_optional_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = InstallOptions { values: None };
        let venv = PythonEnvironment::new(cwd.join(".venv")).unwrap();
        let test_package = Package::from_str("pytest").unwrap();
        venv.uninstall_packages([test_package], &options, &config)
            .unwrap();
        let had_package = venv.contains_module("pytest").unwrap();

        install_project_optional_dependencies(
            &[String::from("dev")],
            &config,
            &options,
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
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = LintOptions {
            values: None,
            include_types: true,
            install_options: InstallOptions { values: None },
        };

        lint_project(&config, &options).unwrap();
    }

    #[test]
    fn test_fix_project() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let ws = config.workspace().unwrap();
        let options = LintOptions {
            values: None,
            include_types: true,
            install_options: InstallOptions { values: None },
        };
        let metadata = ws.current_local_metadata().unwrap();
        let lint_fix_filepath = metadata
            .path
            .parent()
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

        lint_project(&config, &options).unwrap();

        let post_fix_str = std::fs::read_to_string(&lint_fix_filepath).unwrap();

        assert_eq!(post_fix_str, expected);
    }

    #[test]
    fn test_new_lib_project() {
        let dir = tempdir().unwrap().into_path();
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = WorkspaceOptions { uses_git: false };

        new_lib_project(&config, &options).unwrap();

        let ws = config.workspace().unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let test_file_filepath = metadata
            .path
            .parent()
            .unwrap()
            .join("tests")
            .join("test_version.py");
        let test_file = std::fs::read_to_string(test_file_filepath).unwrap();
        let expected_test_file = r#"from mock_project import __version__


def test_version():
    __version__
"#;
        let init_file_filepath = metadata
            .path
            .parent()
            .unwrap()
            .join("src")
            .join("mock_project")
            .join("__init__.py");
        let init_file = std::fs::read_to_string(init_file_filepath).unwrap();
        let expected_init_file = "__version__ = \"0.0.1\"
";

        assert!(metadata.metadata.project.scripts.is_none());
        assert_eq!(test_file, expected_test_file);
        assert_eq!(init_file, expected_init_file);
    }

    #[test]
    fn test_new_app_project() {
        let dir = tempdir().unwrap().into_path();
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = WorkspaceOptions { uses_git: false };

        new_app_project(&config, &options).unwrap();

        let ws = config.workspace().unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let main_file_filepath = metadata
            .path
            .parent()
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
            metadata.metadata.project.scripts.as_ref().unwrap()["mock-project"],
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
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = RemoveOptions {
            install_options: InstallOptions { values: None },
        };
        let ws = config.workspace().unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let venv = PythonEnvironment::new(cwd.join(".venv")).unwrap();
        let test_package = Package::from_str("click==8.1.3").unwrap();
        let test_dep = Dependency::from_str("click==8.1.3").unwrap();
        venv.install_packages([test_dep], &options.install_options, &config)
            .unwrap();
        let venv_had_package = venv.contains_package(&test_package);
        let toml_had_package = metadata
            .metadata
            .dependencies()
            .unwrap()
            .contains(&test_dep.requirement);

        remove_project_dependencies(&["click".to_string()], &config, &options)
            .unwrap();

        let ws = config.workspace().unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let venv_contains_package = venv.contains_package(&test_package);
        let toml_contains_package = metadata
            .metadata
            .dependencies()
            .unwrap()
            .contains(&test_dep.requirement);
        venv.install_packages([test_dep], &options.install_options, &config)
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
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = RemoveOptions {
            install_options: InstallOptions { values: None },
        };
        let ws = config.workspace().unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let venv = PythonEnvironment::new(cwd.join(".venv")).unwrap();
        let test_package = Package::from_str("black==22.8.0").unwrap();
        let test_dep = Dependency::from_str("black==22.8.0").unwrap();
        venv.uninstall_packages(
            [test_package],
            &options.install_options,
            &config,
        )
        .unwrap();
        venv.install_packages([test_dep], &options.install_options, &config)
            .unwrap();
        let venv_had_package =
            venv.contains_module(test_package.name()).unwrap();
        let toml_had_package = metadata
            .metadata
            .optional_dependencey_group("dev")
            .unwrap()
            .contains(&test_dep.requirement);

        remove_project_optional_dependencies(
            &["black".to_string()],
            "dev",
            &config,
            &options,
        )
        .unwrap();

        let ws = config.workspace().unwrap();
        let metadata = ws.current_local_metadata().unwrap();
        let venv_contains_package = venv
            .contains_module(metadata.metadata.project_name())
            .unwrap();
        let toml_contains_package = metadata
            .metadata
            .dependencies()
            .unwrap()
            .contains(&test_dep.requirement);
        venv.uninstall_packages(
            [test_package],
            &options.install_options,
            &config,
        )
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
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = InstallOptions { values: None };
        let venv = PythonEnvironment::new(cwd.join(".venv")).unwrap();
        let test_package = Package::from_str("black").unwrap();
        venv.uninstall_packages([test_package], &options, &config)
            .unwrap();
        let venv_had_package = venv.contains_module("black").unwrap();

        run_command_str("pip install black", &config).unwrap();

        let venv_contains_package = venv.contains_module("black").unwrap();
        venv.uninstall_packages([test_package], &options, &config)
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
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = UpdateOptions {
            install_options: InstallOptions { values: None },
        };

        update_project_dependencies(None, &config, &options).unwrap();
    }

    #[test]
    fn test_update_project_optional_dependencies() {
        let dir = tempdir().unwrap().into_path();
        fs::copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.join("mock-project"),
        )
        .unwrap();
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = UpdateOptions {
            install_options: InstallOptions { values: None },
        };

        update_project_optional_dependencies(None, "dev", &config, &options)
            .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_use_python() {
        let dir = tempdir().unwrap().into_path();
        let env = Environment::new();
        let version =
            env.resolve_python_interpreters().latest().unwrap().version;
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);

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
        let root = dir.join("mock-project");
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = TestOptions {
            values: None,
            install_options: InstallOptions { values: None },
        };

        test_project(&config, &options).unwrap();
    }

    fn test_config<T: AsRef<Path>>(
        root: T,
        cwd: T,
        verbosity: Verbosity,
    ) -> Config {
        let config = Config {
            workspace_root: root.as_ref().to_path_buf(),
            cwd: cwd.as_ref().to_path_buf(),
            terminal_options: TerminalOptions { verbosity },
        };

        config
    }
}
