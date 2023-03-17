use crate::errors::CliResult;

use clap::{Parser, Subcommand};


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
    },
    /// Check for vulnerable dependencies and license compatibility*.
    Audit,
    /// Build tarball and wheel for the project.
    Build,
    /// Interact with the configuration of huak.
    Config {
        #[command(subcommand)]
        command: config::Config,
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
    Fix,
    /// Format the project's Python code.
    Fmt {
        /// Check if Python code is formatted.
        #[arg(long)]
        check: bool,
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
    },
    /// Lint the project's Python code.
    Lint {
        #[arg(long, required = false)]
        fix: bool,
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
    Publish,
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
    Test,
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
            Commands::Add { dependency, group } => add(dependency, group),
            Commands::Audit => audit(),
            Commands::Build => build(),
            Commands::Clean { pycache } => clean(pycache),
            Commands::Doc { check } => doc(check),
            Commands::Fix => fix::run(),
            Commands::Fmt { check } => fmt(check),
            // --lib is the default, so it's unnecessary to handle. If --app is not passed, assume --lib.
            #[allow(unused_variables)]
            Commands::Init { app, lib } => init(app),
            Commands::Install { groups } => install(groups),
            Commands::Lint { fix } => lint(fix),
            // --lib is the default, so it's unnecessary to handle. If --app is not passed, assume --lib.
            #[allow(unused_variables)]
            Commands::New {
                path,
                app,
                lib,
                no_vcs,
            } => new(path, app, no_vcs),
            Commands::Publish => publish(),
            Commands::Remove { dependency, group } => {
                remove(dependency, group)
            }
            Commands::Run { command } => run(command),
            Commands::Test => test(),
            Commands::Update { dependency } => update(dependency),
            Commands::Version => version(),
        }
    }
}
