use crate::error::{CliResult, Error};
use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{self, Shell};
use huak::{
    ops::{self, find_workspace, OperationConfig},
    BuildOptions, CleanOptions, Error as HuakError, FormatOptions, HuakResult,
    InstallerOptions, LintOptions, PublishOptions, TerminalOptions,
    TestOptions, Verbosity, WorkspaceOptions,
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
#[clap(rename_all = "kebab-case")]
pub enum Commands {
    /// Add dependencies to the project.
    Add {
        #[arg(num_args = 1.., required = true)]
        dependencies: Vec<String>,
        /// Adds an optional dependency group.
        #[arg(long)]
        group: Option<String>,
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
        trailing: Option<Vec<String>>,
    },
    /// Build tarball and wheel for the project.
    Build {
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
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
        include_pyc: bool,
        #[arg(long, required = false)]
        /// Remove all __pycache__ directories.
        include_pycache: bool,
    },
    /// Auto-fix fixable lint conflicts
    Fix {
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
        trailing: Option<Vec<String>>,
    },
    /// Format the project's Python code.
    Fmt {
        /// Check if Python code is formatted.
        #[arg(long)]
        check: bool,
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
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
        /// Don't initialize VCS in the project
        #[arg(long)]
        no_vcs: bool,
    },
    /// Install the dependencies of an existing project.
    Install {
        /// Install optional dependency groups
        #[arg(long, num_args = 1..)]
        groups: Option<Vec<String>>,
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
        trailing: Option<Vec<String>>,
    },
    /// Lint the project's Python code.
    Lint {
        /// Address any fixable lints.
        #[arg(long)]
        fix: bool,
        /// Perform type-checking.
        #[arg(long)]
        no_types: bool,
        /// Pass trailing arguments with `--` to `ruff`.
        #[arg(last = true)]
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
        #[arg(last = true)]
        trailing: Option<Vec<String>>,
    },
    /// Remove dependencies from the project.
    Remove {
        #[arg(num_args = 1.., required = true)]
        dependencies: Vec<String>,
        /// Remove from optional dependency group
        #[arg(long, num_args = 1)]
        group: Option<String>,
    },
    /// Run a command within the project's environment context.
    Run {
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Test the project's Python code.
    Test {
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
        trailing: Option<Vec<String>>,
    },
    /// Display the version of the project.
    Version,
}

// Command gating for Huak.
impl Cli {
    pub fn run(self) -> CliResult<()> {
        let workspace_root =
            find_workspace().unwrap_or(std::env::current_dir()?);
        let verbosity = match self.quiet {
            true => Verbosity::Quiet,
            false => Verbosity::Normal,
        };
        let mut operation_config = OperationConfig {
            workspace_root,
            terminal_options: TerminalOptions { verbosity },
            ..Default::default()
        };
        match self.command {
            Commands::Config { command } => config(command),
            Commands::Add {
                dependencies,
                group,
                trailing,
            } => {
                operation_config.installer_options =
                    Some(InstallerOptions { args: trailing });
                add(dependencies, group, operation_config)
            }
            Commands::Build { trailing } => {
                operation_config.build_options =
                    Some(BuildOptions { args: trailing });
                build(operation_config)
            }
            Commands::Clean {
                include_pyc,
                include_pycache,
            } => {
                let options = CleanOptions {
                    include_pycache,
                    include_compiled_bytecode: include_pyc,
                };
                operation_config.clean_options = Some(options);
                clean(operation_config)
            }
            Commands::Fix { trailing } => {
                operation_config.lint_options = Some(LintOptions {
                    args: trailing,
                    include_types: false,
                });
                if let Some(it) = operation_config.lint_options.as_mut() {
                    if let Some(a) = it.args.as_mut() {
                        a.push("--fix".to_string());
                    }
                }
                fix(operation_config)
            }
            Commands::Fmt { check, trailing } => {
                operation_config.format_options =
                    Some(FormatOptions { args: trailing });
                if check {
                    if let Some(it) = operation_config.format_options.as_mut() {
                        if let Some(a) = it.args.as_mut() {
                            a.push("--check".to_string());
                        }
                    }
                }
                fmt(operation_config)
            }
            Commands::Init { app, lib, no_vcs } => {
                operation_config.workspace_root = std::env::current_dir()?;
                operation_config.workspace_options =
                    Some(WorkspaceOptions { uses_git: !no_vcs });
                init(app, lib, operation_config)
            }
            Commands::Install { groups, trailing } => {
                operation_config.installer_options =
                    Some(InstallerOptions { args: trailing });
                install(groups, operation_config)
            }
            Commands::Lint {
                fix,
                no_types,
                trailing,
            } => {
                operation_config.lint_options = Some(LintOptions {
                    args: trailing,
                    include_types: !no_types,
                });
                if fix {
                    if let Some(it) = operation_config.lint_options.as_mut() {
                        if let Some(a) = it.args.as_mut() {
                            a.push("--fix".to_string());
                        }
                    }
                }
                lint(operation_config)
            }
            Commands::New {
                path,
                app,
                lib,
                no_vcs,
            } => {
                operation_config.workspace_root = PathBuf::from(path);
                operation_config.workspace_options =
                    Some(WorkspaceOptions { uses_git: !no_vcs });
                new(app, lib, operation_config)
            }
            Commands::Publish { trailing } => {
                operation_config.publish_options =
                    Some(PublishOptions { args: trailing });
                publish(operation_config)
            }
            Commands::Remove {
                dependencies,
                group,
            } => remove(dependencies, group, operation_config),
            Commands::Run { command } => run(command, operation_config),
            Commands::Test { trailing } => {
                operation_config.test_options =
                    Some(TestOptions { args: trailing });
                test(operation_config)
            }
            Commands::Version => version(operation_config),
        }
        .map_err(|e| Error::new(e, ExitCode::FAILURE))
    }
}

fn add(
    dependencies: Vec<String>,
    group: Option<String>,
    operation_config: OperationConfig,
) -> HuakResult<()> {
    let deps: Vec<&str> =
        dependencies.iter().map(|item| item.as_str()).collect();
    match group.as_ref() {
        Some(it) => {
            ops::add_project_optional_dependencies(&deps, it, &operation_config)
        }
        None => ops::add_project_dependencies(&deps, &operation_config),
    }
}

fn build(operation_config: OperationConfig) -> HuakResult<()> {
    ops::build_project(&operation_config)
}

fn clean(operation_config: OperationConfig) -> HuakResult<()> {
    ops::clean_project(&operation_config)
}

/// Prints the script to stdout and a way to add the script to the shell init file to stderr. This
/// way if the user runs completion <shell> > completion.sh only the stdout will be redirected into
/// completion.sh.
fn config(command: Config) -> HuakResult<()> {
    match command {
        Config::Completion {
            shell,
            install,
            uninstall,
        } => {
            if (install || uninstall) && shell.is_none() {
                return Err(HuakError::HuakConfigurationError(
                    "no shell provided".to_string(),
                ));
            }
            if install {
                run_with_install(shell)
            } else if uninstall {
                run_with_uninstall(shell)
            } else {
                generate_shell_completion_script();
                Ok(())
            }
        }
    }
}

fn fix(operation_config: OperationConfig) -> HuakResult<()> {
    ops::lint_project(&operation_config)
}

fn fmt(operation_config: OperationConfig) -> HuakResult<()> {
    ops::format_project(&operation_config)
}

fn init(
    app: bool,
    _lib: bool,
    operation_config: OperationConfig,
) -> HuakResult<()> {
    if app {
        ops::init_app_project(&operation_config)
    } else {
        ops::init_lib_project(&operation_config)
    }
}

fn install(
    groups: Option<Vec<String>>,
    operation_config: OperationConfig,
) -> HuakResult<()> {
    if let Some(it) = groups {
        for group in &it {
            ops::install_project_optional_dependencies(
                group,
                &operation_config,
            )?;
        }
        Ok(())
    } else {
        ops::install_project_dependencies(&operation_config)
    }
}

fn lint(operation_config: OperationConfig) -> HuakResult<()> {
    ops::lint_project(&operation_config)
}

fn new(
    app: bool,
    _lib: bool,
    operation_config: OperationConfig,
) -> HuakResult<()> {
    if app {
        ops::new_app_project(&operation_config)
    } else {
        ops::new_lib_project(&operation_config)
    }
}

fn publish(operation_config: OperationConfig) -> HuakResult<()> {
    ops::publish_project(&operation_config)
}

fn remove(
    dependencies: Vec<String>,
    group: Option<String>,
    operation_config: OperationConfig,
) -> HuakResult<()> {
    let deps: Vec<&str> =
        dependencies.iter().map(|item| item.as_str()).collect();
    match group.as_ref() {
        Some(it) => ops::remove_project_optional_dependencies(
            &deps,
            it,
            &operation_config,
        ),
        None => ops::remove_project_dependencies(&deps, &operation_config),
    }
}

fn run(
    command: Vec<String>,
    operation_config: OperationConfig,
) -> HuakResult<()> {
    ops::run_command_str(&command.join(" "), &operation_config)
}

fn test(operation_config: OperationConfig) -> HuakResult<()> {
    ops::test_project(&operation_config)
}

fn version(operation_config: OperationConfig) -> HuakResult<()> {
    ops::display_project_version(&operation_config)
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

fn run_with_install(shell: Option<Shell>) -> HuakResult<()> {
    let sh = match shell {
        Some(it) => it,
        None => {
            return Err(HuakError::HuakConfigurationError(
                "no shell provided".to_string(),
            ))
        }
    };
    let mut cmd = Cli::command();
    match sh {
        Shell::Bash => add_completion_bash(),
        Shell::Elvish => Err(HuakError::UnimplementedError(
            "elvish completion".to_string(),
        )),
        Shell::Fish => add_completion_fish(&mut cmd),
        Shell::PowerShell => Err(HuakError::UnimplementedError(
            "powershell completion".to_string(),
        )),
        Shell::Zsh => add_completion_zsh(&mut cmd),
        _ => Err(HuakError::HuakConfigurationError(
            "invalid shell".to_string(),
        )),
    }
}

fn run_with_uninstall(shell: Option<Shell>) -> HuakResult<()> {
    let sh = match shell {
        Some(it) => it,
        None => {
            return Err(HuakError::HuakConfigurationError(
                "no shell provided".to_string(),
            ))
        }
    };
    match sh {
        Shell::Bash => remove_completion_bash(),
        Shell::Elvish => Err(HuakError::UnimplementedError(
            "elvish completion".to_string(),
        )),
        Shell::Fish => remove_completion_fish(),
        Shell::PowerShell => Err(HuakError::UnimplementedError(
            "Powershell completion".to_string(),
        )),
        Shell::Zsh => remove_completion_zsh(),
        _ => Err(HuakError::HuakConfigurationError(
            "invalid shell".to_string(),
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
pub fn add_completion_bash() -> HuakResult<()> {
    let home = std::env::var("HOME")?;
    let file_path = format!("{home}/.bashrc");
    // Opening file in append mode
    let mut file = File::options().append(true).open(file_path)?;
    // This needs to be a string since there will be a \n prepended if it is
    file.write_all(
        format!(r##"{}eval "$(huak config completion)"{}"##, '\n', '\n')
            .as_bytes(),
    )
    .map_err(HuakError::IOError)
}

/// huak config completion fish > ~/.config/fish/completions/huak.fish
/// Fish has a completions directory in which all files are loaded on init.
/// The naming convention is $HOME/.config/fish/completions/huak.fish
pub fn add_completion_fish(cli: &mut Command) -> HuakResult<()> {
    let home = std::env::var("HOME")?;
    let target_file = format!("{home}/.config/fish/completions/huak.fish");
    generate_target_file(target_file, cli)
}

/// Zsh and fish are the same in the sense that the use an entire directory to collect shell init
/// scripts.
pub fn add_completion_zsh(cli: &mut Command) -> HuakResult<()> {
    let target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();
    generate_target_file(target_file, cli)
}

/// Reads the entire file and removes lines that match exactly with:
/// \neval "$(huak config completion)
pub fn remove_completion_bash() -> HuakResult<()> {
    let home = std::env::var("HOME")?;
    let file_path = format!("{home}/.bashrc");
    let file_content = std::fs::read_to_string(&file_path)?;
    let new_content = file_content.replace(
        &format!(r##"{}eval "$(huak config completion)"{}"##, '\n', '\n'),
        "",
    );
    std::fs::write(&file_path, new_content).map_err(HuakError::IOError)
}

pub fn remove_completion_fish() -> HuakResult<()> {
    let home = std::env::var("HOME")?;
    let target_file = format!("{home}/.config/fish/completions/huak.fish");
    std::fs::remove_file(target_file).map_err(HuakError::IOError)
}

pub fn remove_completion_zsh() -> HuakResult<()> {
    let target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();
    std::fs::remove_file(target_file).map_err(HuakError::IOError)
}

fn generate_target_file<P>(target_file: P, cmd: &mut Command) -> HuakResult<()>
where
    P: AsRef<Path>,
{
    let mut file = File::create(&target_file)?;
    clap_complete::generate(Shell::Fish, cmd, "huak", &mut file);
    Ok(())
}
