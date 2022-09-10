use huak::errors::CliResult;

mod cli;
mod commands;

fn main() -> CliResult {
    cli::main()
}
