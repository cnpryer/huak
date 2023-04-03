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
use pyproject_toml::{
    Contact, License, Project, PyProjectToml as ProjectToml, ReadMe,
};
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
const DEFAULT_MANIFEST_FILE_NAME: &str = "pyproject.toml";

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
    /// Terminal options to use.
    terminal_options: TerminalOptions,
}

impl Config {
    /// Resolve the current workspace based on the `Config` data.
    fn workspace(&self) -> Workspace {
        Workspace::from(&self.workspace_root)
    }
}

/// The `Workspace` is a useful struct for reolving things like the current `Package`
/// or the `Environment` itself.
///
/// ```
/// use huak::Workspace;
///
/// let workspace = Workspace::from(".");
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

/// Initialize a `Workspace` from a `Path`-like.
///
/// ```
/// use huak::Workspace;
///
/// let workspace = Workspace::from(".");
/// ```
impl<T> From<T> for Workspace
where
    T: AsRef<Path>,
    T: Into<PathBuf>,
{
    fn from(value: T) -> Self {
        todo!()
    }
}

impl Workspace {
    /// Resolve the current `Environment` for the `Workspace`.
    fn environmet(&self) -> Environment {
        Environment::new()
    }

    /// Resolve a `Package` pulling the current or creating one if none is found.
    fn resolve_package(&self) -> Package {
        todo!()
    }

    /// Resolve the current `Package`. The current `Package` is one found by its
    /// metadata file nearest baseed on `Config` data.
    fn current_package(&self) -> Package {
        todo!()
    }

    /// Resolve the `Package`s for the `Workspace`.
    fn packages(&self) -> Vec<Package> {
        todo!()
    }

    /// Resolve a `PythonEnvironment` pulling the current or creating one if none is found.
    fn resolve_python_environment(&self) -> PythonEnvironment {
        todo!()
    }

    /// Resolve the current `PythonEnvironment`. The current `PythonEnvironment` is one
    /// found by its configuration file or `Interpreter` nearest baseed on `Config` data.
    fn current_python_environment(&self) -> PythonEnvironment {
        todo!()
    }

    /// Resolve the `PythonEnvironment`s for the `Workspace`.
    fn python_environments(&self) -> Vec<PythonEnvironment> {
        todo!()
    }

    /// Create a `PythonEnvironment` for the `Workspace`.
    fn new_python_environment(&self) -> HuakResult<PythonEnvironment> {
        let python_path = match self.env.python_paths().next() {
            Some(it) => it,
            None => return Err(Error::PythonNotFoundError),
        };

        let name = DEFAULT_VENV_NAME;
        let path = self.root.join(name);

        let args = ["-m", "venv", name];
        let mut cmd = Command::new(python_path);
        cmd.args(args).current_dir(&self.root);

        self.env.terminal().run_command(&mut cmd)?;

        PythonEnvironment::new(path)
    }
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
        Ok(it) => it.ok_or(Error::PythonEnvironmentNotFoundError)?,
        Err(_) => return Err(Error::PythonEnvironmentNotFoundError),
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
    /// Python interpreters installed on the system.
    interpreters: Interpreters,
    /// Original environment variables.
    env: Env,
}

impl Environment {
    /// Initialize an `Environment`.
    fn new() -> Environment {
        todo!()
    }

    /// Get a `Terminal` from the `Environment`.
    fn terminal(&self) -> Terminal {
        Terminal::new()
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

struct Interpreters {
    interpreters: Vec<Interpreter>,
}

impl Interpreters {
    /// Get the latest `Interpreter` by `Version`.
    fn latest(&self) -> Option<Interpreter> {
        todo!()
    }

    /// Get an exact `Interpreter` by `Version`.
    fn exact(&self, version: &Version) -> Option<Interpreter> {
        todo!()
    }

    /// Get an interpreter by its `PathBuf`.
    fn get<T: AsRef<Path>>(&self, path: T) -> Option<Interpreter> {
        todo!()
    }
}

/// The `PythonEnvironmentAPI` is an interface for `PythonEnvironment` defining the
/// environment agnostic logic they would implement.
///
/// ```
/// use huak::PythonEnvironmentAPI;
///
/// fn iter_packages<T: PythonEnvironmentAPI>(env: T) -> impl Iterator<Item = &Package> {
///     env.packages()
/// }
/// ```
trait PythonEnvironmentAPI {
    /// Get a referance to the `PythonEnvironment`'s `Interpreter`.
    fn interpreter(&self) -> &Interpreter;
    /// Get an `Iterator` over the `PythonEnvironment`'s `Package`s.
    fn packages<'a>(&'a self) -> PackageIter<'a>;
    /// Get a reference to the `Installer` for the `PythonEnvironment`.
    fn installer(&self) -> &Installer;
    /// Install a `Package` to the `PythonEnvironment`.
    fn install(&self, package: &Package);
    /// Install `Package`s to the `PythonEnvironment`.
    fn install_many(&self, packages: &PackageIter);
    /// Uninstall a `Package` from the `PythonEnvironment`.
    fn uninstall(&self, package: &Package);
    /// Uninstall `Package`s from the `PythonEnvironment`.
    fn uninstall_many(&self, packages: &PackageIter);
    /// Update a `Package` already installed in the `PythonEnvironment`.
    fn update(&self, package: &Package);
    /// Update `Package`s from the `PythonEnvironment`.
    fn update_many(&self, packages: &PackageIter);
}

/// The `PythonEnvironment` can be a `Venv` or `Global`.
///
/// `PythonEnvironment`s implement an `API` useful for interacting with an `Interpreter`
/// and its related environment.
enum PythonEnvironment {
    /// The virtual environment. See https://peps.python.org/pep-0405/.
    Venv(Venv),
}

impl PythonEnvironment {
    fn new<T: AsRef<Path>>(path: T) -> HuakResult<PythonEnvironment> {
        todo!()
    }
}

impl<'a> PythonEnvironmentAPI for PythonEnvironment {
    fn interpreter(&self) -> &Interpreter {
        match self {
            PythonEnvironment::Venv(venv) => &venv.interpreter,
        }
    }

    fn packages<'b>(&'b self) -> PackageIter<'b> {
        match self {
            PythonEnvironment::Venv(venv) => PackageIter {
                iter: venv.packages.iter(),
            },
        }
    }

    fn installer(&self) -> &Installer {
        match self {
            PythonEnvironment::Venv(venv) => &venv.installer,
        }
    }

    fn install(&self, package: &Package) {
        todo!()
    }

    fn install_many(&self, packages: &PackageIter) {
        todo!()
    }

    fn uninstall(&self, package: &Package) {
        todo!()
    }

    fn uninstall_many(&self, packages: &PackageIter) {
        todo!()
    }

    fn update(&self, package: &Package) {
        todo!()
    }

    fn update_many(&self, packages: &PackageIter) {
        todo!()
    }
}

/// The `Venv` is a Python virtual environment as described in PEP405.
///
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
/// lib/python{version-major.version-minor}. `pyvenv.cfg` is the `Venv`'s
/// config file and contains information like the version of the Python
/// interpreter, *potentially* the "home" path to the Python interpreter that
/// created the `Venv`, etc.
///
/// ```
/// use huak::Venv;
///
/// let venv = Venv::new(".venv");
/// ```
struct Venv {
    /// The absolute path to the `Venv`'s root.
    root: PathBuf,
    /// The `Venv`'s Python `Interpreter`.
    interpreter: Interpreter,
    /// The abolute path to the `Venv`'s executables directory. This directory contains
    /// installed Python modules and the `Interpreter` the `Venv` uses. On Windows this
    /// is located at `Venv.root\Scripts\`, otherwise it's located at `Venv.root/bin/`
    executables_dir_path: PathBuf,
    /// The site-packages directory contains all of the `Venv`'s installed Python
    /// packages.
    site_packages_path: PathBuf,
    /// The `Venv`'s installed `Package`s.
    packages: Vec<Package>,
    /// The `Installer` the `Venv` uses to install Python `Package`s.
    installer: Installer,
}

impl Venv {
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
    /// The `Package` `Metadata` containing information about the `Package`.
    /// See https://packaging.python.org/en/latest/specifications/core-metadata/#core-metadata.
    metadata: Metadata,
    /// The `Package`'s canonical name.
    canonical_name: String,
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

        let metadata = Metadata {
            name: name.to_string(),
            description: String::new(),
            version: version_specifer.version().to_owned(),
            authors: None,
            license: None,
            dependencies: None,
            optional_dependencies: None,
            readme: None,
            scripts: None,
        };

        let package = Package {
            metadata,
            canonical_name: canonical_package_name(name.as_ref())?,
        };

        Ok(package)
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=={}", self.canonical_name, self.metadata.version)
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_name == other.canonical_name
    }
}

impl Eq for Package {}

/// The core `Metadata` for Python `Package`s.
///
/// The `Metadata` contains information about the `Package` such as its name, version,
/// authors, dependencies, and more.
///
/// ```
/// use huak::Metadata;
/// use pep440_rs::Version;
///
/// let metadata = Metadata {
///     name: String::from("my-project"),
///     version: Version::from_str("0.0.1"),
///     dependencies: Vec::new(),
///     optional_dependencies: IndexMap::new(),
///     ..Default
/// }
/// ```
struct Metadata {
    /// The name of the `Package`.
    name: String,
    /// The description of the `Package`.
    description: String,
    /// The PEP440-compliant `Version`. See https://peps.python.org/pep-0440/.
    version: PEP440Version,
    /// The authors of the `Package`.
    authors: Option<Vec<Contact>>,
    /// The license for the `Package`.
    license: Option<License>,
    /// The `Pacakge`'s `Dependency`s (cheap abstraction of the `Package`).
    dependencies: Option<Vec<Dependency>>,
    /// The `Package`'s optional `Dependency`s.
    optional_dependencies: Option<IndexMap<String, Vec<Dependency>>>,
    /// The `Package`'s README.
    readme: Option<ReadMe>,
    /// The `Package`'s scripts.
    scripts: Option<IndexMap<String, String>>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            version: PEP440Version::from_str("0.0.1").unwrap(),
            ..Default::default()
        }
    }
}

/// Initialize a `Manifest` from `PyProjectToml`.
impl From<PyProjectToml> for Metadata {
    fn from(value: PyProjectToml) -> Self {
        let project = match value.project.as_ref() {
            Some(it) => it,
            None => return Self::default(),
        };

        Self {
            authors: project.authors.clone(),
            dependencies: parse_toml_depenencies(project),
            description: project.description.unwrap_or(String::new()),
            scripts: project.scripts,
            license: project.license,
            name: project.name,
            optional_dependencies: parse_toml_optional_dependencies(project),
            readme: project.readme,
            version: PEP440Version::from_str(
                project.version.get_or_insert(String::from("0.0.1")),
            )
            .expect("failed to parse version from pyproject.toml"),
        }
    }
}

fn parse_toml_depenencies(project: &Project) -> Option<Vec<Dependency>> {
    project.dependencies.as_ref().map(|items| {
        items
            .iter()
            .map(|item| {
                Dependency::from_str(item)
                    .expect("failed to parse toml dependencies")
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
                        Dependency::from_str(dep).expect(
                            "failed to parse toml optinoal dependencies",
                        )
                    })
                    .collect(),
            )
        }))
    })
}

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

impl<T> From<T> for PyProjectToml
where
    T: AsRef<Path>,
    T: Into<PathBuf>,
{
    fn from(value: T) -> Self {
        let contents = std::fs::read_to_string(value).expect("");
        let pyproject_toml: PyProjectToml =
            toml::from_str(&contents).expect("");

        pyproject_toml
    }
}

impl PyProjectToml {
    /// Initialize a `PyProjectToml` from its path.
    pub fn new<T: AsRef<Path>>(path: T) -> HuakResult<PyProjectToml> {
        let pyproject_toml = PyProjectToml::from(path.as_ref());

        Ok(pyproject_toml)
    }

    pub fn project_name(&self) -> Option<&str> {
        self.project.as_ref().map(|project| project.name.as_str())
    }

    pub fn set_project_name(&mut self, name: String) {
        self.project.as_mut().map(|project| project.name = name);
    }

    pub fn project_version(&self) -> Option<&str> {
        self.project
            .as_ref()
            .and_then(|project| project.version.as_deref())
    }

    pub fn set_project_version(&mut self, version: Option<String>) {
        self.project
            .as_mut()
            .map(|project| project.version = version);
    }

    pub fn dependencies(&self) -> Option<&[String]> {
        self.project
            .as_ref()
            .and_then(|project| project.dependencies.as_deref())
    }

    pub fn set_project_dependencies(
        &mut self,
        dependencies: Option<Vec<String>>,
    ) {
        self.project
            .as_mut()
            .map(|project| project.dependencies = dependencies);
    }

    pub fn optional_dependencies(
        &self,
    ) -> Option<&IndexMap<String, Vec<String>>> {
        self.project
            .as_ref()
            .and_then(|project| project.optional_dependencies.as_ref())
    }

    pub fn set_project_optional_dependencies(
        &mut self,
        optional_dependencies: Option<IndexMap<String, Vec<String>>>,
    ) {
        self.project.as_mut().map(|project| {
            project.optional_dependencies = optional_dependencies
        });
    }

    pub fn set_project_license(&mut self, license: Option<License>) {
        self.project
            .as_mut()
            .map(|project| project.license = license);
    }

    pub fn set_project_readme(&mut self, readme: Option<ReadMe>) {
        self.project.as_mut().map(|project| project.readme = readme);
    }

    pub fn set_project_scripts(
        &mut self,
        scripts: Option<IndexMap<String, String>>,
    ) {
        self.project
            .as_mut()
            .map(|project| project.scripts = scripts);
    }

    pub fn set_project_authors(&mut self, authors: Option<Vec<Contact>>) {
        self.project
            .as_mut()
            .map(|project| project.authors = authors);
    }

    pub fn set_project_description(&mut self, description: Option<String>) {
        self.project
            .as_mut()
            .map(|project| project.description = description);
    }

    pub fn optional_dependencey_group(
        &self,
        group: &str,
    ) -> Option<&Vec<String>> {
        self.project
            .as_ref()
            .and_then(|p| p.optional_dependencies.as_ref())
            .and_then(|deps| deps.get(group))
    }

    pub fn add_dependency(&mut self, dependency: &str) {
        self.project.as_mut().map(|project| {
            project
                .dependencies
                .as_mut()
                .map(|deps| deps.push(dependency.to_string()))
        });
    }

    pub fn add_optional_dependency(&mut self, dependency: &str, group: &str) {
        self.project.as_mut().map(|project| {
            project
                .optional_dependencies
                .as_mut()
                .get_or_insert(&mut IndexMap::new())
                .entry(group.to_string())
                .or_insert_with(Vec::new)
                .push(dependency.to_string())
        });
    }

    pub fn remove_dependency(&mut self, dependency: &str) {
        self.project
            .as_mut()
            .and_then(|project| project.dependencies.as_mut())
            .filter(|deps| deps.iter().any(|dep| dep.contains(dependency)))
            .map(|deps| {
                let i = deps
                    .iter()
                    .position(|dep| dep.contains(dependency))
                    .unwrap();
                deps.remove(i);
            });
    }

    pub fn remove_optional_dependency(
        &mut self,
        dependency: &str,
        group: &str,
    ) {
        self.project
            .as_mut()
            .and_then(|project| project.optional_dependencies.as_mut())
            .and_then(|g| g.get_mut(group))
            .and_then(|deps| {
                deps.iter()
                    .position(|dep| dep.contains(dependency))
                    .map(|i| deps.remove(i))
            });
    }

    pub fn scripts(&self) -> Option<&IndexMap<String, String, RandomState>> {
        self.project
            .as_ref()
            .and_then(|project| project.scripts.as_ref())
    }

    pub fn add_script(&mut self, name: &str, entrypoint: &str) {
        self.project.as_mut().map(|project| {
            project
                .scripts
                .get_or_insert(IndexMap::new())
                .entry(name.to_string())
                .or_insert(entrypoint.to_string())
        });
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> HuakResult<()> {
        let string = self.to_string_pretty()?;
        Ok(std::fs::write(path, string)?)
    }

    pub fn to_string_pretty(&self) -> HuakResult<String> {
        Ok(toml_edit::ser::to_string_pretty(&self)?)
    }

    pub fn to_string(&self) -> HuakResult<String> {
        Ok(toml_edit::ser::to_string(&self)?)
    }
}

impl Default for PyProjectToml {
    fn default() -> Self {
        Self {
            inner: ProjectToml::new(&default_pyproject_toml_contents())
                .expect("could not Initialize default pyproject.toml"),
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
    /// The `Dependency` name.
    name: String,
    /// The canonical name of the `Dependency`.
    canonical_name: String,
    /// The PEP440-compliant `VersionSpecifiers`. See https://peps.python.org/pep-0440/.
    version_specifiers: VersionSpecifiers,
    /// Boolean indicating the `Dependency` is optional.
    optional: bool,
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
            name: name.to_string(),
            canonical_name: canonical_package_name(name.as_ref())?,
            version_specifiers,
            optional: false,
        };

        Ok(dependency)
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Remove any whitespace from the version specifiers.
        let version_specifiers = self.version_specifiers.iter().map(|spec| {
            spec.to_string()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("")
        });

        write!(
            f,
            "{}{}",
            self.name,
            version_specifiers.collect::<Vec<_>>().join(","),
        )
    }
}

impl AsRef<OsStr> for Dependency {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self)
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_name == other.canonical_name
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

/// The `Installer` is used to install Python `Package`s to a `PythonEnvironment`.
///
/// `Installer`s can be native implementations or the installer distributed with
/// `Python`, Pip.
///
/// ```
/// use huak::Installer;
///
/// let installer = Installer::Pip(PathBuf::from("pip"));
/// ```
enum Installer {
    Pip(PathBuf),
}

/// Initialize an `Installer` from a `Path`-like.
///
/// ```
/// use huak::Installer;
///
/// let installer = Installer::from("pip");
/// ```
impl<T> From<T> for Installer
where
    T: AsRef<Path>,
    T: Into<PathBuf>,
{
    fn from(value: T) -> Self {
        todo!()
    }
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

/// A generic `Version` struct implemnted Semantic-version-compliant `Version`s.
///
/// This struct is mainly used for the Python `Interpreter`.
struct Version {
    major: usize,
    minor: usize,
    patch: usize,
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

        let version = Version {
            major: release[0],
            minor: release[1],
            patch: release[2],
        };

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
        match compare_semver(&self, &other) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

fn compare_semver(this: &Version, other: &Version) -> Ordering {
    for (a, b) in [
        (this.major, other.major),
        (this.minor, this.minor),
        (this.patch, other.patch),
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
        let pyproject_toml = PyProjectToml::new(path).unwrap();

        assert_eq!(pyproject_toml.project_name().unwrap(), "mock_project");
        assert_eq!(pyproject_toml.project_version().unwrap(), "0.0.1");
        assert!(pyproject_toml.dependencies().is_some())
    }

    #[test]
    fn toml_to_string_pretty() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let pyproject_toml = PyProjectToml::new(path).unwrap();

        assert_eq!(
            pyproject_toml.to_string_pretty().unwrap(),
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
        let pyproject_toml = PyProjectToml::new(path).unwrap();

        assert_eq!(
            pyproject_toml.dependencies().unwrap().deref(),
            vec!["click==8.1.3"]
        );
    }

    #[test]
    fn toml_optional_dependencies() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let pyproject_toml = PyProjectToml::new(path).unwrap();

        assert_eq!(
            pyproject_toml
                .optional_dependencey_group("dev")
                .unwrap()
                .deref(),
            vec!["pytest>=6", "black==22.8.0", "isort==5.12.0",]
        );
    }

    #[test]
    fn toml_add_dependency() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let mut pyproject_toml = PyProjectToml::new(path).unwrap();

        pyproject_toml.add_dependency("test");
        assert_eq!(
            pyproject_toml.to_string_pretty().unwrap(),
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
        let mut pyproject_toml = PyProjectToml::new(path).unwrap();

        pyproject_toml.add_optional_dependency("test1", "dev");
        pyproject_toml.add_optional_dependency("test2", "new-group");
        assert_eq!(
            pyproject_toml.to_string_pretty().unwrap(),
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
        let mut pyproject_toml = PyProjectToml::new(path).unwrap();

        pyproject_toml.remove_dependency("click");
        assert_eq!(
            pyproject_toml.to_string_pretty().unwrap(),
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
        let mut pyproject_toml = PyProjectToml::new(path).unwrap();

        pyproject_toml.remove_optional_dependency("isort", "dev");
        assert_eq!(
            pyproject_toml.to_string_pretty().unwrap(),
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
        let dep = Dependency::from_str("package_name==0.0.0").unwrap();

        assert_eq!(dep.to_string(), "package_name==0.0.0");
        assert_eq!(dep.name, "package_name");
        assert_eq!(dep.canonical_name, "package-name");
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
