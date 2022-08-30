use huak::errors::CliResult;

mod cli;
mod commands;
mod pyproject;
mod utils;

fn main() -> CliResult {
    cli::main()
}
