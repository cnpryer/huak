use crate::error::{CliResult, Error};
use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{self, Shell};
use huak::{
    ops::{
        self, CleanOptions, OperationConfig, TerminalOptions, WorkspaceOptions,
    },
    Error as HuakError, Verbosity,
};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

/// A Python package manager written in Rust inspired by Cargo.
#[derive(Parser)]
#[command(version, author, about, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

// List of commands.
#[derive(Subcommand)]
pub enum Commands {
    /// Activate the project's virtual environment.
    Activate,
    /// Add a dependency to the existing project.
    Add {
        dependency: String,
        /// Adds an optional dependency group.
        #[arg(long)]
        group: Option<String>,
        /// Pass trailing arguments with `--`.
        #[arg(trailing_var_arg = true)]
        trailing: Option<Vec<String>>,
    },
    /// Check for vulnerable dependencies and license compatibility*.
    Audit,
    /// Build tarball and wheel for the project.
    Build {
        /// Pass trailing arguments with `--`.
        #[arg(trailing_var_arg = true)]
        trailing: Option<Vec<String>>,
    },
    /// Interact with the configuration of huak.
    Config {
        #[command(subcommand)]
        command: Config,
    },
    /// Remove tarball and wheel from the built project.
    Clean {
        #[arg(long, required = false)]
        /// Remove all .pyc files.
        include_compiled_bytecode: bool,
        #[arg(long, required = false)]
        /// Remove all __pycache__ directories.
        include_pycache: bool,
    },
    /// Generates documentation for the project*.
    Doc {
        #[arg(long)]
        check: bool,
    },
    /// Auto-fix fixable lint conflicts
    Fix {
        /// Pass trailing arguments with `--`.
        #[arg(trailing_var_arg = true)]
        trailing: Option<Vec<String>>,
    },
    /// Format the project's Python code.
    Fmt {
        /// Check if Python code is formatted.
        #[arg(long)]
        check: bool,
        /// Pass trailing arguments with `--`.
        #[arg(trailing_var_arg = true)]
        trailing: Option<Vec<String>>,
    },
    /// Initialize the existing project.
    Init {
        /// Use a application template [default].
        #[arg(long, conflicts_with = "lib")]
        app: bool,
        /// Use a library template.
        #[arg(long, conflicts_with = "app")]
        lib: bool,
    },
    /// Install the dependencies of an existing project.
    Install {
        /// Install optional dependency groups
        #[arg(long, num_args = 1..)]
        groups: Option<Vec<String>>,
        /// Pass trailing arguments with `--`.
        #[arg(trailing_var_arg = true)]
        trailing: Option<Vec<String>>,
    },
    /// Lint the project's Python code.
    Lint {
        /// Address any fixable lints.
        #[arg(long, required = false)]
        fix: bool,
        /// Pass trailing arguments with `--`.
        #[arg(trailing_var_arg = true)]
        trailing: Option<Vec<String>>,
    },
    /// Create a new project at <path>.
    New {
        /// Use a application template [default].
        #[arg(long, conflicts_with = "lib")]
        app: bool,
        /// Use a library template.
        #[arg(long, conflicts_with = "app")]
        lib: bool,
        /// Path and name of the python package
        path: String,
        /// Don't initialize VCS in the new project
        #[arg(long)]
        no_vcs: bool,
    },
    /// Builds and uploads current project to a registry.
    Publish {
        /// Pass trailing arguments with `--`.
        #[arg(trailing_var_arg = true)]
        trailing: Option<Vec<String>>,
    },
    /// Remove a dependency from the project.
    Remove {
        dependency: String,
        /// Remove from optional dependency group
        #[arg(long, num_args = 1)]
        group: Option<String>,
    },
    /// Run a command within the project's environment context.
    Run {
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Test the project's Python code.
    Test {
        /// Pass trailing arguments with `--`.
        #[arg(trailing_var_arg = true)]
        trailing: Option<Vec<String>>,
    },
    /// Update dependencies added to the project*.
    Update {
        #[arg(default_value = "*")]
        dependency: String,
    },
    /// Display the version of the project.
    Version,
}

// Command gating for Huak.
impl Cli {
    pub fn run(self) -> CliResult<()> {
        match self.command {
            Commands::Config { command } => config(command, self.quiet),
            Commands::Activate => activate(self.quiet),
            Commands::Add {
                dependency,
                group,
                trailing,
            } => add(dependency, group, trailing, self.quiet),
            Commands::Audit => audit(self.quiet),
            Commands::Build { trailing } => build(trailing, self.quiet),
            Commands::Clean {
                include_compiled_bytecode,
                include_pycache,
            } => clean(include_compiled_bytecode, include_pycache, self.quiet),
            Commands::Doc { check } => doc(check, self.quiet),
            Commands::Fix { trailing } => fix(trailing, self.quiet),
            Commands::Fmt { check, trailing } => {
                fmt(check, trailing, self.quiet)
            }
            Commands::Init { app, lib } => init(app, lib, self.quiet),
            Commands::Install { groups, trailing } => {
                install(groups, trailing, self.quiet)
            }
            Commands::Lint { fix, trailing } => lint(fix, trailing, self.quiet),
            Commands::New {
                path,
                app,
                lib,
                no_vcs,
            } => new(path, app, lib, no_vcs, self.quiet),
            Commands::Publish { trailing } => publish(trailing, self.quiet),
            Commands::Remove { dependency, group } => {
                remove(dependency, group, self.quiet)
            }
            Commands::Run { command } => run(command, self.quiet),
            Commands::Test { trailing } => test(trailing, self.quiet),
            Commands::Update { dependency } => update(dependency, self.quiet),
            Commands::Version => version(self.quiet),
        }
    }
}

fn activate(_quiet: bool) -> CliResult<()> {
    todo!()
}

fn add(
    dependency: String,
    group: Option<String>,
    trailing: Option<Vec<String>>,
    quiet: bool,
) -> CliResult<()> {
    let config = init_config(trailing, quiet)?;
    let deps = [dependency.as_str()];
    match group.as_ref() {
        Some(it) => ops::add_project_optional_dependencies(&deps, it, &config),
        None => ops::add_project_dependencies(&deps, &config),
    }
    .map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn audit(_quiet: bool) -> CliResult<()> {
    todo!()
}

fn build(trailing: Option<Vec<String>>, quiet: bool) -> CliResult<()> {
    let config = init_config(trailing, quiet)?;
    ops::build_project(&config).map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn clean(
    include_compiled_bytecode: bool,
    include_pycache: bool,
    quiet: bool,
) -> CliResult<()> {
    let mut config = init_config(None, quiet)?;
    config.clean_options = Some(CleanOptions {
        include_compiled_bytecode,
        include_pycache,
    });
    ops::clean_project(&config).map_err(|e| Error::new(e, ExitCode::FAILURE))
}

/// Prints the script to stdout and a way to add the script to the shell init file to stderr. This
/// way if the user runs completion <shell> > completion.sh only the stdout will be redirected into
/// completion.sh.
fn config(command: Config, _quiet: bool) -> CliResult<()> {
    match command {
        Config::Completion {
            shell,
            install,
            uninstall,
        } => {
            if (install || uninstall) && shell.is_none() {
                return Err(Error::new(
                    HuakError::HuakConfigurationError(
                        "no shell provided".to_string(),
                    ),
                    ExitCode::FAILURE,
                ));
            }
            if install {
                run_with_install(shell)?;
            } else if uninstall {
                run_with_uninstall(shell)?;
            } else {
                generate_shell_completion_script()
            }
        }
    };

    Ok(())
}

fn doc(_check: bool, _quiet: bool) -> CliResult<()> {
    todo!()
}

fn fix(trailing: Option<Vec<String>>, quiet: bool) -> CliResult<()> {
    let mut config = init_config(trailing, quiet)?;
    if let Some(it) = config.trailing_command_parts.as_mut() {
        it.push("--fix".to_string());
    }
    ops::lint_project(&config).map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn fmt(
    check: bool,
    trailing: Option<Vec<String>>,
    quiet: bool,
) -> CliResult<()> {
    let mut config = init_config(trailing, quiet)?;
    if let Some(it) = config.trailing_command_parts.as_mut() {
        if check {
            it.push("--check".to_string());
        }
    }
    ops::format_project(&config).map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn init(app: bool, _lib: bool, quiet: bool) -> CliResult<()> {
    let config = init_config(None, quiet)?;
    let res = if app {
        ops::init_app_project(&config)
    } else {
        ops::init_lib_project(&config)
    };
    res.map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn install(
    groups: Option<Vec<String>>,
    trailing: Option<Vec<String>>,
    quiet: bool,
) -> CliResult<()> {
    let config = init_config(trailing, quiet)?;
    if let Some(it) = groups {
        for group in &it {
            match ops::install_project_optional_dependencies(group, &config) {
                Ok(_) => (),
                Err(e) => return Err(Error::new(e, ExitCode::FAILURE)),
            }
        }
        Ok(())
    } else {
        ops::install_project_dependencies(&config)
            .map_err(|e| Error::new(e, ExitCode::FAILURE))
    }
}

fn lint(
    fix: bool,
    trailing: Option<Vec<String>>,
    quiet: bool,
) -> CliResult<()> {
    let mut config = init_config(trailing, quiet)?;
    if fix {
        if let Some(it) = config.trailing_command_parts.as_mut() {
            it.push("--fix".to_string());
        }
    }
    ops::lint_project(&config).map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn new(
    path: String,
    app: bool,
    _lib: bool,
    no_vcs: bool,
    quiet: bool,
) -> CliResult<()> {
    let mut config = init_config(None, quiet)?;
    if let Some(it) = PathBuf::from(path).parent() {
        config.workspace_root = it.to_path_buf();
    } else {
        return Err(Error::new(
            HuakError::ProjectRootMissingError,
            ExitCode::FAILURE,
        ));
    }
    config.workspace_options = Some(WorkspaceOptions { uses_git: !no_vcs });
    let res = if app {
        ops::new_lib_project(&config)
    } else {
        ops::new_app_project(&config)
    };
    res.map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn publish(trailing: Option<Vec<String>>, quiet: bool) -> CliResult<()> {
    let config = init_config(trailing, quiet)?;
    ops::publish_project(&config).map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn remove(
    dependency: String,
    group: Option<String>,
    quiet: bool,
) -> CliResult<()> {
    let config = init_config(None, quiet)?;
    let deps = [dependency.as_str()];
    match group.as_ref() {
        Some(it) => {
            ops::remove_project_optional_dependencies(&deps, it, &config)
        }
        None => ops::remove_project_dependencies(&deps, &config),
    }
    .map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn run(command: Vec<String>, quiet: bool) -> CliResult<()> {
    let config = init_config(None, quiet)?;
    ops::run_command_str(&command.join(" "), &config)
        .map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn test(trailing: Option<Vec<String>>, quiet: bool) -> CliResult<()> {
    let config = init_config(trailing, quiet)?;
    ops::test_project(&config).map_err(|e| Error::new(e, ExitCode::FAILURE))
}

fn update(_dependency: String, _quiet: bool) -> CliResult<()> {
    todo!()
}

fn version(quiet: bool) -> CliResult<()> {
    let config = init_config(None, quiet)?;
    ops::display_project_version(&config)
        .map_err(|e| Error::new(e, ExitCode::FAILURE))
}

#[derive(Subcommand)]
pub enum Config {
    /// Generates a shell completion script for supported shells.
    /// See the help menu for more information on supported shells.
    Completion {
        #[arg(short, long, value_name = "shell")]
        shell: Option<Shell>,

        #[arg(short, long)]
        /// Installs the completion script in your shell init file.
        /// If this flag is passed the --shell is required
        install: bool,

        #[arg(short, long)]
        /// Uninstalls the completion script from your shell init file.
        /// If this flag is passed the --shell is required
        uninstall: bool,
    },
}

fn generate_shell_completion_script() {
    let mut cmd = Cli::command();
    clap_complete::generate(
        Shell::Bash,
        &mut cmd,
        "huak",
        &mut std::io::stdout(),
    )
}

fn run_with_install(shell: Option<Shell>) -> CliResult<()> {
    let sh = match shell {
        Some(it) => it,
        None => {
            return Err(Error::new(
                HuakError::HuakConfigurationError(
                    "no shell provided".to_string(),
                ),
                ExitCode::FAILURE,
            ))
        }
    };
    let mut cmd = Cli::command();
    match sh {
        Shell::Bash => add_completion_bash(),
        Shell::Elvish => add_completion_elvish(),
        Shell::Fish => add_completion_fish(&mut cmd),
        Shell::PowerShell => add_completion_powershell(),
        Shell::Zsh => add_completion_zsh(&mut cmd),
        _ => Err(Error::new(
            HuakError::HuakConfigurationError("invalid shell".to_string()),
            ExitCode::FAILURE,
        )),
    }
}

fn run_with_uninstall(shell: Option<Shell>) -> CliResult<()> {
    let sh = match shell {
        Some(it) => it,
        None => {
            return Err(Error::new(
                HuakError::HuakConfigurationError(
                    "no shell provided".to_string(),
                ),
                ExitCode::FAILURE,
            ))
        }
    };
    match sh {
        Shell::Bash => remove_completion_bash(),
        Shell::Elvish => remove_completion_elvish(),
        Shell::Fish => remove_completion_fish(),
        Shell::PowerShell => remove_completion_powershell(),
        Shell::Zsh => remove_completion_zsh(),
        _ => Err(Error::new(
            HuakError::HuakConfigurationError("invalid shell".to_string()),
            ExitCode::FAILURE,
        )),
    }
}

/// Bash has a couple of files that can contain the actual completion script.
/// Only the line `eval "$(huak config completion bash)"` needs to be added
/// These files are loaded in the following order:
/// ~/.bash_profile
/// ~/.bash_login
/// ~/.profile
/// ~/.bashrc
pub fn add_completion_bash() -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(Error::from(e)),
    };
    let file_path = format!("{home}/.bashrc");
    // Opening file in append mode
    let mut file = File::options().append(true).open(file_path)?;
    // This needs to be a string since there will be a \n prepended if it is
    file.write_all(
        format!(r##"{}eval "$(huak config completion)"{}"##, '\n', '\n')
            .as_bytes(),
    )
    .map_err(Error::from)
}

pub fn add_completion_elvish() -> CliResult<()> {
    todo!()
}

/// huak config completion fish > ~/.config/fish/completions/huak.fish
/// Fish has a completions directory in which all files are loaded on init.
/// The naming convention is $HOME/.config/fish/completions/huak.fish
pub fn add_completion_fish(cli: &mut Command) -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(Error::from(e)),
    };
    let target_file = format!("{home}/.config/fish/completions/huak.fish");
    generate_target_file(target_file, cli)
}

pub fn add_completion_powershell() -> CliResult<()> {
    todo!()
}

/// Zsh and fish are the same in the sense that the use an entire directory to collect shell init
/// scripts.
pub fn add_completion_zsh(cli: &mut Command) -> CliResult<()> {
    let target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();
    generate_target_file(target_file, cli)
}

/// Reads the entire file and removes lines that match exactly with:
/// \neval "$(huak config completion)
pub fn remove_completion_bash() -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(Error::from(e)),
    };
    let file_path = format!("{home}/.bashrc");
    let file_content = std::fs::read_to_string(&file_path)?;
    let new_content = file_content.replace(
        &format!(r##"{}eval "$(huak config completion)"{}"##, '\n', '\n'),
        "",
    );
    std::fs::write(&file_path, new_content).map_err(Error::from)
}

pub fn remove_completion_elvish() -> CliResult<()> {
    todo!()
}

pub fn remove_completion_fish() -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(Error::from(e)),
    };
    let target_file = format!("{home}/.config/fish/completions/huak.fish");
    std::fs::remove_file(target_file).map_err(Error::from)
}

pub fn remove_completion_powershell() -> CliResult<()> {
    unimplemented!()
}

pub fn remove_completion_zsh() -> CliResult<()> {
    let target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();
    std::fs::remove_file(target_file).map_err(Error::from)
}

fn generate_target_file<P>(target_file: P, cmd: &mut Command) -> CliResult<()>
where
    P: AsRef<Path>,
{
    let mut file = File::create(&target_file)?;
    clap_complete::generate(Shell::Fish, cmd, "huak", &mut file);
    Ok(())
}

fn init_config(
    trailing: Option<Vec<String>>,
    quiet: bool,
) -> CliResult<OperationConfig> {
    let config = OperationConfig {
        workspace_root: ops::find_workspace()
            .map_err(|e| Error::new(e, ExitCode::FAILURE))?,
        trailing_command_parts: trailing,
        terminal_options: Some(TerminalOptions {
            verbosity: if quiet {
                Verbosity::Quiet
            } else {
                Verbosity::Normal
            },
        }),
        ..Default::default()
    };
    Ok(config)
}

#[cfg(target_family = "unix")]
#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Command, Parser};
    use std::fs;

    #[derive(Parser)]
    struct Cli {}

    #[cfg(target_family = "unix")]
    #[ignore = "incomplete test"] // See TODO
    #[test]
    /// This test ensures the order of operations is always correct
    fn test_bash_completion() {
        test_adding_completion_bash();
        test_remove_completion_bash();
    }

    #[cfg(target_family = "unix")]
    fn test_adding_completion_bash() {
        let _ = add_completion_bash();
        // TODO: Use tempdir
        let file_content =
            fs::read_to_string("resources/test_files/test_bashrc").unwrap();

        assert_eq!(
            format!(
                r##"# This stuff is in here so there is something that should be left over after
# removing the bash completion script


eval "$(huak config completion)"
"##
            ),
            file_content
        )
    }

    #[cfg(target_family = "unix")]
    fn test_remove_completion_bash() {
        let _ = remove_completion_bash();
        // TODO: Use tempdir
        let file_content =
            fs::read_to_string("resources/test_files/test_bashrc").unwrap();

        assert_eq!("# This stuff is in here so there is something that should be left over after
# removing the bash completion script

", file_content)
    }

    #[cfg(target_family = "unix")]
    #[ignore = "incomplete test"] // See TODO
    #[test]
    /// This test ensures the order of operations is always correct
    fn test_fish_completion() {
        let mut cmd = Cli::command();

        test_adding_completion_fish(&mut cmd);
        test_remove_completion_fish();
    }

    #[cfg(target_family = "unix")]
    fn test_adding_completion_fish(cmd: &mut Command) {
        let _ = add_completion_fish(cmd);
        // TODO: Use tempdir
        let result = std::fs::read_to_string("resources/test_files/test_fish");

        assert_eq!(true, result.is_ok());
    }

    #[cfg(target_family = "unix")]
    fn test_remove_completion_fish() {
        let _ = remove_completion_fish();
        // TODO: Use tempdir
        let result = std::fs::read("resources/test_files/test_fish");
        assert_eq!(true, result.is_err());
    }

    #[cfg(target_family = "unix")]
    #[ignore = "incomplete test"] // See TODO
    #[test]
    /// This test ensures the order of operations is always correct
    fn test_zsh_completion() {
        let mut cmd = Cli::command();

        test_adding_completion_zsh(&mut cmd);
        test_remove_completion_zsh();
    }

    #[cfg(target_family = "unix")]
    fn test_adding_completion_zsh(cmd: &mut Command) {
        let _ = add_completion_zsh(cmd);
        // TODO: Use tempdir
        let result = std::fs::read_to_string("resources/test_files/test_zsh");

        assert_eq!(true, result.is_ok());
    }

    #[cfg(target_family = "unix")]
    fn test_remove_completion_zsh() {
        let _ = remove_completion_zsh();
        // TODO: Use tempdir
        let result = std::fs::read("resources/test_files/test_zsh");
        assert_eq!(true, result.is_err());
    }
}
