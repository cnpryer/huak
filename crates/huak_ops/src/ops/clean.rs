use crate::{Config, HuakResult};

pub struct CleanOptions {
    pub include_pycache: bool,
    pub include_compiled_bytecode: bool,
}

pub fn clean_project(
    config: &Config,
    options: &CleanOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();

    // Remove everything from the dist directory if it exists.
    if workspace.root().join("dist").exists() {
        std::fs::read_dir(workspace.root().join("dist"))?
            .filter_map(|x| x.ok().map(|item| item.path()))
            .for_each(|item| {
                if item.is_dir() {
                    std::fs::remove_dir_all(item).ok();
                } else if item.is_file() {
                    std::fs::remove_file(item).ok();
                }
            });
    }

    // Remove all __pycache__ directories in the workspace if they exist.
    if options.include_pycache {
        let pattern = format!(
            "{}",
            workspace.root().join("**").join("__pycache__").display()
        );
        glob::glob(&pattern)?.for_each(|item| {
            if let Ok(it) = item {
                std::fs::remove_dir_all(it).ok();
            }
        })
    }

    // Remove all .pyc files in the workspace if they exist.
    if options.include_compiled_bytecode {
        let pattern =
            format!("{}", workspace.root().join("**").join("*.pyc").display());
        glob::glob(&pattern)?.for_each(|item| {
            if let Ok(it) = item {
                std::fs::remove_file(it).ok();
            }
        })
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fs::{self, CopyDirOptions},
        ops::test_config,
        test_resources_dir_path, Verbosity,
    };
    use tempfile::tempdir;

    #[test]
    fn test_clean_project() {
        let dir = tempdir().unwrap();
        fs::copy_dir(
            test_resources_dir_path().join("mock-project"),
            dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let root = dir.path().join("mock-project");
        let cwd = root.to_path_buf();
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = CleanOptions {
            include_pycache: true,
            include_compiled_bytecode: true,
        };

        clean_project(&config, &options).unwrap();

        let dist = glob::glob(&format!(
            "{}",
            config.workspace_root.join("dist").join("*").display()
        ))
        .unwrap()
        .map(|item| item.unwrap())
        .collect::<Vec<_>>();
        let pycaches = glob::glob(&format!(
            "{}",
            config
                .workspace_root
                .join("**")
                .join("__pycache__")
                .display()
        ))
        .unwrap()
        .map(|item| item.unwrap())
        .collect::<Vec<_>>();
        let bytecode = glob::glob(&format!(
            "{}",
            config.workspace_root.join("**").join("*.pyc").display()
        ))
        .unwrap()
        .map(|item| item.unwrap())
        .collect::<Vec<_>>();

        assert!(dist.is_empty());
        assert!(pycaches.is_empty());
        assert!(bytecode.is_empty());
    }
}
