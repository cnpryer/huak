use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;

use clap::{Command, Subcommand};
use clap_complete::{generate, Shell};

/// Bash has a couple of files that can contain the actual completion script.
/// Only the line `eval "$(huak config completion bash)"` needs to be added
/// These files are loaded in the following order:
/// ~/.bash_profile
/// ~/.bash_login
/// ~/.profile
/// ~/.bashrc
pub fn add_completion_bash() -> Result<(), Error> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => {
            // defaulting to root, this might not be the right call
            eprintln!("{}", e);
            String::from("root")
        }
    };

    let _file_path = format!("{}/.bashrc", home);

    #[cfg(test)]
    let _file_path = format!("test_files/test_bashrc");

    // opening file in append mode
    let mut file: File = File::options().append(true).open(_file_path)?;

    // This needs to be a string since there will be a \n prepended if it is
    file.write_all(
        format!(r##"{}eval "$(huak config completion bash)"{}"##, '\n', '\n')
            .to_string()
            .as_bytes(),
    )?;

    Ok(())
}

// TODO
pub fn add_completion_elvish() -> Result<(), Error> {
    todo!()
}

/// huak config completion fish > ~/.config/fish/completions/huak.fish
/// Fish has a completions directory in which all files are loaded on init.
/// The naming convention is $HOME/.config/fish/completions/huak.fish
pub fn add_completion_fish(cli: &mut Command) -> Result<(), Error> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => {
            // defaulting to root, this might not be the right call
            eprintln!("{}", e);
            String::from("root")
        }
    };

    let _target_file = format!("{}/.config/fish/completions/huak.fish", home);

    #[cfg(test)]
    let _target_file = "test_files/test_fish".to_string();

    generate_target_file(_target_file, cli)?;
    Ok(())
}

// TODO
pub fn add_completion_powershell() -> Result<(), Error> {
    todo!()
}

/// Zsh and fish are the same in the sense that the use an entire directory to collect shell init
/// scripts.
pub fn add_completion_zsh(cli: &mut Command) -> Result<(), Error> {
    let _target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();

    #[cfg(test)]
    let _target_file = "test_files/test_zsh".to_string();

    generate_target_file(_target_file, cli)?;
    Ok(())
}

/// Reads the entire file and removes lines that match exactly with:
/// \neval "$(huak config completion)
pub fn remove_completion_bash() -> Result<(), Error> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => {
            // defaulting to root, this might not be the right call
            eprintln!("{}", e);
            String::from("root")
        }
    };

    let _file_path = format!("{}/.bashrc", home);

    #[cfg(test)]
    let _file_path = format!("test_files/test_bashrc");

    let file_content = std::fs::read_to_string(&_file_path)?;
    let new_content = file_content.replace(
        &format!(r##"{}eval "$(huak config completion bash)"{}"##, '\n', '\n'),
        "",
    );

    std::fs::write(&_file_path, new_content)?;

    Ok(())
}

// TODO
pub fn remove_completion_elvish() -> Result<(), Error> {
    todo!()
}

pub fn remove_completion_fish() -> Result<(), Error> {
    let home = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(e) => {
            // defaulting to root, this might not be the right call
            eprintln!("{}", e);
            String::from("root")
        }
    };

    let _target_file = format!("{}/.config/fish/completions/huak.fish", home);

    #[cfg(test)]
    let _target_file = "test_files/test_fish".to_string();

    std::fs::remove_file(_target_file)?;

    Ok(())
}

// TODO
pub fn remove_completion_powershell() -> Result<(), Error> {
    todo!()
}

pub fn remove_completion_zsh() -> Result<(), Error> {
    let _target_file = "/usr/local/share/zsh/site-functions/_huak".to_string();

    #[cfg(test)]
    let _target_file = "test_files/test_zsh".to_string();

    std::fs::remove_file(_target_file)?;

    Ok(())
}

fn generate_target_file<P>(
    target_file: P,
    cmd: &mut Command,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let mut file: File = File::create(&target_file)?;

    generate(Shell::Fish, cmd, "huak", &mut file);

    Ok(())
}

#[derive(Debug, Subcommand)]
pub enum Config {
    /// Generates a shell completion script for supported shells.
    /// See the help menu for more information on supported shells.
    Completion { shell: Shell },
    /// Installs the completion script in your shell init file.
    Install { shell: Shell },
    /// Uninstalls the completion script from your shell init file.
    Uninstall { shell: Shell },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use clap::{Command, CommandFactory, Parser};

    #[derive(Parser)]
    struct Cli {}

    #[test]
    /// This test ensures the order of operations is always correct
    fn test_bash_completion() {
        test_adding_completion_bash();
        test_remove_completion_bash();
    }

    fn test_adding_completion_bash() {
        let _ = add_completion_bash();

        let file_content =
            fs::read_to_string("test_files/test_bashrc").unwrap();

        assert_eq!(
            format!(
                r##"# This stuff is in here so there is something that should be left over after
# removing the bash completion script


eval "$(huak config completion bash)"
"##
            ),
            file_content
        )
    }

    fn test_remove_completion_bash() {
        let _ = remove_completion_bash();

        let file_content =
            fs::read_to_string("test_files/test_bashrc").unwrap();

        assert_eq!("# This stuff is in here so there is something that should be left over after
# removing the bash completion script

", file_content)
    }

    #[test]
    /// This test ensures the order of operations is always correct
    fn test_fish_completion() {
        let mut cmd = Cli::command();

        test_adding_completion_fish(&mut cmd);
        test_remove_completion_fish();
    }

    fn test_adding_completion_fish(cmd: &mut Command) {
        let _ = add_completion_fish(cmd);

        let result = std::fs::read_to_string("test_files/test_fish");

        assert_eq!(true, result.is_ok());
    }

    fn test_remove_completion_fish() {
        let _ = remove_completion_fish();

        let result = std::fs::read("test_files/test_fish");
        assert_eq!(true, result.is_err());
    }

    #[test]
    /// This test ensures the order of operations is always correct
    fn test_zsh_completion() {
        let mut cmd = Cli::command();

        test_adding_completion_zsh(&mut cmd);
        test_remove_completion_zsh();
    }

    fn test_adding_completion_zsh(cmd: &mut Command) {
        let _ = add_completion_zsh(cmd);

        let result = std::fs::read_to_string("test_files/test_zsh");

        assert_eq!(true, result.is_ok());
    }

    fn test_remove_completion_zsh() {
        let _ = remove_completion_zsh();

        let result = std::fs::read("test_files/test_zsh");
        assert_eq!(true, result.is_err());
    }
}
