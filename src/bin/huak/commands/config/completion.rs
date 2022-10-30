use crate::commands::Cli;
use crate::errors::CliResult;

use huak::ops::config;

use clap::{Args, Command, CommandFactory, Subcommand};
use clap_complete::{generate, Shell};

/// Prints the script to stdout and a way to add the script to the shell init file to stderr. This
/// way if the user runs completion <shell> > completion.sh only the stdout will be redirected into
/// completion.sh.
pub fn run(config_command: Config) -> CliResult<()> {
    match config_command.command {
        ConfigCommand::Completion { shell } => {
            generate_shell_completion_script(shell)
        }
        ConfigCommand::Install { shell } => {
            let mut cmd: Command = Cli::command();
            let _result = match shell {
                Shell::Bash => config::_add_completion_bash(),
                Shell::Elvish => config::_add_completion_elvish(),
                Shell::Fish => config::_add_completion_fish(&mut cmd),
                Shell::PowerShell => config::_add_completion_powershell(),
                Shell::Zsh => config::_add_completion_zsh(&mut cmd),
                _ => Ok(()),
            };
        }
        ConfigCommand::Uninstall { shell } => {
            let _result = match shell {
                Shell::Bash => config::_remove_completion_bash(),
                Shell::Elvish => config::_remove_completion_elvish(),
                Shell::Fish => config::_remove_completion_fish(),
                Shell::PowerShell => config::_remove_completion_powershell(),
                Shell::Zsh => config::_remove_completion_zsh(),
                _ => Ok(()),
            };
        }
    }
    Ok(())
}

fn generate_shell_completion_script(shell: Shell) {
    let mut cmd = Cli::command();

    generate(shell, &mut cmd, "huak", &mut std::io::stdout())
}

#[derive(Args)]
pub struct Config {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub enum Config {
    /// Generates a shell completion script for supported shells.
    /// See the help menu for more information on supported shells.
    Completion { shell: Shell },
    /// Installs the completion script in your shell init file.
    Install { shell: Shell },
    /// Uninstalls the completion script from your shell init file.
    Uninstall { shell: Shell },
}
