use crate::error::{CliResult, Error};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{self, Shell};
use huak_home::huak_home_dir;
use huak_package_manager::ops::{
    self, AddOptions, BuildOptions, CleanOptions, FormatOptions, LintOptions, PublishOptions,
    RemoveOptions, TestOptions, UpdateOptions,
};
use huak_package_manager::{
    find_package_root, Config, Error as HuakError, HuakResult, InstallOptions, TerminalOptions,
    Verbosity, WorkspaceOptions,
};
use huak_python_manager::RequestedVersion;
use std::{env::current_dir, path::PathBuf, process::ExitCode, str::FromStr};
use termcolor::ColorChoice;

/// A Python package manager written in Rust inspired by Cargo.
#[derive(Parser)]
#[command(version, author, about, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, global = true)]
    quiet: bool,
    #[arg(long, global = true)]
    no_color: bool,
}

// List of commands.
#[derive(Subcommand)]
#[clap(rename_all = "kebab-case")]
enum Commands {
    /// Activate the virtual environment.
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
        /// Use an application template.
        #[arg(long, conflicts_with = "lib")]
        app: bool,
        /// Use a library template [default].
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
        /// Use an application template.
        #[arg(long, conflicts_with = "lib")]
        app: bool,
        /// Use a library template [default].
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
    #[clap(alias = "py")]
    Python {
        #[command(subcommand)]
        command: Python,
    },
    /// Remove dependencies from the project.
    Remove {
        #[arg(num_args = 1.., required = true)]
        dependencies: Vec<String>,
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
        /// Pass trailing arguments with `--`.
        #[arg(last = true)]
        trailing: Option<Vec<String>>,
    },
    /// Display the version of the project.
    Version,
}

#[derive(Subcommand)]
enum Python {
    /// List available Python interpreters.
    List,
    /// Use an available Python interpreter.
    Use {
        /// The version of Python to use.
        #[arg(required = true)]
        version: RequestedVersion,
    },
    /// Install a Python interpreter.
    Install {
        /// The version of Python to install.
        #[arg(required = true)]
        version: RequestedVersion,
    },
}

#[derive(Subcommand)]
enum Toolchain {
    /// List available toolchains.
    List,
    /// Use an available toolchain.
    Use {
        /// The version of Python to use.
        #[arg(required = true)]
        version: RequestedVersion,
    },
    /// Install a toolchain.
    Install {
        /// The version of Python to install.
        #[arg(required = true)]
        version: RequestedVersion,
        /// The path to install a toolchain to.
        target: PathBuf,
    },
}

// Command gating for Huak.
impl Cli {
    pub fn run(self) -> CliResult<i32> {
        let cwd = current_dir()?;
        let mut config = get_config(cwd, &self);

        match exec_command(self.command, &mut config) {
            Ok(()) => Ok(0),
            // TODO: Implement our own ExitCode or status handler.
            Err(HuakError::SubprocessFailure(e)) => Ok(e.code().unwrap_or_default()),
            Err(e) => Err(Error::new(e, ExitCode::FAILURE)),
        }
    }
}

// TODO(cnpryer): Might be a [lints] bug.
#[allow(clippy::too_many_lines)]
fn exec_command(cmd: Commands, config: &mut Config) -> HuakResult<()> {
    match cmd {
        Commands::Activate => activate(config),
        Commands::Add {
            dependencies,
            group,
            trailing,
        } => {
            let options = AddOptions {
                install_options: InstallOptions { values: trailing },
            };
            add(&dependencies, group.as_ref(), config, &options)
        }
        Commands::Build { trailing } => {
            let options = BuildOptions {
                values: trailing,
                install_options: InstallOptions { values: None },
            };
            build(config, &options)
        }
        Commands::Clean {
            include_pyc,
            include_pycache,
        } => {
            let options = CleanOptions {
                include_pycache,
                include_compiled_bytecode: include_pyc,
            };
            clean(config, &options)
        }
        Commands::Completion { shell } => {
            let options = CompletionOptions { shell };
            completion(&options);
            Ok(())
        }
        Commands::Fix { trailing } => {
            let options = LintOptions {
                values: trailing,
                include_types: false,
                install_options: InstallOptions { values: None },
            };
            fix(config, &options)
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
            let options = FormatOptions {
                values: Some(args),
                install_options: InstallOptions { values: None },
            };
            fmt(config, &options)
        }
        Commands::Init { app, lib, no_vcs } => {
            config.workspace_root = config.cwd.clone();
            let options = WorkspaceOptions { uses_git: !no_vcs };
            init(app, lib, config, &options)
        }
        Commands::Install { groups, trailing } => {
            let options = InstallOptions { values: trailing };
            install(groups, config, &options)
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
            let options = LintOptions {
                values: Some(args),
                include_types: !no_types,
                install_options: InstallOptions { values: None },
            };
            lint(config, &options)
        }
        Commands::New {
            path,
            app,
            lib,
            no_vcs,
        } => {
            config.workspace_root = PathBuf::from(path);
            let options = WorkspaceOptions { uses_git: !no_vcs };
            new(app, lib, config, &options)
        }
        Commands::Publish { trailing } => {
            let options = PublishOptions {
                values: trailing,
                install_options: InstallOptions { values: None },
            };
            publish(config, &options)
        }
        Commands::Python { command } => python(command, config),
        Commands::Remove {
            dependencies,
            trailing,
        } => {
            let options = RemoveOptions {
                install_options: InstallOptions { values: trailing },
            };
            remove(&dependencies, config, &options)
        }
        Commands::Run { command } => run(&command, config),
        Commands::Test { trailing } => {
            let options = TestOptions {
                values: trailing,
                install_options: InstallOptions { values: None },
            };
            test(config, &options)
        }
        Commands::Update {
            dependencies,
            trailing,
        } => {
            let options = UpdateOptions {
                install_options: InstallOptions { values: trailing },
            };
            update(dependencies, config, &options)
        }
        Commands::Version => version(config),
    }
}

fn get_config(cwd: PathBuf, cli: &Cli) -> Config {
    // TODO: Use find_workspace_root
    let workspace_root =
        find_package_root(&cwd, &huak_home_dir().expect("home directory")).unwrap_or(cwd.clone());
    let verbosity = if cli.quiet {
        Verbosity::Quiet
    } else {
        Verbosity::Normal
    };
    let terminal_options = TerminalOptions {
        verbosity,
        ..Default::default()
    };
    let mut config = Config {
        workspace_root,
        cwd,
        terminal_options,
    };
    if cli.no_color {
        config.terminal_options = TerminalOptions {
            verbosity,
            color_choice: ColorChoice::Never,
        };
    }
    config
}

fn activate(config: &Config) -> HuakResult<()> {
    ops::activate_python_environment(config)
}

fn add(
    dependencies: &[Dependency],
    group: Option<&String>,
    config: &Config,
    options: &AddOptions,
) -> HuakResult<()> {
    let deps = dependencies
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    match group.as_ref() {
        Some(it) => ops::add_project_optional_dependencies(&deps, it, config, options),
        None => ops::add_project_dependencies(&deps, config, options),
    }
}

fn build(config: &Config, options: &BuildOptions) -> HuakResult<()> {
    ops::build_project(config, options)
}

fn clean(config: &Config, options: &CleanOptions) -> HuakResult<()> {
    ops::clean_project(config, options)
}

fn fix(config: &Config, options: &LintOptions) -> HuakResult<()> {
    ops::lint_project(config, options)
}

fn fmt(config: &Config, options: &FormatOptions) -> HuakResult<()> {
    ops::format_project(config, options)
}

fn init(app: bool, _lib: bool, config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    if app {
        ops::init_app_project(config, options)
    } else {
        ops::init_lib_project(config, options)
    }
}

#[allow(clippy::needless_pass_by_value)]
fn install(
    groups: Option<Vec<String>>,
    config: &Config,
    options: &InstallOptions,
) -> HuakResult<()> {
    ops::install_project_dependencies(groups.as_ref(), config, options)
}

fn lint(config: &Config, options: &LintOptions) -> HuakResult<()> {
    ops::lint_project(config, options)
}

fn new(app: bool, _lib: bool, config: &Config, options: &WorkspaceOptions) -> HuakResult<()> {
    if app {
        ops::new_app_project(config, options)
    } else {
        ops::new_lib_project(config, options)
    }
}

fn publish(config: &Config, options: &PublishOptions) -> HuakResult<()> {
    ops::publish_project(config, options)
}

fn python(command: Python, config: &Config) -> HuakResult<()> {
    match command {
        Python::List => ops::list_python(config),
        Python::Use { version } => ops::use_python(&version.to_string(), config),
        Python::Install { version } => ops::install_python(&version),
    }
}

fn remove(dependencies: &[String], config: &Config, options: &RemoveOptions) -> HuakResult<()> {
    ops::remove_project_dependencies(dependencies, config, options)
}

fn run(command: &[String], config: &Config) -> HuakResult<()> {
    ops::run_command_str(&command.join(" "), config)
}

fn test(config: &Config, options: &TestOptions) -> HuakResult<()> {
    ops::test_project(config, options)
}

fn update(
    dependencies: Option<Vec<String>>,
    config: &Config,
    options: &UpdateOptions,
) -> HuakResult<()> {
    ops::update_project_dependencies(dependencies, config, options)
}

fn version(config: &Config) -> HuakResult<()> {
    ops::display_project_version(config)
}

fn completion(options: &CompletionOptions) {
    generate_shell_completion_script(options.shell);
}

struct CompletionOptions {
    shell: Option<Shell>,
}

fn generate_shell_completion_script(shell: Option<Shell>) {
    let mut cmd = Cli::command();
    clap_complete::generate(
        shell.unwrap_or(Shell::Bash),
        &mut cmd,
        "huak",
        &mut std::io::stdout(),
    );
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
        self.0.clone()
    }
}
