mod completion;

use crate::commands::Cli;
use crate::errors::CliResult;

use clap::{Args, Command, CommandFactory, Subcommand};
use clap_complete::{generate, Shell};

/// Prints the script to stdout and a way to add the script to the shell init file to stderr. This
/// way if the user runs completion <shell> > completion.sh only the stdout will be redirected into
/// completion.sh.
pub fn run(config_command: Config) -> CliResult<()> {
    match config_command.command {
        ConfigCommand::Completion {
            shell,
            install,
            uninstall,
        } => {
            if install {
                let mut cmd: Command = Cli::command();
                let _result = match shell {
                    Shell::Bash => completion::add_completion_bash(),
                    Shell::Elvish => completion::add_completion_elvish(),
                    Shell::Fish => completion::add_completion_fish(&mut cmd),
                    Shell::PowerShell => {
                        completion::add_completion_powershell()
                    }
                    Shell::Zsh => completion::add_completion_zsh(&mut cmd),
                    _ => Ok(()),
                };
            } else if uninstall {
                let _result = match shell {
                    Shell::Bash => completion::remove_completion_bash(),
                    Shell::Elvish => completion::remove_completion_elvish(),
                    Shell::Fish => completion::remove_completion_fish(),
                    Shell::PowerShell => {
                        completion::remove_completion_powershell()
                    }
                    Shell::Zsh => completion::remove_completion_zsh(),
                    _ => Ok(()),
                };
            } else {
                generate_shell_completion_script(shell)
            }
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
pub enum ConfigCommand {
    /// Generates a shell completion script for supported shells.
    /// See the help menu for more information on supported shells.
    Completion {
        #[arg(short, long, value_name = "shell")]
        shell: Shell,

        #[arg(short, long)]
        /// Installs the completion script in your shell init file.
        install: bool,

        #[arg(short, long)]
        /// Uninstalls the completion script from your shell init file.
        uninstall: bool,
    },
}
