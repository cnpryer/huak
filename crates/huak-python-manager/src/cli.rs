use anyhow::Error;
use clap::{Parser, Subcommand};
use huak_python_manager::RequestedVersion;
use std::path::PathBuf;

/// A Python interpreter management system for Huak.
#[derive(Parser)]
#[command(version, author, about, arg_required_else_help = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, global = true)]
    quiet: bool,
    #[arg(long, global = true)]
    no_color: bool,
}

impl Cli {
    pub(crate) fn run(self) -> Result<(), Error> {
        match self.command {
            Commands::Install { version, target } => cmd::install(version, target),
        }
    }
}

// List of commands.
#[derive(Subcommand)]
#[clap(rename_all = "kebab-case")]
enum Commands {
    /// Install a Python interpreter.
    Install {
        #[arg(required = true)]
        /// Version of Python to install.
        version: RequestedVersion,

        /// Target path to install Python to.
        #[arg(long, required = true)]
        target: PathBuf,
    },
}

mod cmd {
    use std::path::PathBuf;

    use super::{Error, RequestedVersion};
    use huak_python_manager::{install_with_target, Options, Strategy};

    pub(crate) fn install(version: RequestedVersion, target: PathBuf) -> Result<(), Error> {
        println!("installing Python {version}...");

        let strategy = Strategy::Selection(Options {
            version: Some(version),
            ..Default::default()
        });

        install_with_target(&strategy, target)
    }
}