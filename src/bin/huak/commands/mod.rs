use crate::errors::CliResult;
use clap::{Parser, Subcommand};

pub(crate) mod activate;
pub(crate) mod add;
pub(crate) mod build;
pub(crate) mod clean;
pub(crate) mod clean_pycache;
pub(crate) mod doc;
pub(crate) mod fmt;
pub(crate) mod init;
pub(crate) mod install;
pub(crate) mod lint;
pub(crate) mod new;
pub(crate) mod publish;
pub(crate) mod remove;
pub(crate) mod run;
pub(crate) mod test;
pub(crate) mod update;
pub(crate) mod version;

// Main CLI struct.

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
    /// Add a python module to the existing project.
    Add {
        dependency: String,
        /// Adds an optional dependency.
        #[arg(long)]
        dev: bool,
    },
    /// Build tarball and wheel for the project.
    Build,
    /// Remove tarball and wheel from the built project.
    Clean,
    /// Remove all .pyc files and __pycache__ directories.
    #[command(name = "clean-pycache")]
    Cleanpycache,
    /// Builds and uploads current project to a registry.
    Doc {
        /// Check if Python code is formatted.
        #[arg(long)]
        check: bool,
    },
    /// Format Python code.
    Fmt {
        /// Check if Python code is formatted.
        #[arg(long)]
        check: bool,
    },
    /// Initialize the existing project.
    Init,
    /// Install the dependencies of an existing project.
    Install {
        /// Install main and all optional dependencies.
        #[arg(long)]
        all: bool,
    },
    /// Lint Python code.
    Lint,
    /// Create a project from scratch.
    New {
        /// Create a library.
        #[arg(long, conflicts_with = "app")]
        lib: bool,
        /// Create a runnable application.
        #[arg(long)]
        app: bool,
        path: Option<String>,
    },
    /// Builds and uploads current project to a registry.
    Publish,
    /// Remove a dependency from the project.
    Remove { dependency: String },
    /// Run a command within the project's environment context.
    Run { command: String },
    /// Test Python Code.
    Test,
    /// Update dependencies added to the project.
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
            Commands::Activate => activate::run(),
            Commands::Add { dependency, dev } => add::run(dependency, dev),
            Commands::Build => build::run(),
            Commands::Clean => clean::run(),
            Commands::Cleanpycache => clean_pycache::run(),
            Commands::Doc { check } => doc::run(check),
            Commands::Fmt { check } => fmt::run(check),
            Commands::Init => init::run(),
            Commands::Install { all } => install::run(all),
            Commands::Lint => lint::run(),
            Commands::New { path, app, lib } => new::run(path, app, lib),
            Commands::Publish => publish::run(),
            Commands::Remove { dependency } => remove::run(dependency),
            Commands::Run { command } => run::run(command),
            Commands::Test => test::run(),
            Commands::Update { dependency } => update::run(dependency),
            Commands::Version => version::run(),
        }
    }
}
