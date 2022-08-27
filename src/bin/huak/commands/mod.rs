use clap::{self, Command};

pub(crate) mod activate;
pub(crate) mod add;
pub(crate) mod build;
pub(crate) mod clean;
pub(crate) mod help;
pub(crate) mod init;
pub(crate) mod new;
pub(crate) mod remove;
pub(crate) mod run;
pub(crate) mod update;
pub(crate) mod version;

pub fn args() -> Command<'static> {
    let mut app = app();

    let args = vec![
        activate::arg(),
        add::arg(),
        remove::arg(),
        build::arg(),
        clean::arg(),
        help::arg(),
        init::arg(),
        new::arg(),
        run::arg(),
        update::arg(),
        version::arg(),
    ];

    for arg in args {
        app = app.arg(arg);
    }

    app
}

fn app() -> Command<'static> {
    Command::new("huak")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("A Python package manager written in Rust inspired by Cargo")
}
