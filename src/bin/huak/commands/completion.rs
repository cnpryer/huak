use crate::errors::CliResult;

use clap::CommandFactory;
use clap_complete::{Shell, generate};



pub fn run(shell: Shell) -> CliResult<()> {
    let mut cmd = crate::commands::Cli::command();

    generate(shell, &mut cmd, "huak", &mut std::io::stdout());
    Ok(())
}
