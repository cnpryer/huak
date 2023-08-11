use std::process::Command;

use huak_ops::{Config, HuakResult};

pub fn activate_python_environment(config: &Config) -> HuakResult<()> {
    let workspace = config.workspace();
    let python_env = workspace.current_python_environment()?;

    if python_env.active() {
        return Ok(());
    }

    #[cfg(unix)]
    let mut cmd = Command::new("bash");
    #[cfg(unix)]
    cmd.args([
        "--init-file",
        &format!(
            "{}",
            python_env.executables_dir_path().join("activate").display()
        ),
        "-i",
    ]);
    #[cfg(windows)]
    let mut cmd = Command::new("powershell");
    #[cfg(windows)]
    cmd.args([
        "-executionpolicy",
        "bypass",
        "-NoExit",
        "-NoLogo",
        "-File",
        &format!(
            "{}",
            python_env
                .executables_dir_path()
                .join("activate.ps1")
                .display()
        ),
    ]);

    config.terminal().run_command(&mut cmd)
}
