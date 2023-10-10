use std::path::PathBuf;

use crate::{error::HuakResult, Error};
use git2::Repository;

/// From <https://github.com/github/gitignore/blob/main/Python.gitignore>.
const DEFAULT_PYTHON_GITIGNORE: &str = r"
__pycache__/
*.py[cod]
*$py.class
*.so
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
share/python-wheels/
*.egg-info/
.installed.cfg
*.egg
MANIFEST
*.manifest
*.spec
pip-log.txt
pip-delete-this-directory.txt
htmlcov/
.tox/
.nox/
.coverage
.coverage.*
.cache
nosetests.xml
coverage.xml
*.cover
*.py,cover
.hypothesis/
.pytest_cache/
cover/
*.mo
*.pot
*.log
local_settings.py
db.sqlite3
db.sqlite3-journal
instance/
.webassets-cache
.scrapy
docs/_build/
.pybuilder/
target/
.ipynb_checkpoints
profile_default/
ipython_config.py
__pypackages__/
celerybeat-schedule
celerybeat.pid
*.sage.py
.env
.venv
env/
venv/
ENV/
env.bak/
venv.bak/
.spyderproject
.spyproject
.ropeproject
/site
.mypy_cache/
.dmypy.json
dmypy.json
.pyre/
.pytype/
cython_debug/
";

/// Initialize a directory on a local system as a git repository
/// and return the Repository.
pub fn init<T: Into<PathBuf>>(path: T) -> HuakResult<Repository> {
    Repository::init(path.into()).map_err(Error::GitError)
}

#[must_use]
pub fn default_python_gitignore() -> &'static str {
    DEFAULT_PYTHON_GITIGNORE
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init() {
        let dir = tempdir().unwrap();
        init(dir.path()).unwrap();
        assert!(dir.path().join(".git").is_dir());
    }
}
