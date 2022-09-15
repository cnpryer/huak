use clap::{self, Command};

pub(crate) mod activate;
pub(crate) mod add;
pub(crate) mod build;
pub(crate) mod clean;
pub(crate) mod clean_pycache;
pub(crate) mod fmt;
pub(crate) mod help;
pub(crate) mod init;
pub(crate) mod install;
pub(crate) mod lint;
pub(crate) mod new;
pub(crate) mod remove;
pub(crate) mod run;
pub(crate) mod test;
pub(crate) mod update;
pub(crate) mod utils;
pub(crate) mod version;

pub fn args() -> Command<'static> {
    let mut app = Command::new("huak")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("A Python package manager written in Rust inspired by Cargo");

    let subcommands = vec![
        activate::cmd(),
        add::cmd(),
        build::cmd(),
        clean::cmd(),
        clean_pycache::cmd(),
        help::cmd(),
        fmt::cmd(),
        init::cmd(),
        install::cmd(),
        lint::cmd(),
        new::cmd(),
        remove::cmd(),
        run::cmd(),
        test::cmd(),
        update::cmd(),
        version::cmd(),
    ];

    for cmd in subcommands {
        app = app.subcommand(cmd)
    }

    app
}
