use crate::resolve::RequestedVersion;
use anyhow::Error;
use clap::{Parser, Subcommand};

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
            Commands::Install { version } => cmd::install(version),
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
        version: RequestedVersion,
    },
}

mod cmd {
    use super::{Error, RequestedVersion};
    use crate::install::install_to_home;
    use crate::resolve::{Options, Strategy};

    pub(crate) fn install(version: RequestedVersion) -> Result<(), Error> {
        println!("installing Python {version}");
        install_to_home(&Strategy::Selection(Options {
            version: Some(version),
            ..Default::default()
        }))
    }
}
