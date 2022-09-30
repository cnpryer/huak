use clap::{Parser, Args, Subcommand};

pub(crate) mod activate;
pub(crate) mod add;
pub(crate) mod build;
pub(crate) mod clean;
pub(crate) mod clean_pycache;
pub(crate) mod doc;
pub(crate) mod fmt;
pub(crate) mod help;
pub(crate) mod init;
pub(crate) mod install;
pub(crate) mod lint;
pub(crate) mod new;
pub(crate) mod publish;
pub(crate) mod remove;
pub(crate) mod run;
pub(crate) mod test;
pub(crate) mod update;
pub(crate) mod utils;
pub(crate) mod version;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // Activate the project's virtual environment.
    Activate,
    // Add a python module to the existing project.
    Add {
        dependency: String,
        // Adds an optional dependency.
        dev: bool
    },
    // Build tarball and wheel for the project.
    Build,
    // Remove tarball and wheel from the built project.
    Clean,
    // Remove all .pyc files and __pycache__ directores.
    #[command(name = "clean-pycache")]
    Cleanpycache,
    // Builds and uploads current project to a registry.
    Doc {
        check: bool,
    },
    //Check if Python code is formatted.
    Fmt {
        check: bool
    },
    // Display Huak commands and general usage information.
    Help,
    // Initialize the existing project.
    Init,
    // Install the dependencies of an existing project.
    Install,
    // Lint Python code.
    Lint,
    // Create a project from scratch.
    New {
        path: String
    },
    // Builds and uploads current project to a registry.
    Publish,
    // Remove a dependency from the project.
    Remove {
        dependency: String,
    },
    // Run a command within the project's environment context.
    Run {
        command: String,
    },
    // Test Python Code.
    Test,
    // Update dependencies added to the project.
    Update {
        dependency: String,
    },
    // Display the version of the project.
    Version,

}

/*
pub fn args() -> Command<'static> {
    let mut app = Command::new("huak");

    let subcommands = vec![
        activate::cmd(),
        add::cmd(),
        build::cmd(),
        clean::cmd(),
        clean_pycache::cmd(),
        doc::cmd(),
        help::cmd(),
        fmt::cmd(),
        init::cmd(),
        install::cmd(),
        lint::cmd(),
        new::cmd(),
        publish::cmd(),
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
*/
