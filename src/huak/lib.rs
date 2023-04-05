///! # Huak
///!
///! A Python package manager writen in Rust inspired by Cargo.
///!
///! ## About
///!
///! Huak is considered a package manager but focuses on supporting development workflows
///! useful for building both Python packages and projects in general.
///!
///! Workflows supported consist of the following life-cycle:
///! 1. Initialization and setup
///! 2. Making some change to the project
///! 3. Running tests
///! 4. Distributing the project
///!
///!❯ huak help
///!
///!A Python package manager written in Rust inspired by Cargo.
///!
///!Usage: huak [OPTIONS] <COMMAND>
///!
///!Commands:
///!  activate    Activate the virtual envionrment
///!  add         Add dependencies to the project
///!  build       Build tarball and wheel for the project
///!  completion  Generates a shell completion script for supported shells
///!  clean       Remove tarball and wheel from the built project
///!  fix         Auto-fix fixable lint conflicts
///!  fmt         Format the project's Python code
///!  init        Initialize the existing project
///!  install     Install the dependencies of an existing project
///!  lint        Lint the project's Python code
///!  new         Create a new project at <path>
///!  lish     Builds and uploads current project to a registry
///!  python      Manage Python installations
///!  remove      Remove dependencies from the project
///!  run         Run a command within the project's environment context
///!  test        Test the project's Python code
///!  update      Update the project's dependencies
///!  version     Display the version of the project
///!  help        Print this message or the help of the given subcommand(s)
///!
///! Options:
///!   -q, --quiet    
///!   -h, --help     Print help
///!   -V, --version  Print version
///
use error::{Error, HuakResult};
use indexmap::IndexMap;
use pep440_rs::{Operator, Version as PEP440Version, VersionSpecifiers};
use pep508_rs::Requirement;
use pyproject_toml::{Project, PyProjectToml as ProjectToml};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::hash_map::RandomState,
    collections::HashMap,
    env::consts::OS,
    ffi::OsStr,
    ffi::OsString,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};
use sys::{Terminal, TerminalOptions};
use toml::Table;
mod error;
mod fs;
mod git;
pub mod ops;
mod sys;

const DEFAULT_VENV_NAME: &str = ".venv";
const VENV_CONFIG_FILE_NAME: &str = "pyvenv.cfg";
const VERSION_OPERATOR_CHARACTERS: [char; 5] = ['=', '~', '!', '>', '<'];
const VIRTUAL_ENV_ENV_VAR: &str = "VIRUTAL_ENV";
const CONDA_ENV_ENV_VAR: &str = "CONDA_PREFIX";
const DEFAULT_PROJECT_VERSION_STR: &str = "0.0.1";
const DEFAULT_METADATA_FILE_NAME: &str = "pyproject.toml";

/// The main `Config` for Huak.
///
/// The `Config` contains data telling Huak what to do during certain operations.
/// An example would be indicating what the initial `Workspace` root should be or
/// what the current working directory was at the time the operation was requested.
///
/// ```
/// use huak::Config;
///
/// let config = Config {
///     workspace_root: PathBuf::from("."),
///     cwd: PathBuf::from("."),
/// }
///
/// let workspace = config.workspace();
/// ```
struct Config {
    /// The configured `Workspace` root path.
    workspace_root: PathBuf,
    /// The current working directory where Huak was invoked or otherwise requested from.
    cwd: PathBuf,
    /// `Terminal` options to use.
    terminal_options: TerminalOptions,
}

impl Config {
    /// Resolve the current workspace based on the `Config` data.
    fn workspace(&self) -> HuakResult<Workspace> {
        Workspace::new(&self.workspace_root)
    }
    /// Get a `Terminal` based on the `Config` data.
    fn terminal(&self) -> Terminal {
        let mut terminal = Terminal::new();
        terminal.set_verbosity(self.terminal_options.verbosity().clone());

        terminal
    }
}

/// The `Workspace` is a useful struct for reolving things like the current `Package`
/// or the `Environment` itself.
///
/// ```
/// use huak::Workspace;
///
/// let workspace = Workspace::new(".");
///
/// // Get the `Workspace`'s `Environment`.
/// let env = workspace.environment();
/// ```
struct Workspace {
    /// The absolute path to the `Workspace` root.
    root: PathBuf,
    /// The `Environment associated with the `Workspace`.
    env: Environment,
    /// Huak's `Config`.
    config: Config,
}

impl Workspace {
    fn new<T: AsRef<Path>>(path: T) -> HuakResult<Self> {
        todo!()
    }

    /// Get the current `Environment` for the `Workspace`.
    fn environmet(&self) -> Environment {
        Environment::new()
    }

    /// Resolve a `Package` pulling the current or creating one if none is found.
    fn resolve_package(&self) -> Package {
        todo!()
    }

    /// Get the current `Package`. The current `Package` is one found by its
    /// metadata file nearest baseed on `Config` data.
    fn current_package(&self) -> HuakResult<Package> {
        todo!()
    }

    /// Get the `Package`s for the `Workspace`.
    fn packages(&self) -> Vec<Package> {
        todo!()
    }

    /// Get the current `LocalMetadata` based on the `Config` data.
    fn current_local_metadata(&self) -> HuakResult<LocalMetdata> {
        todo!()
    }

    /// Resolve a `PythonEnvironment` pulling the current or creating one if none is found.
    fn resolve_python_environment(&self) -> PythonEnvironment {
        todo!()
    }

    /// Get the current `PythonEnvironment`. The current `PythonEnvironment` is one
    /// found by its configuration file or `Interpreter` nearest baseed on `Config` data.
    fn current_python_environment(&self) -> HuakResult<PythonEnvironment> {
        todo!()
    }

    /// Get the `PythonEnvironment`s for the `Workspace`.
    fn python_environments(&self) -> Vec<PythonEnvironment> {
        todo!()
    }

    /// Create a `PythonEnvironment` for the `Workspace`.
    fn new_python_environment(&self) -> HuakResult<PythonEnvironment> {
        let python_path = match self.env.python_paths().next() {
            Some(it) => it,
            None => return Err(Error::PythonNotFound),
        };

        let name = DEFAULT_VENV_NAME;
        let path = self.root.join(name);

        let args = ["-m", "venv", name];
        let mut cmd = Command::new(python_path);
        cmd.args(args).current_dir(&self.root);

        self.config.terminal().run_command(&mut cmd)?;

        Ok(PythonEnvironment::new(path)?)
    }
}

struct WorkspaceOptions {
    uses_git: bool,
}

/// Search for a Python virtual environment.
/// 1. If VIRTUAL_ENV exists then a venv is active; use it.
/// 2. Walk from configured cwd up searching for dir containing the Python environment config file.
/// 3. Stop after searching `stop_after`.
pub fn find_venv_root<T: AsRef<Path>>(
    from: T,
    stop_after: T,
) -> HuakResult<PathBuf> {
    if let Ok(path) = std::env::var("VIRTUAL_ENV") {
        return Ok(PathBuf::from(path));
    }

    let file_path = match fs::find_root_file_bottom_up(
        VENV_CONFIG_FILE_NAME,
        from,
        stop_after,
    ) {
        Ok(it) => it.ok_or(Error::PythonEnvironmentNotFound)?,
        Err(_) => return Err(Error::PythonEnvironmentNotFound),
    };

    let root = file_path.parent().ok_or(Error::InternalError(
        "failed to establish parent directory".to_string(),
    ))?;

    Ok(root.to_path_buf())
}

/// The `Environment` manages interacting with the system, installed Python
/// interpreters, and more useful features Huak utilizes.
///
/// `Environment`s would be used for resolving environment variables, the
/// the paths to Python interpreters, executing commands and relaying information
/// to a terminal for the user, and much more.
///
/// ```
/// use huak::Environment;
///
/// let env = Environment::new();
/// let python_path = env.python_paths().max()
/// ```
struct Environment {
    /// Python `Interpreter`s installed on the system.
    interpreters: Interpreters,
    /// `PythonEnvironment`s found on the system.
    python_envs: PythonEnvironments,
    /// Original environment variables.
    env: Env,
}

impl Environment {
    /// Initialize an `Environment`.
    fn new() -> Environment {
        todo!()
    }

    /// Get an `Iterator` over the Python `Interpreter` `PathBuf`s found.
    fn python_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.interpreters
            .interpreters
            .iter()
            .map(|interpreter| &interpreter.path)
    }

    /// Resolve `Interpreters` for the `Environment`.
    fn resolve_interpreters(&self) -> Interpreters {
        todo!()
    }
}

struct PythonEnvironments {
    envs: Vec<PythonEnvironment>,
}

impl PythonEnvironments {
    fn get<T: AsRef<Path>>(&self, k: T) -> Option<PythonEnvironment> {
        todo!()
    }

    fn insert(&mut self, env: PythonEnvironment) {
        todo!()
    }
}

/// The `PythonEnvironment` is a struct used to intereact with an environment
/// containing an installed Python `Interpreter` and `Package`s.
///
/// An example of a valid `PythonEnvironment` would be a Virtual environment.
/// See https://peps.python.org/pep-0405/
/// The structure of a `Venv` on a system depends on if it is Windows or not.
///
/// For Windows:
/// .venv (typically .venv or venv)
/// ├── Scripts
/// │   ├── python.exe
/// │   └── pip.exe
/// │── Lib
/// │   └── site-packages
/// └── pyvenv.cfg
///
/// Otherwise:
/// .venv (typically .venv or venv)
/// ├── bin
/// │   ├── python
/// │   └── pip
/// │── lib
/// │   └── python3.11
/// │      └── site-packages
/// └── pyvenv.cfg
///
/// Note that on Windows site-packages is under Lib but elsewhere it's under
/// lib/python{version-major.version-minor}. `pyvenv.cfg` is the `PythonEnvironment`'s
/// config file and contains information like the version of the Python
/// interpreter, *potentially* the "home" path to the Python interpreter that
/// created the `PythonEnvironment`, etc.
///
/// ```
/// use huak::PythonEnvironment;
///
/// let venv = PythonEnvironment::from(".venv");
/// ```
struct PythonEnvironment {
    /// The absolute path to the `PythonEnvironment`'s root.
    root: PathBuf,
    /// The `PythonEnvironment`'s Python `Interpreter`.
    interpreter: Interpreter,
    /// The abolute path to the `PythonEnvironment`'s executables directory. This directory contains
    /// installed Python modules and the `Interpreter` the `Venv` uses. On Windows this
    /// is located at `PythonEnvironment.root\Scripts\`, otherwise it's located at
    /// `PythonEnvironment.root/bin/`
    executables_dir_path: PathBuf,
    /// The site-packages directory contains all of the `PythonEnvironment`'s installed Python packages.
    site_packages_path: PathBuf,
    /// The `PythonEnvironment`'s installed `Package`s.
    packages: Vec<Package>,
}

impl PythonEnvironment {
    /// Initialize a new `PythonEnvironment`.
    pub fn new<T: AsRef<Path>>(path: T) -> HuakResult<Self> {
        if !path.as_ref().join(VENV_CONFIG_FILE_NAME).exists() {
            return Err(Error::Unimplemented(format!(
                "{} is not supported",
                path.as_ref().display()
            )));
        }

        let env = new_venv(path)?;

        Ok(env)
    }

    /// Get a reference to the absolute path to the python environment.
    pub fn root(&self) -> &Path {
        self.root.as_ref()
    }

    /// Get the name of the Python environment.
    pub fn name(&self) -> HuakResult<String> {
        fs::last_path_component(&self.root)
    }

    /// The absolute path to the Python environment's Python interpreter binary.
    pub fn python_path(&self) -> &PathBuf {
        &self.interpreter.path
    }

    #[allow(dead_code)]
    /// Get the version of the Python environment's Python interpreter.
    pub fn python_version(&self) -> &Version {
        &self.interpreter.version
    }

    /// The absolute path to the Python environment's executables directory.
    pub fn executables_dir_path(&self) -> &PathBuf {
        &self.executables_dir_path
    }

    /// The absolute path to the Python environment's site-packages directory.
    pub fn site_packages_dir_path(&self) -> &PathBuf {
        &self.site_packages_path
    }

    /// Install Python packages to the environment.
    pub fn install_packages(
        &self,
        packages: PackageIter,
        options: &InstallOptions,
        config: &Config,
    ) -> HuakResult<()> {
        let mut cmd = Command::new(self.python_path());
        cmd.args(["-m", "pip", "install"])
            .args(packages.map(|item| item.to_dep_str()));

        if let Some(v) = options.values.as_ref() {
            cmd.args(v.iter().map(|item| item.as_str()));
        }

        config.terminal().run_command(&mut cmd)
    }

    /// Uninstall many Python packages from the environment.
    pub fn uninstall_packages(
        &self,
        packages: PackageIter,
        options: &InstallOptions,
        config: &Config,
    ) -> HuakResult<()> {
        let mut cmd = Command::new(self.python_path());
        cmd.args(["-m", "pip", "uninstall"])
            .args(packages.map(|item| item.to_dep_str()))
            .arg("-y");

        if let Some(v) = options.values.as_ref() {
            cmd.args(v.iter().map(|item| item.as_str()));
        }

        config.terminal().run_command(&mut cmd)
    }

    /// Update many Python packages in the environment.
    pub fn update_packages<T>(
        &self,
        packages: PackageIter,
        options: &InstallOptions,
        config: &Config,
    ) -> HuakResult<()>
    where
        T: Display + AsRef<OsStr>,
    {
        let mut cmd = Command::new(self.python_path());
        cmd.args(["-m", "pip", "install", "--upgrade"])
            .args(packages.map(|item| item.to_dep_str()))
            .arg("-y");

        if let Some(v) = options.values.as_ref() {
            cmd.args(v.iter().map(|item| item.as_str()));
        }

        config.terminal().run_command(&mut cmd)
    }

    /// Check if the environment is already activated.
    pub fn is_active(&self) -> bool {
        if let Some(path) = active_virtual_env_path() {
            return self.root == path;
        }
        if let Some(path) = active_conda_env_path() {
            return self.root == path;
        }
        false
    }

    /// Check if the environment has a module installed to the executables directory.
    pub fn contains_module(&self, module_name: &str) -> HuakResult<bool> {
        let dir = self.executables_dir_path();
        #[cfg(unix)]
        return Ok(dir.join(module_name).exists());
        #[cfg(windows)]
        {
            let mut path = dir.join(module_name);
            match path.set_extension("exe") {
                true => return Ok(path.exists()),
                false => Err(Error::InternalError(format!(
                    "failed to create path for {module_name}"
                ))),
            }
        }
    }

    #[allow(dead_code)]
    /// Check if the environment has a package already installed.
    pub fn contains_package(&self, package: &Package) -> bool {
        self.site_packages_dir_path()
            .join(
                package
                    .importable_name()
                    .unwrap_or(package.name().to_string()),
            )
            .exists()
    }

    /// Get all of the packages installed to the environment.
    fn installed_packages(&self) -> HuakResult<Vec<Package>> {
        let mut cmd = Command::new(self.python_path());
        cmd.args(["-m", "pip", "freeze"]);

        let output = cmd.output()?;
        let output = sys::parse_command_output(output)?;
        let mut packages = Vec::new();
        for line in output.split('\n') {
            if !line.is_empty() {
                packages.push(Package::from_str(line)?);
            }
        }

        Ok(packages)
    }

    /// Check if the environment is already activated.
    fn active(&self) -> bool {
        if let Some(path) = active_virtual_env_path() {
            return self.root == path;
        }
        if let Some(path) = active_conda_env_path() {
            return self.root == path;
        }
        false
    }
}

fn new_venv<T: AsRef<Path>>(path: T) -> HuakResult<PythonEnvironment> {
    let root = path.as_ref();

    let config = VenvConfig::new(&root.join(VENV_CONFIG_FILE_NAME))?;
    let python_version = config.version;

    // Establishing paths differs between Windows and Unix systems.
    #[cfg(unix)]
    let executables_dir_path = root.join("bin");
    #[cfg(unix)]
    let python_path = executables_dir_path.join("python");
    #[cfg(windows)]
    let executables_dir_path = root.join("Scripts");
    #[cfg(windows)]
    let python_path = executables_dir_path.join("python.exe");

    let version = config.version;
    let interpreter = Interpreter {
        version,
        path: python_path.to_path_buf(),
    };

    // On Unix systems the Venv's site-package directory depends on the Python version.
    // The path is root/lib/pythonX.X/site-packages.
    #[cfg(unix)]
    let site_packages_path = root
        .join("lib")
        .join(format!(
            "python{}.{}",
            version.release[0], version.release[1]
        ))
        .join("site-packages");
    #[cfg(windows)]
    let site_packages_path = root.join("Lib").join("site-packages");

    let venv = PythonEnvironment {
        root: root.to_path_buf(),
        interpreter,
        executables_dir_path,
        site_packages_path,
        packages: pip_freeze(&interpreter)?,
    };

    Ok(venv)
}

/// Execute and parse a `pip freeze` command with an `Interpreter`.
fn pip_freeze(interpreter: &Interpreter) -> HuakResult<Vec<Package>> {
    let mut cmd = Command::new(interpreter.path());
    cmd.args(["-m", "pip", "freeze"]);

    let output = cmd.output()?;
    let output = sys::parse_command_output(output)?;
    let mut packages = Vec::new();
    for line in output.split('\n') {
        if !line.is_empty() {
            packages.push(Package::from_str(line)?);
        }
    }

    Ok(packages)
}

#[derive(Clone)]
struct InstallOptions {
    values: Option<Vec<String>>,
}

// Data about some environment's Python configuration. This abstraction is modeled after
/// the pyenv.cfg file used for Python virtual environments.
struct VenvConfig {
    /// The version of the environment's Python interpreter.
    version: Version,
}

impl VenvConfig {
    fn new<T: AsRef<Path>>(value: T) -> HuakResult<Self> {
        // Read the file and flatten the lines for parsing.
        let file = File::open(value).unwrap_or_else(|_| {
            panic!("failed to open {}", value.as_ref().display())
        });
        let buff_reader = BufReader::new(file);
        let lines: Vec<String> = buff_reader.lines().flatten().collect();

        // Search for version = "X.X.X"
        let mut version = Version::from_str("0.0.0");
        lines.iter().for_each(|item| {
            let mut split = item.splitn(2, '=');
            let key = split.next().unwrap_or_default().trim();
            let val = split.next().unwrap_or_default().trim();
            if key == "version" {
                version = Version::from_str(val)
            }
        });

        let version = version.expect("Python version from pyvenv.cfg");
        let cfg = VenvConfig { version };

        Ok(cfg)
    }
}

struct Interpreters {
    interpreters: Vec<Interpreter>,
}

impl Interpreters {
    fn latest(&self) -> Option<Interpreter> {
        todo!()
    }

    fn exact(&self, version: &Version) -> Option<Interpreter> {
        todo!()
    }

    fn get<T: AsRef<Path>>(&self, path: T) -> Option<Interpreter> {
        todo!()
    }
}

trait ToPkgId {
    fn to_pkg(self) -> PackageId;
}

trait ToDepStr {
    fn to_dep_str(&self) -> &str;
}

/// The `Package` manages interacting with Python packages or Python projects.
///
/// `Package` contains information like the project's name, its version, authors
/// dependencies (other `Package`s), and more. This data is stored in its
/// `Metadata`.
///
/// ```
/// use huak::Package;
/// use pep440_rs::Version;
///
/// let mut package = Package::new("my-project");
/// package.set_version(Version::from_str("0.0.1").unwrap()));
///
/// assert_eq!(package.version, Version::from_str("0.0.1").unwrap()));
/// ```
struct Package {
    /// Information used to identify the `Package`.
    id: PackageId,
    /// The `Package`'s core `Metadata`.
    metadata: Metadata,
}

impl Package {
    fn name(&self) -> &str {
        &self.id.name
    }

    fn importable_name(&self) -> HuakResult<String> {
        importable_package_name(&self.id.name)
    }

    fn version(&self) -> &PEP440Version {
        &self.id.version
    }
}

impl ToDepStr for Package {
    fn to_dep_str(&self) -> &str {
        format!("{}=={}", self.name(), self.version()).as_str()
    }
}

struct PackageIter<'a> {
    iter: std::slice::Iter<'a, Package>,
}

impl<'a> Iterator for PackageIter<'a> {
    type Item = &'a Package;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl FromStr for Package {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let spec_str = parse_version_specifiers_str(s)?;
        let name = s.strip_suffix(spec_str).unwrap_or(s).to_string();
        let version_specifiers = VersionSpecifiers::from_str(spec_str)?;

        if version_specifiers.len() > 1 {
            return Err(Error::InvalidVersionString(format!(
                "{} can only contain one version specifier",
                s
            )));
        }

        let version_specifer = version_specifiers.first().unwrap();

        if version_specifer.operator() != &Operator::Equal {
            return Err(Error::InvalidVersionString(format!(
                "{} must contain {} specifier",
                s,
                Operator::Equal
            )));
        }

        let id = PackageId {
            name: canonical_package_name(&name)?,
            version: version_specifer.version().to_owned(),
        };

        let project = Project {
            name,
            version: None,
            description: None,
            readme: None,
            requires_python: None,
            license: None,
            authors: None,
            maintainers: None,
            keywords: None,
            classifiers: None,
            urls: None,
            entry_points: None,
            scripts: None,
            gui_scripts: None,
            dependencies: None,
            optional_dependencies: None,
            dynamic: None,
            license_expression: None,
            license_files: None,
        };

        let metadata = Metadata {
            project,
            tool: None,
        };

        let package = Package { id, metadata };

        Ok(package)
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Package {}

struct PackageId {
    name: String,
    version: PEP440Version,
}

struct LocalMetdata {
    metadata: Metadata, // TODO: https://github.com/cnpryer/huak/issues/574
    path: PathBuf,
}

impl LocalMetdata {
    /// Initialize `LocalMetdata` from a `Path`.
    fn new<T: AsRef<Path>>(path: T) -> HuakResult<LocalMetdata> {
        if path.as_ref().file_name()
            != Some(OsStr::new(DEFAULT_METADATA_FILE_NAME))
        {
            return Err(Error::Unimplemented(format!(
                "{} is not supported",
                path.as_ref().display()
            )));
        }

        let local_metadata = local_metadata(path)?;

        Ok(local_metadata)
    }

    pub fn write_file(&self) -> HuakResult<()> {
        let string = self.to_string_pretty()?;
        Ok(std::fs::write(self.path, string)?)
    }

    pub fn to_string_pretty(&self) -> HuakResult<String> {
        Ok(toml_edit::ser::to_string_pretty(&self.metadata)?)
    }
}

impl Display for LocalMetdata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.metadata)
    }
}

fn local_metadata<T: AsRef<Path>>(path: T) -> HuakResult<LocalMetdata> {
    let pyproject_toml = PyProjectToml::new(path.as_ref())?;
    let project = match pyproject_toml.project.as_ref() {
        Some(it) => it,
        None => {
            return Err(Error::InternalError(format!(
                "{} is missing a project table",
                path.as_ref().display()
            )))
        }
    }
    .to_owned();
    let tool = pyproject_toml.tool.to_owned();

    let metadata = Metadata { project, tool };

    let local_metadata = LocalMetdata {
        metadata,
        path: path.as_ref().to_path_buf(),
    };

    Ok(local_metadata)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
/// The core `Metadata` of a `Package`.
///
/// See https://peps.python.org/pep-0621/.
struct Metadata {
    project: Project,
    tool: Option<Table>,
}

impl Metadata {
    pub fn project_name(&self) -> &str {
        self.project.name.as_str()
    }

    pub fn set_project_name(&mut self, name: String) {
        self.project.name = name
    }

    pub fn project_version(&self) -> Option<PEP440Version> {
        self.project.version
    }

    pub fn dependencies(&self) -> Option<&[Requirement]> {
        self.project.dependencies.as_deref()
    }

    pub fn contains_dependency(
        &self,
        dependency: &Dependency,
    ) -> HuakResult<bool> {
        todo!()
    }

    pub fn contains_dependency_any(
        &self,
        dependency: &Dependency,
    ) -> HuakResult<bool> {
        todo!()
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.project
            .dependencies
            .as_mut()
            .map(|deps| deps.push(dependency.requirement));
    }

    pub fn optional_dependencies(
        &self,
    ) -> Option<&IndexMap<String, Vec<Requirement>>> {
        self.project.optional_dependencies.as_ref()
    }

    pub fn contains_optional_dependency(
        &self,
        dependency: &Dependency,
        group: &str,
    ) -> HuakResult<bool> {
        todo!()
    }

    pub fn optional_dependencey_group(
        &self,
        group: &str,
    ) -> Option<&Vec<Requirement>> {
        self.project
            .optional_dependencies
            .as_ref()
            .and_then(|deps| deps.get(group))
    }

    pub fn add_optional_dependency(
        &mut self,
        dependency: Dependency,
        group: &str,
    ) {
        self.project
            .optional_dependencies
            .as_mut()
            .get_or_insert(&mut IndexMap::new())
            .entry(group.to_string())
            .or_insert_with(Vec::new)
            .push(dependency.requirement);
    }

    pub fn remove_dependency(&mut self, dependency: &Dependency) {
        self.project
            .dependencies
            .as_mut()
            .filter(|deps| deps.contains(&dependency.requirement))
            .map(|deps| {
                let i = deps
                    .iter()
                    .position(|dep| *dep == dependency.requirement)
                    .unwrap();
                deps.remove(i);
            });
    }

    pub fn remove_optional_dependency(
        &mut self,
        dependency: &Dependency,
        group: &str,
    ) {
        self.project
            .optional_dependencies
            .as_mut()
            .and_then(|g| g.get_mut(group))
            .and_then(|deps| {
                deps.iter()
                    .position(|dep| *dep == dependency.requirement)
                    .map(|i| deps.remove(i))
            });
    }

    pub fn scripts(&self) -> Option<&IndexMap<String, String, RandomState>> {
        self.project.scripts.as_ref()
    }

    pub fn add_script(&mut self, name: &str, entrypoint: &str) {
        self.project
            .scripts
            .get_or_insert(IndexMap::new())
            .entry(name.to_string())
            .or_insert(entrypoint.to_string());
    }
}

fn parse_toml_depenencies(project: &Project) -> Option<Vec<Dependency>> {
    project.dependencies.as_ref().map(|items| {
        items
            .iter()
            .map(|item| {
                Dependency::from_str(&item.to_string())
                    .expect("toml dependencies")
            })
            .collect::<Vec<Dependency>>()
    })
}

fn parse_toml_optional_dependencies(
    project: &Project,
) -> Option<IndexMap<String, Vec<Dependency>>> {
    project.optional_dependencies.as_ref().map(|groups| {
        IndexMap::from_iter(groups.iter().map(|(group, deps)| {
            (
                group.clone(),
                deps.iter()
                    .map(|dep| {
                        Dependency::from_str(&dep.to_string())
                            .expect("toml optional dependencies")
                    })
                    .collect(),
            )
        }))
    })
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.project == other.project && self.tool == other.tool
    }
}

impl Eq for Metadata {}

/// A pyproject.toml as specified in PEP 517
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PyProjectToml {
    #[serde(flatten)]
    inner: ProjectToml,
    tool: Option<Table>,
}

impl std::ops::Deref for PyProjectToml {
    type Target = ProjectToml;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for PyProjectToml {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl PyProjectToml {
    /// Initialize a `PyProjectToml` from its path.
    pub fn new<T: AsRef<Path>>(path: T) -> HuakResult<PyProjectToml> {
        let contents = std::fs::read_to_string(path)?;
        let pyproject_toml: PyProjectToml = toml::from_str(&contents)?;

        Ok(pyproject_toml)
    }
}

impl Default for PyProjectToml {
    fn default() -> Self {
        Self {
            inner: ProjectToml::new(&default_pyproject_toml_contents())
                .expect("valid pyproject.toml contents"),
            tool: None,
        }
    }
}

fn default_pyproject_toml_contents() -> &'static str {
    r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"
[project]
name = "{project_name}"
version = "0.0.1"
description = ""
dependencies = []
"#
}

/// The `Dependency` is an abstraction for `Package` data used as a cheap alternative
/// for operations on lots of `Package` information.
///
/// `Dependency`s can contain different information about a `Package` necessary to
/// use them as `Package` `Dependency`s, such as having multiple `VersionSpecifiers`
/// or boolean flags indicating they're optional `Dependency`s.
///
/// ```
/// use huak::Dependency;
///
/// let dependency = Dependency::from_str("my-dependency>=0.1.0,<0.2.0").unwrap();
/// ```
struct Dependency {
    /// PEP 508 dependency (called `Requirement` in pep508_rs).
    requirement: Requirement,
    /// The PEP440-compliant `VersionSpecifiers`. See https://peps.python.org/pep-0440/.
    version_specifiers: VersionSpecifiers,
}

impl Dependency {
    fn name(&self) -> &str {
        &self.requirement.name
    }

    fn version_specifiers(&self) -> &VersionSpecifiers {
        &self.version_specifiers
    }
}

impl ToPkgId for Dependency {
    fn to_pkg(mut self) -> PackageId {
        PackageId {
            name: self.requirement.name,
            version: *self.version_specifiers.first().unwrap().version(),
        } // TODO: Handle multiple version specs
    }
}

impl ToDepStr for Dependency {
    fn to_dep_str(&self) -> &str {
        let version_specifiers = self.version_specifiers.iter().map(|spec| {
            spec.to_string()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("")
        });

        format!(
            "{}{}",
            self.name(),
            version_specifiers.collect::<Vec<_>>().join(",")
        )
        .as_str()
    }
}

/// Initialize a `Dependency` from a `&str`.
///
/// ```
/// use huak::Dependency;
///
/// let dependency = Dependency::from_str("my-dependency>=0.1.0,<0.2.0").unwrap();
/// ```
impl FromStr for Dependency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let spec_str = parse_version_specifiers_str(s)?;
        let name = s.strip_suffix(spec_str).unwrap_or(s).to_string();
        let version_specifiers = VersionSpecifiers::from_str(spec_str)?;

        let dependency = Dependency {
            requirement: Requirement {
                name,
                extras: None,
                version_or_url: None,
                marker: None,
            },
            version_specifiers,
        };

        Ok(dependency)
    }
}

impl AsRef<OsStr> for Dependency {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self)
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.requirement == other.requirement
    }
}

impl Eq for Dependency {}

/// Construct an `Iterator` over a `IntoIterator` of `&str`s.
///
/// ```
/// let dependencies = vec!["my-dep", "my-dep==0.0.1"];
/// let iter = dependency_iter(dependencies);
/// ```
fn dependency_iter<I>(iter: I) -> impl Iterator<Item = Dependency>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    iter.into_iter()
        .filter_map(|item| Dependency::from_str(item.as_ref()).ok())
}

fn parse_version_specifiers_str(s: &str) -> HuakResult<&str> {
    let found = s
        .chars()
        .enumerate()
        .find(|x| VERSION_OPERATOR_CHARACTERS.contains(&x.1));

    let spec = match found {
        Some(it) => &s[it.0..],
        None => {
            return Err(Error::InvalidVersionString(format!(
                "{} is missing a version specifier",
                s
            )))
        }
    };

    Ok(spec)
}

fn importable_package_name(name: &str) -> HuakResult<String> {
    let canonical_name = canonical_package_name(name)?;
    Ok(canonical_name.replace('-', "_"))
}

fn canonical_package_name(name: &str) -> HuakResult<String> {
    let re = Regex::new("[-_. ]+")?;
    let res = re.replace_all(name, "-");
    Ok(res.into_owned())
}

/// The Python `Interpreter` is used to interact with installed Python `Interpreter`s.
///
/// `Interpreter` contains information like the `Interpreter`'s path, `Version`, etc.
///
/// ```
/// use huak::Interpreter;
///
/// let python = Interpreter::new("path/to/python");
/// ```
struct Interpreter {
    /// The `Version` of the Python `Interpreter`.
    version: Version,
    /// The absolute path to the Python `Interpreter`.
    path: PathBuf,
}

impl Interpreter {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn version(&self) -> &Version {
        &self.version
    }
}

impl PartialEq<Self> for Interpreter {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Interpreter {}

impl PartialOrd<Self> for Interpreter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Interpreter {
    fn cmp(&self, other: &Self) -> Ordering {
        match compare_interpreters(&self, &other) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

fn compare_interpreters(this: &Interpreter, other: &Interpreter) -> Ordering {
    if this.version != other.version {
        return this.version.cmp(&other.version);
    }

    Ordering::Equal
}

/// The `Env` struct is a lighter abstraction than `Environmnet` used to manage
/// environment variables.
///
/// ```
/// use huak::Env;
///
/// let env = Env::new();
/// let path = env.get("PATH");
/// let paths = env.paths();
/// ```
struct Env {
    env: HashMap<String, String>,
}

trait ToSemVer {
    fn to_semver(self) -> SemVer;
}

/// A generic `Version` struct implemnted Semantic-version-compliant `Version`s.
///
/// This struct is mainly used for the Python `Interpreter`.
struct Version {
    release: Vec<usize>,
}

struct SemVer {
    major: usize,
    minor: usize,
    patch: usize,
}

impl ToSemVer for Version {
    fn to_semver(self) -> SemVer {
        SemVer {
            major: self.release[0],
            minor: self.release[1],
            patch: self.release[2],
        }
    }
}

/// Initialize a `Version` from a `&str`.
///
/// ```
/// use huak::Version;
///
/// let a = Version::from_str("0.0.1").unwrap();
/// let b = Version::from_str("0.0.2").unwrap();
///
/// assert!(a < b);
/// ```
impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = captures_version_str(s)?;
        let release = parse_semver_from_captures(&captures)?;

        if release.len() != 3 {
            return Err(Error::InvalidVersionString(format!(
                "{} must be SemVer-compatiable",
                s
            )));
        }

        let version = Version { release };

        Ok(version)
    }
}

fn captures_version_str(s: &str) -> HuakResult<Captures> {
    let re = Regex::new(r"^(\d+)(?:\.(\d+))?(?:\.(\d+))?$")?;
    let captures = match re.captures(s) {
        Some(captures) => captures,
        None => return Err(Error::InvalidVersionString(s.to_string())),
    };
    Ok(captures)
}

fn parse_semver_from_captures(captures: &Captures) -> HuakResult<Vec<usize>> {
    let mut parts = vec![0, 0, 0];
    for i in [0, 1, 2].into_iter() {
        if let Some(it) = captures.get(i + 1) {
            parts[i] = it
                .as_str()
                .parse::<usize>()
                .map_err(|e| Error::InternalError(e.to_string()))?
        }
    }
    Ok(parts)
}

impl PartialEq<Self> for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

impl PartialOrd<Self> for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match compare_release(&self, &other) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

fn compare_release(this: &Version, other: &Version) -> Ordering {
    for (a, b) in [
        // TODO
        (this.release[0], other.release[0]),
        (this.release[1], this.release[1]),
        (this.release[2], other.release[2]),
    ] {
        if a != b {
            return a.cmp(&b);
        }
    }

    Ordering::Equal
}

/// Get an iterator over available Python interpreter paths parsed from PATH.
/// Inspired by brettcannon/python-launcher
pub fn python_paths() -> impl Iterator<Item = (Option<Version>, PathBuf)> {
    let paths =
        fs::flatten_directories(env_path_values().unwrap_or(Vec::new()));
    python_interpreters_in_paths(paths)
}

/// Get an iterator over all found python interpreter paths with their version.
fn python_interpreters_in_paths(
    paths: impl IntoIterator<Item = PathBuf>,
) -> impl Iterator<Item = (Option<Version>, PathBuf)> {
    paths.into_iter().filter_map(|item| {
        item.file_name()
            .or(None)
            .and_then(|raw_file_name| raw_file_name.to_str().or(None))
            .and_then(|file_name| {
                if valid_python_interpreter_file_name(file_name) {
                    #[cfg(unix)]
                    {
                        if let Ok(version) =
                            version_from_python_interpreter_file_name(file_name)
                        {
                            Some((Some(version), item.clone()))
                        } else {
                            None
                        }
                    }
                    #[cfg(windows)]
                    Some((
                        version_from_python_interpreter_file_name(file_name)
                            .ok(),
                        item.clone(),
                    ))
                } else {
                    None
                }
            })
    })
}

#[cfg(unix)]
fn valid_python_interpreter_file_name(file_name: &str) -> bool {
    file_name.len() >= "python3.0".len() && file_name.starts_with("python")
}

#[cfg(windows)]
fn valid_python_interpreter_file_name(file_name: &str) -> bool {
    file_name.starts_with("python") && file_name.ends_with(".exe")
}

fn version_from_python_interpreter_file_name(
    file_name: &str,
) -> HuakResult<Version> {
    match OS {
        "windows" => Version::from_str(
            &file_name.strip_suffix(".exe").unwrap_or(file_name)
                ["python".len()..],
        ),
        _ => Version::from_str(&file_name["python".len()..]),
    }
    .map_err(|_| {
        Error::InternalError(format!("could not version from {file_name}"))
    })
}

/// Get a vector of paths from the system PATH environment variable.
pub fn env_path_values() -> Option<Vec<PathBuf>> {
    if let Some(val) = env_path_string() {
        return Some(std::env::split_paths(&val).collect());
    }
    None
}

/// Get the OsString value of the enrionment variable PATH.
pub fn env_path_string() -> Option<OsString> {
    std::env::var_os("PATH")
}

/// Get the VIRTUAL_ENV environment path if it exists.
pub fn active_virtual_env_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var(VIRTUAL_ENV_ENV_VAR) {
        return Some(PathBuf::from(path));
    }
    None
}

/// Get the CONDA_PREFIX environment path if it exists.
pub fn active_conda_env_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var(CONDA_ENV_ENV_VAR) {
        return Some(PathBuf::from(path));
    }
    None
}

/// Get a `Version` from a Python interpreter using its path.
///
/// 1. Attempt to parse the version number from the path.
/// 2. Run `{path} --version` and parse from the output.
fn parse_python_interpreter_version<T: AsRef<Path>>(
    path: T,
) -> HuakResult<Option<Version>> {
    let version = match path
        .as_ref()
        .file_name()
        .and_then(|raw_file_name| raw_file_name.to_str())
    {
        Some(file_name) => {
            version_from_python_interpreter_file_name(file_name).ok()
        }
        None => {
            let mut cmd = Command::new(path.as_ref());
            cmd.args(["--version"]);
            let output = cmd.output()?;
            Version::from_str(&sys::parse_command_output(output)?).ok()
        }
    };
    Ok(version)
}

#[cfg(test)]
/// The resource directory found in the Huak repo used for testing purposes.
pub(crate) fn test_resources_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dev-resources")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;
    use tempfile::tempdir;

    #[test]
    fn toml_from_path() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metdata = LocalMetdata::new(path).unwrap();

        assert_eq!(local_metdata.metadata.project_name(), "mock_project");
        assert_eq!(
            local_metdata.metadata.project_version().unwrap(),
            PEP440Version::from_str("0.0.1").unwrap()
        );
        assert!(local_metdata.metadata.dependencies().is_some())
    }

    #[test]
    fn toml_to_string_pretty() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metdata = LocalMetdata::new(path).unwrap();

        assert_eq!(
            local_metdata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = ["click==8.1.3"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest>=6",
    "black==22.8.0",
    "isort==5.12.0",
]
"#
        );
    }

    #[test]
    fn toml_dependencies() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metdata = LocalMetdata::new(path).unwrap();

        assert_eq!(
            local_metdata.metadata.dependencies().unwrap().deref(),
            vec![Requirement::from_str("click==8.1.3").unwrap()]
        );
    }

    #[test]
    fn toml_optional_dependencies() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let local_metdata = LocalMetdata::new(path).unwrap();

        assert_eq!(
            local_metdata
                .metadata
                .optional_dependencey_group("dev")
                .unwrap()
                .deref(),
            vec![
                Requirement::from_str("pytest>=6").unwrap(),
                Requirement::from_str("black==22.8.0").unwrap(),
                Requirement::from_str("isort==5.12.0").unwrap()
            ]
        );
    }

    #[test]
    fn toml_add_dependency() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metdata = LocalMetdata::new(path).unwrap();
        let dep = Dependency {
            requirement: Requirement {
                name: "test".to_string(),
                extras: None,
                version_or_url: None,
                marker: None,
            },
            version_specifiers: VersionSpecifiers::from_str("==0.0.0").unwrap(),
        };
        local_metdata.metadata.add_dependency(dep);

        assert_eq!(
            local_metdata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = [
    "click==8.1.3",
    "test",
]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest>=6",
    "black==22.8.0",
    "isort==5.12.0",
]
"#
        )
    }

    #[test]
    fn toml_add_optional_dependency() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetdata::new(path).unwrap();

        local_metadata.metadata.add_optional_dependency(
            Dependency::from_str("test1").unwrap(),
            "dev",
        );
        local_metadata.metadata.add_optional_dependency(
            Dependency::from_str("test2").unwrap(),
            "new-group",
        );
        assert_eq!(
            local_metadata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = ["click==8.1.3"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest>=6",
    "black==22.8.0",
    "isort==5.12.0",
    "test1",
]
new-group = ["test2"]
"#
        )
    }

    #[test]
    fn toml_remove_dependency() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetdata::new(path).unwrap();

        local_metadata
            .metadata
            .remove_dependency(&Dependency::from_str("click").unwrap());
        assert_eq!(
            local_metadata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = []

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest>=6",
    "black==22.8.0",
    "isort==5.12.0",
]
"#
        )
    }

    #[test]
    fn toml_remove_optional_dependency() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let mut local_metadata = LocalMetdata::new(path).unwrap();

        local_metadata.metadata.remove_optional_dependency(
            &Dependency::from_str("isort").unwrap(),
            "dev",
        );
        assert_eq!(
            local_metadata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock_project"
version = "0.0.1"
description = ""
dependencies = ["click==8.1.3"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
dev = [
    "pytest>=6",
    "black==22.8.0",
]
"#
        )
    }

    #[test]
    fn python_environment_executable_dir_name() {
        let venv = PythonEnvironment::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();

        assert!(venv.executables_dir_path.exists());
        #[cfg(unix)]
        assert!(venv.executables_dir_path.join("python").exists());
        #[cfg(windows)]
        assert!(venv.executables_dir_path().join("python.exe").exists());
    }

    #[test]
    fn dependency_from_str() {
        let dep = Dependency::from_str("package-name==0.0.0").unwrap();

        assert_eq!(dep.to_dep_str(), "package-name==0.0.0");
        assert_eq!(dep.requirement.name, "package-name");
        assert_eq!(
            *dep.version_specifiers,
            vec![pep440_rs::VersionSpecifier::from_str("==0.0.0").unwrap()]
        );
    }

    #[test]
    fn find_python() {
        let path = python_paths().next().unwrap().1;

        assert!(path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn python_search() {
        let dir = tempdir().unwrap().into_path();
        std::fs::write(dir.join("python3.11"), "").unwrap();
        let path_vals = vec![dir.to_str().unwrap().to_string()];
        std::env::set_var("PATH", path_vals.join(":"));
        let mut interpreter_paths = python_paths();

        assert_eq!(interpreter_paths.next().unwrap().1, dir.join("python3.11"));
    }

    #[cfg(windows)]
    #[test]
    fn python_search() {
        let dir = tempdir().unwrap().into_path();
        std::fs::write(dir.join("python.exe"), "").unwrap();
        let path_vals = vec![dir.to_str().unwrap().to_string()];
        std::env::set_var("PATH", path_vals.join(":"));
        let mut interpreter_paths = python_paths();

        assert_eq!(interpreter_paths.next().unwrap().1, dir.join("python.exe"));
    }
}
