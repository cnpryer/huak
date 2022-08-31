use clap::{self, Command};

pub(crate) mod activate;
pub(crate) mod add;
pub(crate) mod build;
pub(crate) mod clean;
pub(crate) mod clean_pycache;
pub(crate) mod fmt;
pub(crate) mod help;
pub(crate) mod init;
pub(crate) mod lint;
pub(crate) mod new;
pub(crate) mod remove;
pub(crate) mod run;
pub(crate) mod update;
pub(crate) mod utils;
pub(crate) mod version;

pub fn args() -> Command<'static> {
    let mut app = app();

    let subcommands = vec![
        activate::arg(),
        add::arg(),
        build::arg(),
        clean::arg(),
        clean_pycache::arg(),
        help::arg(),
        fmt::arg(),
        init::arg(),
        lint::arg(),
        new::arg(),
        remove::arg(),
        run::arg(),
        update::arg(),
        version::arg(),
    ];

    for cmd in subcommands {
        app = app.subcommand(cmd)
    }

    app
}

fn app() -> Command<'static> {
    Command::new("huak")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("A Python package manager written in Rust inspired by Cargo")
}
