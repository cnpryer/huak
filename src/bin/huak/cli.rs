use std::{fs::File, io::Write, path::Path, process::ExitCode};

use crate::error::{CliError, CliResult};

use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use huak::Error;

/// A Python package manager written in Rust inspired by Cargo.
#[derive(Parser)]
#[command(version, author, about, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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
        /// Pass a trailing command to the executed operation.
        #[arg(trailing_var_arg = true)]
        command: Option<String>,
    },
    /// Check for vulnerable dependencies and license compatibility*.
    Audit,
    /// Build tarball and wheel for the project.
    Build {
        /// Pass a trailing command to the executed operation.
        #[arg(trailing_var_arg = true)]
        command: Option<String>,
    },
    /// Interact with the configuration of huak.
    Config {
        #[command(subcommand)]
        command: Config,
    },
    /// Remove tarball and wheel from the built project.
    Clean {
        #[arg(long, required = false)]
        /// Remove all .pyc files and __pycache__ directories.
        pycache: bool,
    },
    /// Generates documentation for the project*.
    Doc {
        #[arg(long)]
        check: bool,
    },
    /// Auto-fix fixable lint conflicts
    Fix {
        /// Pass a trailing command to the executed operation.
        #[arg(trailing_var_arg = true)]
        command: Option<String>,
    },
    /// Format the project's Python code.
    Fmt {
        /// Check if Python code is formatted.
        #[arg(long)]
        check: bool,
        /// Pass a trailing command to the executed operation.
        #[arg(trailing_var_arg = true)]
        command: Option<String>,
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
        /// Pass a trailing command to the executed operation.
        #[arg(trailing_var_arg = true)]
        command: Option<String>,
    },
    /// Lint the project's Python code.
    Lint {
        #[arg(long, required = false)]
        fix: bool,
        /// Pass a trailing command to the executed operation.
        #[arg(trailing_var_arg = true)]
        command: Option<String>,
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
        /// Pass a trailing command to the executed operation.
        #[arg(trailing_var_arg = true)]
        command: Option<String>,
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
        command: String,
    },
    /// Test the project's Python code.
    Test {
        /// Pass a trailing command to the executed operation.
        #[arg(trailing_var_arg = true)]
        command: Option<String>,
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
            Commands::Config { command } => config(command),
            Commands::Activate => activate(),
            Commands::Add {
                dependency,
                group,
                command,
            } => add(dependency, group, command),
            Commands::Audit => audit(),
            Commands::Build { command } => build(command),
            Commands::Clean { pycache } => clean(pycache),
            Commands::Doc { check } => doc(check),
            Commands::Fix { command } => fix(command),
            Commands::Fmt { check, command } => fmt(check, command),
            // --lib is the default, so it's unnecessary to handle. If --app is not passed, assume --lib.
            #[allow(unused_variables)]
            Commands::Init { app, lib } => init(app),
            Commands::Install { groups, command } => install(groups, command),
            Commands::Lint { fix, command } => lint(fix, command),
            // --lib is the default, so it's unnecessary to handle. If --app is not passed, assume --lib.
            #[allow(unused_variables)]
            Commands::New {
                path,
                app,
                lib,
                no_vcs,
            } => new(path, app, no_vcs),
            Commands::Publish { command } => publish(command),
            Commands::Remove { dependency, group } => remove(dependency, group),
            Commands::Run { command } => run(command),
            Commands::Test { command } => test(command),
            Commands::Update { dependency } => update(dependency),
            Commands::Version => version(),
        }
    }
}

fn activate() -> CliResult<()> {
    todo!()
}

fn add(
    dependency: String,
    group: Option<String>,
    command: Option<String>,
) -> CliResult<()> {
    todo!()
}

fn audit() -> CliResult<()> {
    todo!()
}

fn build(command: Option<String>) -> CliResult<()> {
    todo!()
}

fn clean(pycache: bool) -> CliResult<()> {
    todo!()
}

/// Prints the script to stdout and a way to add the script to the shell init file to stderr. This
/// way if the user runs completion <shell> > completion.sh only the stdout will be redirected into
/// completion.sh.
fn config(command: Config) -> CliResult<()> {
    match command {
        Config::Completion {
            shell,
            install,
            uninstall,
        } => {
            if (install || uninstall) && shell.is_none() {
                return Err(CliError::new(
                    Error::HuakConfigurationError(
                        "No shell provided".to_string(),
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

fn doc(check: bool) -> CliResult<()> {
    todo!()
}

fn fix(command: Option<String>) -> CliResult<()> {
    todo!()
}

fn fmt(check: bool, command: Option<String>) -> CliResult<()> {
    todo!()
}

fn init(app: bool) -> CliResult<()> {
    todo!()
}

fn install(
    groups: Option<Vec<String>>,
    command: Option<String>,
) -> CliResult<()> {
    todo!()
}

fn lint(fix: bool, command: Option<String>) -> CliResult<()> {
    todo!()
}

fn new(path: String, app: bool, no_vcs: bool) -> CliResult<()> {
    todo!()
}

fn publish(command: Option<String>) -> CliResult<()> {
    todo!()
}

fn remove(dependency: String, group: Option<String>) -> CliResult<()> {
    todo!()
}

fn run(command: String) -> CliResult<()> {
    todo!()
}

fn test(command: Option<String>) -> CliResult<()> {
    todo!()
}

fn update(dependency: String) -> CliResult<()> {
    todo!()
}

fn version() -> CliResult<()> {
    todo!()
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

    generate(Shell::Bash, &mut cmd, "huak", &mut std::io::stdout())
}

fn run_with_install(shell: Option<Shell>) -> CliResult<()> {
    let sh = match shell {
        Some(it) => it,
        None => {
            return Err(CliError::new(
                Error::HuakConfigurationError("No shell provided".to_string()),
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
        _ => {
            return Err(CliError::new(
                Error::HuakConfigurationError("Invalid shell".to_string()),
                ExitCode::FAILURE,
            ));
        }
    }?;

    Ok(())
}

fn run_with_uninstall(shell: Option<Shell>) -> CliResult<()> {
    let sh = match shell {
        Some(it) => it,
        None => {
            return Err(CliError::new(
                Error::HuakConfigurationError("No shell provided".to_string()),
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
        _ => {
            return Err(CliError::new(
                Error::HuakConfigurationError("Invalid shell".to_string()),
                ExitCode::FAILURE,
            ));
        }
    }?;

    Ok(())
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
        Err(e) => return Err(CliError::from(e)),
    };

    let file_path = format!("{home}/.bashrc");

    // opening file in append mode
    let mut file = File::options().append(true).open(file_path)?;

    // This needs to be a string since there will be a \n prepended if it is
    file.write_all(
        format!(r##"{}eval "$(huak config completion)"{}"##, '\n', '\n')
            .as_bytes(),
    )?;

    Ok(())
}

// TODO
pub fn add_completion_elvish() -> CliResult<()> {
    todo!()
}

/// huak config completion fish > ~/.config/fish/completions/huak.fish
/// Fish has a completions directory in which all files are loaded on init.
/// The naming convention is $HOME/.config/fish/completions/huak.fish
pub fn add_completion_fish(cli: &mut Command) -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(CliError::from(e)),
    };

    let target_file = format!("{home}/.config/fish/completions/huak.fish");

    generate_target_file(target_file, cli)?;
    Ok(())
}

// TODO
pub fn add_completion_powershell() -> CliResult<()> {
    todo!()
}

/// Zsh and fish are the same in the sense that the use an entire directory to collect shell init
/// scripts.
pub fn add_completion_zsh(cli: &mut Command) -> CliResult<()> {
    let target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();

    generate_target_file(target_file, cli)?;
    Ok(())
}

/// Reads the entire file and removes lines that match exactly with:
/// \neval "$(huak config completion)
pub fn remove_completion_bash() -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(CliError::from(e)),
    };

    let file_path = format!("{home}/.bashrc");

    let file_content = std::fs::read_to_string(&file_path)?;
    let new_content = file_content.replace(
        &format!(r##"{}eval "$(huak config completion)"{}"##, '\n', '\n'),
        "",
    );

    std::fs::write(&file_path, new_content)?;

    Ok(())
}

// TODO
pub fn remove_completion_elvish() -> CliResult<()> {
    unimplemented!()
}

pub fn remove_completion_fish() -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(CliError::from(e)),
    };

    let target_file = format!("{home}/.config/fish/completions/huak.fish");

    std::fs::remove_file(target_file)?;

    Ok(())
}

// TODO
pub fn remove_completion_powershell() -> CliResult<()> {
    unimplemented!()
}

pub fn remove_completion_zsh() -> CliResult<()> {
    let target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();

    std::fs::remove_file(target_file)?;

    Ok(())
}

fn generate_target_file<P>(target_file: P, cmd: &mut Command) -> CliResult<()>
where
    P: AsRef<Path>,
{
    let mut file = File::create(&target_file)?;

    generate(Shell::Fish, cmd, "huak", &mut file);

    Ok(())
}

// TODO:
//   - Use tempdir and mocking for testing these features.
//   - Requires refactors of functions and their signatures.
//   - Windows tests
#[cfg(target_family = "unix")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use clap::{Command, Parser};

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
