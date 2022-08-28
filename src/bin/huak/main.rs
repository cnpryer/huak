use huak::errors::CliResult;

mod cli;
mod commands;
mod utils;

fn main() -> CliResult {
    cli::main()
}
