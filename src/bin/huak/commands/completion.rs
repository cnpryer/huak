use crate::commands::Cli;
use crate::errors::CliResult;

use clap::CommandFactory;
use clap_complete::{generate, Shell};

/// Prints the script to stdout and a way to add the script to the shell init file to stderr. This
/// way if the user runs completion <shell> > completion.sh only the stdout will be redirected into
/// completion.sh.
pub fn run(shell: Shell) -> CliResult<()> {
    let mut cmd = Cli::command();

    // We can't take a reference to the cmd variable since we also need a mutable reference to this
    // in the generate() function.
    let cmd_name = String::from(Cli::command().get_name());

    let help_output = match &shell {
        Shell::Bash => completion_help_bash(&cmd_name),
        Shell::Elvish => completion_help_elvish(&cmd_name),
        Shell::Fish => completion_help_fish(&cmd_name),
        Shell::PowerShell => completion_help_powershell(&cmd_name),
        Shell::Zsh => completion_help_zsh(&cmd_name),

        // We need this since the Shell enum is marked as non exhaustive
        _ => completion_help_unknown(&cmd_name),
    };

    generate(shell, &mut cmd, &cmd_name, &mut std::io::stdout());

    eprintln!("{}", help_output);
    Ok(())
}

fn completion_help_bash(cmd_name: &str) -> String {
    format!(
        r##""First, ensure that you install `bash-completion` using your package manager.
After, add this to your `~/.bash_profile`:

`eval "$({cmd_name} completion bash --rename {cmd_name})"`
"##
    )
}

// TODO
fn completion_help_elvish(_cmd_name: &str) -> String {
    r##"This shell is supported, but the suggestion as to how this should be added to your shell init file is missing.

If you are able to test this please head over to https://github.com/cnpryer/huak/issues and file an issue"##.to_string()
}

fn completion_help_fish(cmd_name: &str) -> String {
    format!(
        r##"Generate a `tool.fish` completion script:

`{cmd_name} completion fish --rename {cmd_name} > ~/.config/fish/completions/{cmd_name}.fish`
"##
    )
}

fn completion_help_powershell(cmd_name: &str) -> String {
    format!(
        r##"Open your profile script with:

`mkdir -Path (Split-Path -Parent $profile) -ErrorAction SilentlyContinue`
`notepad $profile`

Add the line and save the file:
`Invoke-Expression -Command $({cmd_name} completion powershell --rename {cmd_name} | Out-String)`
"##
    )
}

fn completion_help_zsh(cmd_name: &str) -> String {
    format!(
        r##"Generate a `_{cmd_name}` completion script and put it somewhere in your `$fpath`:
`{cmd_name} completion zsh --rename {cmd_name} > /usr/local/share/zsh/site-functions/_{cmd_name}`

Ensure that the following is present in your `~/.zshrc`:

`autoload -U compinit`
`compinit -i`
"##
    )
}

fn completion_help_unknown(_cmd_name: &str) -> String {
    r##"The completion script generation is not implemented for this shell. Head over to
https://github.com/cnpryer/huak/issues to file an issue to add support.
"##.to_string()
}
