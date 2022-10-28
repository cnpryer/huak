use crate::commands::Cli;
use crate::errors::CliResult;

use clap::{Args, CommandFactory, Subcommand};
use clap_complete::{generate, Shell};

/// Prints the script to stdout and a way to add the script to the shell init file to stderr. This
/// way if the user runs completion <shell> > completion.sh only the stdout will be redirected into
/// completion.sh.
pub fn run(config_command: Config) -> CliResult<()> {
    match config_command.command {
        ConfigCommand::Completion { shell } => {
            generate_shell_completion_script(Some(shell))
        }
    }
    Ok(())
}

fn generate_shell_completion_script(shell: Option<Shell>) {
    let mut cmd = Cli::command();

    // We can't take a reference to the cmd variable since we also need a mutable reference to this
    // in the generate() function.
    let cmd_name = String::from(Cli::command().get_name());

    generate(shell.unwrap(), &mut cmd, &cmd_name, &mut std::io::stdout())
}

#[derive(Args)]
pub struct Config {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Generates a shell completion script for supported shells.
    /// See the help menu for more information on supported shells.
    Completion { shell: Shell },
}
