use crate::errors::CliResult;
use clap::{Parser, Subcommand};
use clap_complete::Shell;

pub(crate) mod activate;
pub(crate) mod add;
pub(crate) mod audit;
pub(crate) mod build;
pub(crate) mod clean;
pub(crate) mod completion;
pub(crate) mod doc;
pub(crate) mod fix;
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
    Completion {
        shell: Shell
    },

    /// Activate the project's virtual environment.
    Activate,
    /// Add a dependency to the existing project.
    Add {
        dependency: String,
        /// Adds an optional dependency.
        #[arg(long)]
        dev: bool,
    },
    /// Check for vulnerable dependencies and license compatibility.
    Audit,
    /// Build tarball and wheel for the project.
    Build,
    /// Remove tarball and wheel from the built project.
    Clean {
        #[arg(long, required = false)]
        /// Remove all .pyc files and __pycache__ directories.
        pycache: bool,
    },
    /// Generates documentation for the project.
    Doc {
        #[arg(long)]
        check: bool,
    },
    /// Auto-fix fixable lint conflicts
    Fix,
    /// Format the project's Python code.
    Fmt {
        /// Check if Python code is formatted.
        #[arg(long)]
        check: bool,
    },
    /// Initialize the existing project.
    Init,
    /// Install the dependencies of an existing project.
    Install {
        /// Install optional dependency groups
        #[arg(long, conflicts_with = "all", num_args = 1..)]
        groups: Option<Vec<String>>,
        /// Install main and all optional dependencies.
        #[arg(long)]
        all: bool,
    },
    /// Lint the project's Python code.
    Lint {
        #[arg(long, required = false)]
        fix: bool,
    },
    /// Create a new project at <path>.
    New {
        /// Use a library template.
        #[arg(long, conflicts_with = "app")]
        lib: bool,
        /// Use a application template [default].
        #[arg(long)]
        app: bool,
        /// Path and name of the python package
        path: String,
        /// Don't initialize VCS in the new project
        #[arg(long)]
        no_vcs: bool,
    },
    /// Builds and uploads current project to a registry.
    Publish,
    /// Remove a dependency from the project.
    Remove { dependency: String },
    /// Run a command within the project's environment context.
    Run {
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Test the project's Python code.
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
            Commands::Completion { shell } => completion::run(shell),
            Commands::Activate => activate::run(),
            Commands::Add { dependency, dev } => add::run(dependency, dev),
            Commands::Audit => audit::run(),
            Commands::Build => build::run(),
            Commands::Clean { pycache } => clean::run(pycache),
            Commands::Doc { check } => doc::run(check),
            Commands::Fix => fix::run(),
            Commands::Fmt { check } => fmt::run(check),
            Commands::Init => init::run(),
            Commands::Install { groups, all } => install::run(groups, all),
            Commands::Lint { fix } => lint::run(fix),
            // --lib is the default, so it's unnecessary to handle. If --app is not passed, assume --lib.
            #[allow(unused_variables)]
            Commands::New {
                path,
                app,
                lib,
                no_vcs,
            } => new::run(path, app, no_vcs),
            Commands::Publish => publish::run(),
            Commands::Remove { dependency } => remove::run(dependency),
            Commands::Run { command } => run::run(command),
            Commands::Test => test::run(),
            Commands::Update { dependency } => update::run(dependency),
            Commands::Version => version::run(),
        }
    }
}
