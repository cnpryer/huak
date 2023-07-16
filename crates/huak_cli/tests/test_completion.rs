
#[cfg(test)]
mod tests {
    use std::process::Command;
    use insta_cmd::assert_cmd_snapshot;

    #[test]
    fn test_help() {
        assert_cmd_snapshot!(Command::new("huak").arg("completion").arg("--help"));
    }
}
