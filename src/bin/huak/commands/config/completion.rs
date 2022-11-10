use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::process::ExitCode;

use clap::{Command, CommandFactory};
use clap_complete::{generate, Shell};
use huak::errors::HuakError;

use crate::commands::Cli;
use crate::errors::{CliError, CliResult};

/// Prints the script to stdout and a way to add the script to the shell init file to stderr. This
/// way if the user runs completion <shell> > completion.sh only the stdout will be redirected into
/// completion.sh.
pub fn run(
    shell: Option<Shell>,
    install: bool,
    uninstall: bool,
) -> CliResult<()> {
    if (install || uninstall) && shell.is_none() {
        return Err(CliError::new(
            HuakError::ConfigurationError("No shell provided".to_string()),
            ExitCode::FAILURE,
        ));
    }
    if install {
        run_with_install(shell)?;
    } else if uninstall {
        run_with_uninstall(shell)?;
    } else {
        generate_shell_completion_script()
    }
    Ok(())
}

fn generate_shell_completion_script() {
    let mut cmd = Cli::command();

    generate(Shell::Bash, &mut cmd, "huak", &mut std::io::stdout())
}

fn run_with_install(shell: Option<Shell>) -> CliResult<()> {
    let sh = match shell {
        Some(it) => it,
        None => {
            return Err(CliError::new(
                HuakError::ConfigurationError("No shell provided".to_string()),
                ExitCode::FAILURE,
            ))
        }
    };
    let mut cmd: Command = Cli::command();
    match sh {
        Shell::Bash => add_completion_bash(),
        Shell::Elvish => add_completion_elvish(),
        Shell::Fish => add_completion_fish(&mut cmd),
        Shell::PowerShell => add_completion_powershell(),
        Shell::Zsh => add_completion_zsh(&mut cmd),
        _ => {
            return Err(CliError::new(
                HuakError::ConfigurationError("Invalid shell".to_string()),
                ExitCode::FAILURE,
            ));
        }
    }?;

    Ok(())
}

fn run_with_uninstall(shell: Option<Shell>) -> CliResult<()> {
    let sh = match shell {
        Some(it) => it,
        None => {
            return Err(CliError::new(
                HuakError::ConfigurationError("No shell provided".to_string()),
                ExitCode::FAILURE,
            ))
        }
    };
    match sh {
        Shell::Bash => remove_completion_bash(),
        Shell::Elvish => remove_completion_elvish(),
        Shell::Fish => remove_completion_fish(),
        Shell::PowerShell => remove_completion_powershell(),
        Shell::Zsh => remove_completion_zsh(),
        _ => {
            return Err(CliError::new(
                HuakError::ConfigurationError("Invalid shell".to_string()),
                ExitCode::FAILURE,
            ));
        }
    }?;

    Ok(())
}

/// Bash has a couple of files that can contain the actual completion script.
/// Only the line `eval "$(huak config completion bash)"` needs to be added
/// These files are loaded in the following order:
/// ~/.bash_profile
/// ~/.bash_login
/// ~/.profile
/// ~/.bashrc
pub fn add_completion_bash() -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(CliError::from(e)),
    };

    let file_path = format!("{}/.bashrc", home);

    // opening file in append mode
    let mut file = File::options().append(true).open(file_path)?;

    // This needs to be a string since there will be a \n prepended if it is
    file.write_all(
        format!(r##"{}eval "$(huak config completion)"{}"##, '\n', '\n')
            .as_bytes(),
    )?;

    Ok(())
}

// TODO
pub fn add_completion_elvish() -> CliResult<()> {
    todo!()
}

/// huak config completion fish > ~/.config/fish/completions/huak.fish
/// Fish has a completions directory in which all files are loaded on init.
/// The naming convention is $HOME/.config/fish/completions/huak.fish
pub fn add_completion_fish(cli: &mut Command) -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(CliError::from(e)),
    };

    let target_file = format!("{}/.config/fish/completions/huak.fish", home);

    generate_target_file(target_file, cli)?;
    Ok(())
}

// TODO
pub fn add_completion_powershell() -> CliResult<()> {
    todo!()
}

/// Zsh and fish are the same in the sense that the use an entire directory to collect shell init
/// scripts.
pub fn add_completion_zsh(cli: &mut Command) -> CliResult<()> {
    let target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();

    generate_target_file(target_file, cli)?;
    Ok(())
}

/// Reads the entire file and removes lines that match exactly with:
/// \neval "$(huak config completion)
pub fn remove_completion_bash() -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(CliError::from(e)),
    };

    let file_path = format!("{}/.bashrc", home);

    let file_content = std::fs::read_to_string(&file_path)?;
    let new_content = file_content.replace(
        &format!(r##"{}eval "$(huak config completion)"{}"##, '\n', '\n'),
        "",
    );

    std::fs::write(&file_path, new_content)?;

    Ok(())
}

// TODO
pub fn remove_completion_elvish() -> CliResult<()> {
    unimplemented!()
}

pub fn remove_completion_fish() -> CliResult<()> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => return Err(CliError::from(e)),
    };

    let target_file = format!("{}/.config/fish/completions/huak.fish", home);

    std::fs::remove_file(target_file)?;

    Ok(())
}

// TODO
pub fn remove_completion_powershell() -> CliResult<()> {
    unimplemented!()
}

pub fn remove_completion_zsh() -> CliResult<()> {
    let target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();

    std::fs::remove_file(target_file)?;

    Ok(())
}

fn generate_target_file<P>(
    target_file: P,
    cmd: &mut Command,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let mut file = File::create(&target_file)?;

    generate(Shell::Fish, cmd, "huak", &mut file);

    Ok(())
}

// TODO:
//   - Use tempdir and mocking for testing these features.
//   - Requires refactors of functions and their signatures.
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use clap::{Command, Parser};

    #[derive(Parser)]
    struct Cli {}

    #[cfg(target_family = "unix")]
    #[ignore = "incomplete test"] // See TODO
    #[test]
    /// This test ensures the order of operations is always correct
    fn test_bash_completion() {
        test_adding_completion_bash();
        test_remove_completion_bash();
    }

    #[cfg(target_family = "unix")]
    fn test_adding_completion_bash() {
        let _ = add_completion_bash();
        // TODO: Use tempdir
        let file_content =
            fs::read_to_string("resources/test_files/test_bashrc").unwrap();

        assert_eq!(
            format!(
                r##"# This stuff is in here so there is something that should be left over after
# removing the bash completion script


eval "$(huak config completion)"
"##
            ),
            file_content
        )
    }

    #[cfg(target_family = "unix")]
    fn test_remove_completion_bash() {
        let _ = remove_completion_bash();
        // TODO: Use tempdir
        let file_content =
            fs::read_to_string("resources/test_files/test_bashrc").unwrap();

        assert_eq!("# This stuff is in here so there is something that should be left over after
# removing the bash completion script

", file_content)
    }

    #[cfg(target_family = "unix")]
    #[ignore = "incomplete test"] // See TODO
    #[test]
    /// This test ensures the order of operations is always correct
    fn test_fish_completion() {
        let mut cmd = Cli::command();

        test_adding_completion_fish(&mut cmd);
        test_remove_completion_fish();
    }

    #[cfg(target_family = "unix")]
    fn test_adding_completion_fish(cmd: &mut Command) {
        let _ = add_completion_fish(cmd);
        // TODO: Use tempdir
        let result = std::fs::read_to_string("resources/test_files/test_fish");

        assert_eq!(true, result.is_ok());
    }

    #[cfg(target_family = "unix")]
    fn test_remove_completion_fish() {
        let _ = remove_completion_fish();
        // TODO: Use tempdir
        let result = std::fs::read("resources/test_files/test_fish");
        assert_eq!(true, result.is_err());
    }

    #[cfg(target_family = "unix")]
    #[ignore = "incomplete test"] // See TODO
    #[test]
    /// This test ensures the order of operations is always correct
    fn test_zsh_completion() {
        let mut cmd = Cli::command();

        test_adding_completion_zsh(&mut cmd);
        test_remove_completion_zsh();
    }

    #[cfg(target_family = "unix")]
    fn test_adding_completion_zsh(cmd: &mut Command) {
        let _ = add_completion_zsh(cmd);
        // TODO: Use tempdir
        let result = std::fs::read_to_string("resources/test_files/test_zsh");

        assert_eq!(true, result.is_ok());
    }

    #[cfg(target_family = "unix")]
    fn test_remove_completion_zsh() {
        let _ = remove_completion_zsh();
        // TODO: Use tempdir
        let result = std::fs::read("resources/test_files/test_zsh");
        assert_eq!(true, result.is_err());
    }
}
