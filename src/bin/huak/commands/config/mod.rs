mod completion;

use clap::Subcommand;
use clap_complete::Shell;

use crate::errors::CliResult;

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

pub fn run(command: Config) -> CliResult<()> {
    match command {
        Config::Completion {
            shell,
            install,
            uninstall,
        } => completion::run(shell, install, uninstall),
    }?;

    Ok(())
}
