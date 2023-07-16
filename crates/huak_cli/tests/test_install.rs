#[cfg(test)]
mod tests {
    use insta_cmd::assert_cmd_snapshot;
    use std::process::Command;

    #[test]
    fn test_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("install").arg("--help"));
    }
}
