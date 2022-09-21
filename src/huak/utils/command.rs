use std::{path::Path, process};

use crate::errors::CliResult;

/// Run a command using process::Command and an array of args. The command will
/// execute inside a `from` dir.
pub(crate) fn run_command(cmd: &str, args: &[&str], from: &Path) -> CliResult {
    let output = process::Command::new(cmd)
        .args(args)
        .current_dir(from)
        .output()?;

    let stdout = buff_to_string(&output.stdout)?;
    let stderr = buff_to_string(&output.stdout)?;

    println!("{}", &stdout);
    eprintln!("{}", &stderr);

    // TODO: DEBUG-level logging.
    // if !output.status.success() {
    //     eprintln!("failed to run command '{}' with {:?}", cmd, args);
    // }

    Ok(())
}

fn buff_to_string(buff: &[u8]) -> Result<String, anyhow::Error> {
    let stdout = match std::str::from_utf8(buff) {
        Ok(v) => v,
        Err(e) => return Err(anyhow::format_err!(e)),
    };

    Ok(stdout.to_string())
}
