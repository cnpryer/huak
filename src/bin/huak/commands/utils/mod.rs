use clap::{self, App, AppSettings};

/// Create a clap subcommand.
pub fn subcommand(name: &'static str) -> clap::Command<'static> {
    App::new(name)
        .dont_collapse_args_in_usage(true)
        .setting(AppSettings::DeriveDisplayOrder)
}
