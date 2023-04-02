use crate::error::{CliResult, Error};
use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{self, Shell};
use huak::{
    ops::{
        self, AddOptions, BuildOptions, CleanOptions, FormatOptions,
        InstallOptions, LintOptions, PublishOptions, RemoveOptions,
        TestOptions, UpdateOptions,
    },
    Config, Error as HuakError, HuakResult, Terminal, Verbosity, Version,
    WorkspaceOptions,
};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
    str::FromStr,
};

/// A Python package manager written in Rust inspired by Cargo.
#[derive(Parser)]
#[command(version, author, about, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, global = true)]
    quiet: bool,
}

// List of commands.
#[derive(Subcommand)]
#[clap(rename_all = "kebab-case")]
enum Commands {
    /// Activate the virtual envionrment.
    Activate,
    /// Add dependencies to the project.
    Add {
        #[arg(num_args = 1.., required = true)]
        dependencies: Vec<Dependency>,
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
    /// Remove tarball and wheel from the built project.
    Clean {
        #[arg(long, required = false)]
        /// Remove all .pyc files.
        include_pyc: bool,
        #[arg(long, required = false)]
        /// Remove all __pycache__ directories.
        include_pycache: bool,
    },
    /// Generates a shell completion script for supported shells.
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
    /// Manage Python installations.
    Python {
        #[command(subcommand)]
        command: Python,
    },
    /// Remove dependencies from the project.
    Remove {
        #[arg(num_args = 1.., required = true)]
        dependencies: Vec<String>,
        /// Remove from optional dependency group
        #[arg(long, num_args = 1)]
        group: Option<String>,
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
        trailing: Option<Vec<String>>,
    },
    /// Run a command within the project's environment context.
    Run {
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Test the project's Python code.
    Test {
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
        trailing: Option<Vec<String>>,
    },
    /// Update the project's dependencies.
    Update {
        #[arg(num_args = 0..)]
        dependencies: Option<Vec<String>>,
        /// Update an optional dependency group
        #[arg(long)]
        group: Option<String>,
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
        trailing: Option<Vec<String>>,
    },
    /// Display the version of the project.
    Version,
}

#[derive(Subcommand)]
enum Python {
    /// List the installed Python interpreters.
    List,
    /// Use a specific Python interpreter.
    Use {
        /// A Python interpreter version number.
        #[arg(required = true)]
        version: PythonVersion,
    },
}

// Command gating for Huak.
impl Cli {
    pub fn run(self) -> CliResult<()> {
        let cwd = std::env::current_dir()?;
        let verbosity = match self.quiet {
            true => Verbosity::Quiet,
            false => Verbosity::Normal,
        };
        let mut terminal = Terminal::new();
        terminal.set_verbosity(verbosity);
        let mut config = Config {
            workspace_root: cwd.to_path_buf(),
            cwd,
            terminal,
        };
        match self.command {
            Commands::Activate => activate(&mut config),
            Commands::Add {
                dependencies,
                group,
                trailing,
            } => {
                let options = Some(AddOptions {
                    install_options: Some(InstallOptions { args: trailing }),
                    args: None,
                });
                add(dependencies, group, &mut config, options)
            }
            Commands::Build { trailing } => {
                let options = Some(BuildOptions {
                    args: trailing,
                    install_options: None,
                });
                build(&mut config, options)
            }
            Commands::Clean {
                include_pyc,
                include_pycache,
            } => {
                let options = Some(CleanOptions {
                    include_pycache,
                    include_compiled_bytecode: include_pyc,
                });
                clean(&mut config, options)
            }
            Commands::Completion {
                shell,
                install,
                uninstall,
            } => {
                if (install || uninstall) && shell.is_none() {
                    Err(HuakError::HuakConfigurationError(
                        "no shell provided".to_string(),
                    ))
                } else if install {
                    run_with_install(shell)
                } else if uninstall {
                    run_with_uninstall(shell)
                } else {
                    generate_shell_completion_script();
                    Ok(())
                }
            }
            Commands::Fix { trailing } => {
                let options = Some(LintOptions {
                    args: trailing,
                    include_types: false,
                    install_options: None,
                });
                fix(&mut config, options)
            }
            Commands::Fmt { check, trailing } => {
                let mut args = if check {
                    vec!["--check".to_string()]
                } else {
                    Vec::new()
                };
                if let Some(it) = trailing {
                    args.extend(it);
                }
                let options = Some(FormatOptions {
                    args: Some(args),
                    install_options: None,
                });
                fmt(&mut config, options)
            }
            Commands::Init { app, lib, no_vcs } => {
                config.workspace_root = config.cwd.clone();
                let options = Some(WorkspaceOptions { uses_git: !no_vcs });
                init(app, lib, &mut config, options)
            }
            Commands::Install { groups, trailing } => {
                let options = Some(InstallOptions { args: trailing });
                install(groups, &mut config, options)
            }
            Commands::Lint {
                fix,
                no_types,
                trailing,
            } => {
                let mut args = if fix {
                    vec!["--fix".to_string()]
                } else {
                    Vec::new()
                };
                if let Some(it) = trailing {
                    args.extend(it);
                }
                let options = Some(LintOptions {
                    args: Some(args),
                    include_types: !no_types,
                    install_options: None,
                });
                lint(&mut config, options)
            }
            Commands::New {
                path,
                app,
                lib,
                no_vcs,
            } => {
                config.workspace_root =
                    std::fs::canonicalize(PathBuf::from(path))?;
                let options = Some(WorkspaceOptions { uses_git: !no_vcs });
                new(app, lib, &mut config, options)
            }
            Commands::Publish { trailing } => {
                let options = Some(PublishOptions {
                    args: trailing,
                    install_options: None,
                });
                publish(&mut config, options)
            }
            Commands::Python { command } => python(command, &mut config),
            Commands::Remove {
                dependencies,
                group,
                trailing,
            } => {
                let options = Some(RemoveOptions {
                    install_options: Some(InstallOptions { args: trailing }),
                    args: None,
                });
                remove(dependencies, group, &mut config, options)
            }
            Commands::Run { command } => run(command, &mut config),
            Commands::Test { trailing } => {
                let options = Some(TestOptions {
                    args: trailing,
                    install_options: None,
                });
                test(&mut config, options)
            }
            Commands::Update {
                dependencies,
                group,
                trailing,
            } => {
                let options = Some(UpdateOptions {
                    install_options: Some(InstallOptions { args: trailing }),
                    args: None,
                });
                update(dependencies, group, &mut config, options)
            }
            Commands::Version => version(&mut config),
        }
        .map_err(|e| Error::new(e, ExitCode::FAILURE))
    }
}

fn activate(config: &mut Config) -> HuakResult<()> {
    ops::activate_python_environment(config)
}

fn add(
    dependencies: Vec<Dependency>,
    group: Option<String>,
    config: &mut Config,
    options: Option<AddOptions>,
) -> HuakResult<()> {
    let deps = dependencies
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<String>>();
    match group.as_ref() {
        Some(it) => {
            ops::add_project_optional_dependencies(&deps, it, config, options)
        }
        None => ops::add_project_dependencies(&deps, config, options),
    }
}

fn build(config: &mut Config, options: Option<BuildOptions>) -> HuakResult<()> {
    ops::build_project(config, options)
}

fn clean(config: &mut Config, options: Option<CleanOptions>) -> HuakResult<()> {
    ops::clean_project(config, options)
}

fn fix(config: &mut Config, options: Option<LintOptions>) -> HuakResult<()> {
    ops::lint_project(config, options)
}

fn fmt(config: &mut Config, options: Option<FormatOptions>) -> HuakResult<()> {
    ops::format_project(config, options)
}

fn init(
    app: bool,
    _lib: bool,
    config: &mut Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    if app {
        ops::init_app_project(config, options)
    } else {
        ops::init_lib_project(config, options)
    }
}

fn install(
    groups: Option<Vec<String>>,
    config: &mut Config,
    options: Option<InstallOptions>,
) -> HuakResult<()> {
    if let Some(it) = groups {
        if it.contains(&"all".to_string()) {
            ops::install_project_dependencies(config, options.clone())?;
        }
        ops::install_project_optional_dependencies(&it, config, options)
    } else {
        ops::install_project_dependencies(config, options)
    }
}

fn lint(config: &mut Config, options: Option<LintOptions>) -> HuakResult<()> {
    ops::lint_project(config, options)
}

fn new(
    app: bool,
    _lib: bool,
    config: &mut Config,
    options: Option<WorkspaceOptions>,
) -> HuakResult<()> {
    if app {
        ops::new_app_project(config, options)
    } else {
        ops::new_lib_project(config, options)
    }
}

fn publish(
    config: &mut Config,
    options: Option<PublishOptions>,
) -> HuakResult<()> {
    ops::publish_project(config, options)
}

fn python(command: Python, config: &mut Config) -> HuakResult<()> {
    match command {
        Python::List => ops::list_python(config),
        Python::Use { version } => ops::use_python(version.0.as_str(), config),
    }
}

fn remove(
    dependencies: Vec<String>,
    group: Option<String>,
    config: &mut Config,
    options: Option<RemoveOptions>,
) -> HuakResult<()> {
    match group.as_ref() {
        Some(it) => ops::remove_project_optional_dependencies(
            &dependencies,
            it,
            config,
            options,
        ),
        None => {
            ops::remove_project_dependencies(&dependencies, config, options)
        }
    }
}

fn run(command: Vec<String>, config: &mut Config) -> HuakResult<()> {
    ops::run_command_str(&command.join(" "), config)
}

fn test(config: &mut Config, options: Option<TestOptions>) -> HuakResult<()> {
    ops::test_project(config, options)
}

fn update(
    dependencies: Option<Vec<String>>,
    group: Option<String>,
    config: &mut Config,
    options: Option<UpdateOptions>,
) -> HuakResult<()> {
    match group.as_ref() {
        Some(it) => {
            if it == "all" {
                ops::update_project_dependencies(
                    dependencies.clone(),
                    config,
                    options.clone(),
                )?;
            }
            ops::update_project_optional_dependencies(
                dependencies,
                it,
                config,
                options,
            )
        }
        None => ops::update_project_dependencies(dependencies, config, options),
    }
}

fn version(config: &mut Config) -> HuakResult<()> {
    ops::display_project_version(config)
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

#[derive(Debug, Clone)]
pub struct Dependency(String);

impl FromStr for Dependency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.replace('@', "==")))
    }
}

impl ToString for Dependency {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct PythonVersion(String);

impl FromStr for PythonVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let version = Version::from_str(s).map_err(|_| {
            Error::new(
                HuakError::InternalError("failed to parse version".to_string()),
                ExitCode::FAILURE,
            )
        })?;

        Ok(Self(version.to_string()))
    }
}
