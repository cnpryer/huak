use crate::commands::Cli;
use crate::errors::CliResult;

use clap::CommandFactory;
use clap_complete::{generate, Shell};

/// Prints the script to stdout and a way to add the script to the shell init file to stderr. This
/// way if the user runs completion <shell> > completion.sh only the stdout will be redirected into
/// completion.sh.
pub fn run(shell: Shell) -> CliResult<()> {
    let mut cmd = Cli::command();

    // We can't take a reference to the cmd variable since we also need a mutable reference to this
    // in the generate() function.
    let cmd_name = String::from(Cli::command().get_name());

    generate(shell, &mut cmd, &cmd_name, &mut std::io::stdout());
    Ok(())
}
