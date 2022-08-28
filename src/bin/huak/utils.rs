use clap::{App, AppSettings, Command};

pub fn subcommand(name: &'static str) -> Command<'static> {
    App::new(name)
        .dont_collapse_args_in_usage(true)
        .setting(AppSettings::DeriveDisplayOrder)
}
