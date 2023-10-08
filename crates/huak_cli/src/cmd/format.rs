use super::make_venv_command;
use huak_python_package_manager::{Config, Dependency, HuakResult, InstallOptions};
use std::{process::Command, str::FromStr};

pub struct FormatOptions {
    /// A values vector of format options typically used for passing on arguments.
    pub values: Option<Vec<String>>,
    pub install_options: InstallOptions,
}

pub fn format_project(config: &Config, options: &FormatOptions) -> HuakResult<()> {
    let workspace = config.workspace();
    let package = workspace.current_package()?;
    let mut metadata = workspace.current_local_metadata()?;
    let python_env = workspace.resolve_python_environment()?;

    // Install `ruff` and `black` if they aren't already installed.
    let format_deps = [
        Dependency::from_str("black")?,
        Dependency::from_str("ruff")?,
    ];

    let new_format_deps = format_deps
        .iter()
        .filter(|dep| !python_env.contains_module(dep.name()).unwrap_or_default())
        .collect::<Vec<_>>();

    if !new_format_deps.is_empty() {
        python_env.install_packages(&new_format_deps, &options.install_options, config)?;
    }

    // Add the installed `ruff` and `black` packages to the metadata file if not already there.
    let new_format_deps = format_deps
        .iter()
        .filter(|dep| {
            !metadata
                .metadata()
                .contains_dependency_any(dep)
                .unwrap_or_default()
        })
        .map(|dep| dep.name())
        .collect::<Vec<_>>();

    if !new_format_deps.is_empty() {
        for pkg in python_env
            .installed_packages()?
            .iter()
            .filter(|pkg| new_format_deps.contains(&pkg.name()))
        {
            metadata
                .metadata_mut()
                .add_optional_dependency(Dependency::from_str(&pkg.to_string())?, "dev");
        }
    }

    if package.metadata() != metadata.metadata() {
        metadata.write_file()?;
    }

    // Run `ruff` and `black` for formatting imports and the rest of the Python code in the workspace.
    let mut terminal = config.terminal();
    let mut cmd = Command::new(python_env.python_path());
    let mut ruff_cmd = Command::new(python_env.python_path());
    let mut ruff_args = vec!["-m", "ruff", "check", ".", "--select", "I001", "--fix"];
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
    ruff_cmd.args(ruff_args).current_dir(workspace.root());
    terminal.run_command(&mut ruff_cmd)?;
    cmd.args(args).current_dir(workspace.root());
    terminal.run_command(&mut cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::test_utils::test_resources_dir_path;
    use huak_python_package_manager::{
        copy_dir, initialize_venv, CopyDirOptions, TerminalOptions, Verbosity,
    };
    use tempfile::tempdir;

    #[test]
    fn test_format_project() {
        let dir = tempdir().unwrap();
        copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let workspace_root = dir.path().join("mock-project");
        let cwd = workspace_root.to_path_buf();
        let terminal_options = TerminalOptions {
            verbosity: Verbosity::Quiet,
            ..Default::default()
        };
        let config = Config {
            workspace_root,
            cwd,
            terminal_options,
        };
        let ws = config.workspace();
        initialize_venv(ws.root().join(".venv"), &ws.environment()).unwrap();
        let fmt_filepath = ws.root().join("src").join("mock_project").join("fmt_me.py");
        let pre_fmt_str = r#"
def fn( ):
    pass"#;
        std::fs::write(&fmt_filepath, pre_fmt_str).unwrap();
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
}
