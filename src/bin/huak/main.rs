use huak::errors::CliResult;

mod cli;
mod commands;

#[cfg(test)]
mod test_utils;

fn main() -> CliResult {
    cli::main()
}
