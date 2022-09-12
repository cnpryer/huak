use clap::{self, Command};

/// Application namespace for `activate` command.
pub(crate) mod activate;
/// Application namespace for `add` command.
pub(crate) mod add;
/// Application namespace for `build` command.
pub(crate) mod build;
/// Application namespace for `clean` command.
pub(crate) mod clean;
/// Application namespace for `clean_pycache` command.
pub(crate) mod clean_pycache;
/// Application namespace for `fmt` command.
pub(crate) mod fmt;
/// Application namespace for `help` command.
pub(crate) mod help;
/// Application namespace for `init` command.
pub(crate) mod init;
/// Application namespace for `install` command.
pub(crate) mod install;
/// Application namespace for `lint` command.
pub(crate) mod lint;
/// Application namespace for `new` command.
pub(crate) mod new;
/// Application namespace for `remove` command.
pub(crate) mod remove;
/// Application namespace for `run` command.
pub(crate) mod run;
/// Application namespace for `test` command.
pub(crate) mod test;
/// Application namespace for `update` command.
pub(crate) mod update;
/// Application namespace for `utils` command.
pub(crate) mod utils;
/// Application namespace for `version` command.
pub(crate) mod version;

/// Get the command application.
pub fn args() -> Command<'static> {
    let mut app = _app();

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

/// Get the application.
fn _app() -> Command<'static> {
    Command::new("huak")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("A Python package manager written in Rust inspired by Cargo")
}
