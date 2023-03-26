pub use error::{Error, HuakResult};
use indexmap::IndexMap;
use pep440_rs::{
    parse_version_specifiers, Operator as VersionOperator, Version,
    VersionSpecifier,
};
use pyproject_toml::PyProjectToml as ProjectToml;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::RandomState,
    env::consts::OS,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};
use sys::Terminal;
pub use sys::Verbosity;

mod error;
mod fs;
mod git;
pub mod ops;
mod sys;

const DEFAULT_VIRTUAL_ENVIRONMENT_NAME: &str = ".venv";
const VIRTUAL_ENVIRONMENT_CONFIG_FILE_NAME: &str = "pyvenv.cfg";
const VERSION_OPERATOR_CHARACTERS: [char; 5] = ['=', '~', '!', '>', '<'];

#[cfg(test)]
/// The resource directory found in the Huak repo used for testing purposes.
pub(crate) fn test_resources_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dev-resources")
}

/// A Python project can be anything from a script to automate some process to a
/// production web application. Projects consist of Python source code and a
/// project-marking `pyproject.toml` file. PEPs provide Python’s ecosystem with
/// standardization and Huak leverages them to do many things such as identify your
/// project. See PEP 621.
#[derive(Default, Debug)]
pub struct Project {
    /// A value to indicate the type of the project (app, library, etc.).
    project_type: ProjectType,
    /// Data about the project's layout.
    project_layout: ProjectLayout,
    /// The project's pyproject.toml file containing metadata about the project.
    /// See https://peps.python.org/pep-0621/
    pyproject_toml: PyProjectToml,
}

impl Project {
    /// Create a new project.
    pub fn new() -> Project {
        Project {
            project_type: ProjectType::Library,
            project_layout: ProjectLayout {
                root: PathBuf::new(),
                pyproject_toml_path: PathBuf::new(),
            },
            pyproject_toml: PyProjectToml::new(),
        }
    }

    /// Create a project from its manifest file path.
    pub fn from_manifest(path: impl AsRef<Path>) -> HuakResult<Project> {
        let path = path.as_ref();
        let mut project = Project::new();
        project.pyproject_toml = PyProjectToml::from_path(path)?;
        project.project_layout = ProjectLayout {
            root: path
                .parent()
                .ok_or(Error::ProjectRootMissingError)?
                .to_path_buf(),
            pyproject_toml_path: path.to_path_buf(),
        };
        Ok(project)
    }

    /// Get the absolute path to the root directory of the project.
    pub fn root(&self) -> &PathBuf {
        &self.project_layout.root
    }

    /// Get the name of the project.
    pub fn name(&self) -> HuakResult<&str> {
        self.pyproject_toml
            .project_name()
            .ok_or(Error::InternalError("project name not found".to_string()))
    }

    /// Get the version of the project.
    pub fn version(&self) -> HuakResult<&str> {
        self.pyproject_toml
            .project_version()
            .ok_or(Error::InternalError(
                "project version not found".to_string(),
            ))
    }

    /// Get the path to the manifest file.
    pub fn manifest_path(&self) -> &PathBuf {
        &self.project_layout.pyproject_toml_path
    }

    /// Get the project type.
    pub fn project_type(&self) -> &ProjectType {
        &self.project_type
    }

    /// Get the Python project's pyproject.toml file.
    pub fn pyproject_toml(&self) -> &PyProjectToml {
        &self.pyproject_toml
    }

    /// Get the Python project's main dependencies listed in the project file.
    pub fn dependencies(&self) -> Option<&Vec<String>> {
        self.pyproject_toml.dependencies()
    }

    /// Get a group of optional dependencies from the Python project's project file.
    pub fn optional_dependencey_group(
        &self,
        group_name: &str,
    ) -> Option<&Vec<String>> {
        self.pyproject_toml.optional_dependencey_group(group_name)
    }

    /// Add a Python package as a dependency to the project's project file.
    pub fn add_dependency(&mut self, package_str: &str) {
        self.pyproject_toml.add_dependency(package_str);
    }

    /// Add a Python package as a dependency to the project' project file.
    pub fn add_optional_dependency(
        &mut self,
        package_str: &str,
        group_name: &str,
    ) {
        self.pyproject_toml
            .add_optional_dependency(package_str, group_name)
    }

    /// Remove a dependency from the project's project file.
    pub fn remove_dependency(&mut self, package_str: &str) {
        self.pyproject_toml.remove_dependency(package_str);
    }

    /// Remove an optional dependency from the project's project file.
    pub fn remove_optional_dependency(
        &mut self,
        package_str: &str,
        group_name: &str,
    ) {
        self.pyproject_toml
            .remove_optional_dependency(package_str, group_name);
    }
}

impl From<ProjectType> for Project {
    fn from(value: ProjectType) -> Self {
        let mut project = Project::new();
        project.project_type = value;
        project
    }
}

/// A project type might indicate if a project is an application-like project or a
/// library-like project.
#[derive(Default, Eq, PartialEq, Debug)]
pub enum ProjectType {
    /// Library-like projects are essentially anything that isn’t an application. An
    /// example would be a typical Python package distributed to PyPI.
    #[default]
    Library,
    /// Application-like projects are projects intended to be distributed as an executed
    /// process. Examples would include web applications, automated scripts, etc..
    Application,
}

/// Data about the project's layout. The project layout includes the location of
/// important files and directories.
#[derive(Default, Debug)]
pub struct ProjectLayout {
    /// The absolute path to the root directory of the project.
    root: PathBuf,
    /// The absolute path to the pyproject.toml file.
    pyproject_toml_path: PathBuf,
}

/// A pyproject.toml as specified in PEP 517
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PyProjectToml {
    #[serde(flatten)]
    inner: ProjectToml,
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
    /// Create new pyproject.toml data.
    pub fn new() -> PyProjectToml {
        PyProjectToml::default()
    }

    /// Create new pyproject.toml data from a pyproject.toml's path.
    pub fn from_path(path: impl AsRef<Path>) -> HuakResult<PyProjectToml> {
        let contents = std::fs::read_to_string(path)?;
        let pyproject_toml: PyProjectToml = toml::from_str(&contents)?;
        Ok(pyproject_toml)
    }

    /// Get the project name.
    pub fn project_name(&self) -> Option<&str> {
        self.project.as_ref().map(|project| project.name.as_str())
    }

    /// Set the project name listed in the toml.
    pub fn set_project_name(&mut self, name: &str) {
        if let Some(project) = self.project.as_mut() {
            project.name = name.to_string();
        }
    }

    /// Get the project version.
    pub fn project_version(&self) -> Option<&str> {
        if let Some(project) = self.project.as_ref() {
            return project.version.as_deref();
        }
        None
    }

    /// Get the Python project's main dependencies.
    pub fn dependencies(&self) -> Option<&Vec<String>> {
        if let Some(project) = self.project.as_ref() {
            return project.dependencies.as_ref();
        }
        None
    }

    /// Get a group of optional dependencies from the Python project.
    pub fn optional_dependencey_group(
        &self,
        group_name: &str,
    ) -> Option<&Vec<String>> {
        if let Some(project) = self.project.as_ref() {
            if let Some(dependencies) = &project.optional_dependencies {
                return dependencies.get(group_name);
            }
        }
        None
    }

    /// Add a Python package as a dependency to the project.
    pub fn add_dependency(&mut self, package_str: &str) {
        if let Some(project) = self.project.as_mut() {
            if let Some(dependencies) = project.dependencies.as_mut() {
                dependencies.push(package_str.to_string());
            }
        };
    }

    /// Add a Python package as a dependency to the project.
    pub fn add_optional_dependency(
        &mut self,
        package_str: &str,
        group_name: &str,
    ) {
        if let Some(project) = self.project.as_mut() {
            if let Some(map) = project.optional_dependencies.as_mut() {
                if let Some(dependencies) = map.get_mut(group_name) {
                    dependencies.push(package_str.to_string());
                } else {
                    map.insert(
                        group_name.to_string(),
                        vec![package_str.to_string()],
                    );
                };
            }
        };
    }

    /// Remove a dependency from the project.
    pub fn remove_dependency(&mut self, package_str: &str) {
        if let Some(project) = self.project.as_mut() {
            if let Some(dependencies) = project.dependencies.as_mut() {
                if let Some(i) = dependencies
                    .iter()
                    .position(|item| item.contains(package_str))
                {
                    dependencies.remove(i);
                };
            }
        };
    }

    /// Remove an optional dependency from the project.
    pub fn remove_optional_dependency(
        &mut self,
        package_str: &str,
        group_name: &str,
    ) {
        if let Some(project) = self.project.as_mut() {
            if let Some(group) = project.optional_dependencies.as_mut() {
                if let Some(dependencies) = group.get_mut(group_name) {
                    if let Some(i) = dependencies
                        .iter()
                        .position(|item| item.contains(package_str))
                    {
                        dependencies.remove(i);
                    };
                };
            }
        };
    }

    /// Get the scripts listed in the toml.
    pub fn scripts(&self) -> Option<&IndexMap<String, String, RandomState>> {
        if let Some(project) = self.project.as_ref() {
            return project.scripts.as_ref();
        }
        None
    }

    /// Add a new console script to the toml.
    pub fn add_script(
        &mut self,
        name: &str,
        entrypoint: &str,
    ) -> HuakResult<()> {
        if let Some(project) = self.project.as_mut() {
            if let Some(scripts) = project.scripts.as_mut() {
                scripts.insert_full(name.to_string(), entrypoint.to_string());
            } else {
                project.scripts = Some(IndexMap::from([(
                    name.to_string(),
                    entrypoint.to_string(),
                )]));
            }
        }
        Ok(())
    }

    /// Save the toml contents to a filepath.
    pub fn write_file(&self, path: impl AsRef<Path>) -> HuakResult<()> {
        let string = self.to_string_pretty()?;
        Ok(std::fs::write(path, string)?)
    }

    /// Convert the toml struct to a formatted String.
    pub fn to_string_pretty(&self) -> HuakResult<String> {
        Ok(toml_edit::ser::to_string_pretty(&self)?)
    }

    /// Convert the toml to a string as-is.
    pub fn to_string(&self) -> HuakResult<String> {
        Ok(toml_edit::ser::to_string(&self)?)
    }
}

impl Default for PyProjectToml {
    fn default() -> Self {
        Self {
            inner: ProjectToml::new(&default_pyproject_toml_contents(""))
                .expect("could not initilize default pyproject.toml"),
        }
    }
}

fn default_virtual_environment_name() -> &'static str {
    DEFAULT_VIRTUAL_ENVIRONMENT_NAME
}

fn default_pyproject_toml_contents(project_name: &str) -> String {
    format!(
        r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "{project_name}"
version = "0.0.1"
description = ""
dependencies = []
    "#
    )
}

fn default_init_file_contents(version: &str) -> String {
    format!(
        r#"__version__ = "{version}"
"#
    )
}

fn default_entrypoint_string(importable_name: &str) -> String {
    format!("{importable_name}.main:main")
}

fn default_test_file_contents(importable_name: &str) -> String {
    format!(
        r#"from {importable_name} import __version__


def test_version():
    __version__
"#
    )
}

fn default_main_file_contents() -> String {
    r#"def main():
    print("Hello, World!")


if __name__ == "__main__":
    main()
"#
    .to_string()
}

/// A PEP-compliant Python environment API.
///
/// Python environments contain the following:
///   executables directory (unix: bin; windows: Scripts)
///   include (windows: Include)
///   lib
///    └── pythonX.Y
///      └── site-packages (windows: Lib/site-packages)
///        ├── some_pkg
///        └── some_pkg-X.X.X.dist-info
///   pyvenv.cfg
#[derive(Default)]
pub struct VirtualEnvironment {
    /// Absolute path to the root of the virtual environment directory.
    root: PathBuf,
    /// The installer the virtual environment uses to install python packages.
    installer: Installer,
    /// The pyvenv.cfg data.
    config: VirtualEnvironmentConfig,
}

impl VirtualEnvironment {
    /// Create a new virtual environment.
    pub fn new() -> VirtualEnvironment {
        VirtualEnvironment {
            root: PathBuf::new(),
            installer: Installer::new(),
            config: VirtualEnvironmentConfig::new(),
        }
    }

    /// Get a reference to the absolute path to the virtual environment.
    pub fn root(&self) -> &Path {
        self.root.as_ref()
    }

    /// Create a virtual environment from its root path.
    pub fn from_path(path: impl AsRef<Path>) -> HuakResult<VirtualEnvironment> {
        let path = path.as_ref();
        let mut venv = VirtualEnvironment {
            root: path.to_path_buf(),
            installer: Installer::new(),
            config: VirtualEnvironmentConfig::from_path(
                path.join(VIRTUAL_ENVIRONMENT_CONFIG_FILE_NAME),
            )?,
        };
        let mut installer = Installer::new();
        installer.set_config(InstallerConfig {
            path: venv.executables_dir_path().join("pip"),
        });
        venv.installer = installer;
        Ok(venv)
    }

    /// The absolute path to the Python environment's python interpreter binary.
    pub fn python_path(&self) -> PathBuf {
        #[cfg(windows)]
        let file_name = "python.exe";
        #[cfg(unix)]
        let file_name = "python";
        self.executables_dir_path().join(file_name)
    }

    /// Get the version of the Python environment's Python interpreter.
    pub fn python_version(&self) -> Option<&Version> {
        self.config.version.as_ref()
    }

    /// The absolute path to the Python environment's executables directory.
    pub fn executables_dir_path(&self) -> PathBuf {
        #[cfg(windows)]
        let dir_name = "Scripts";
        #[cfg(unix)]
        let dir_name = "bin";
        self.root.join(dir_name)
    }

    /// The absolute path to the Python environment's site-packages directory.
    pub fn site_packages_dir_path(&self) -> HuakResult<PathBuf> {
        let path = match OS {
            "windows" => self.root.join("Lib").join("site-packages"),
            _ => {
                let version = match self.python_version() {
                    Some(it) => it,
                    None => {
                        return Err(Error::VenvInvalidConfigFile(
                            "missing version".to_string(),
                        ))
                    }
                };
                self.root
                    .join("lib")
                    .join(format!(
                        "python{}",
                        version
                            .release
                            .iter()
                            .take(2)
                            .map(|&x| x.to_string())
                            .collect::<Vec<String>>()
                            .join(".")
                    ))
                    .join("site-packages")
            }
        };
        Ok(path)
    }

    /// Install Python packages to the environment.
    pub fn install_packages(
        &self,
        packages: &[Package],
        terminal: &mut Terminal,
    ) -> HuakResult<()> {
        for package in packages {
            self.installer.install(package, terminal)?;
        }
        Ok(())
    }

    /// Uninstall many Python packages from the environment.
    pub fn uninstall_packages(
        &self,
        package_names: &[&str],
        terminal: &mut Terminal,
    ) -> HuakResult<()> {
        for package_name in package_names {
            self.installer.uninstall(package_name, terminal)?;
        }
        Ok(())
    }

    /// Check if the environment is already activated.
    pub fn is_active(&self) -> bool {
        if let Some(path) = sys::active_virtual_env_path() {
            return self.root == path;
        }
        if let Some(path) = sys::active_conda_env_path() {
            return self.root == path;
        }
        false
    }

    /// Check if the environment has a module installed to the executables directory.
    pub fn has_module(&self, module_name: &str) -> HuakResult<bool> {
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

    /// Check if the environment has a package already installed.
    pub fn has_package(&self, package: &Package) -> HuakResult<bool> {
        Ok(self
            .site_packages_dir_path()?
            .join(to_importable_package_name(package.name())?)
            .exists())
    }
}

/// Search for a Python virtual environment.
/// 1. If VIRTUAL_ENV exists then a venv is active; use it.
/// 2. Walk from CWD up searching for dir containing pyvenv.cfg.
pub fn find_venv_root() -> HuakResult<PathBuf> {
    if let Ok(path) = std::env::var("VIRTUAL_ENV") {
        return Ok(PathBuf::from(path));
    }
    let cwd = std::env::current_dir()?;
    if let Some(path) =
        fs::find_file_bottom_up(VIRTUAL_ENVIRONMENT_CONFIG_FILE_NAME, cwd, 10)?
    {
        return Ok(path
            .parent()
            .ok_or(Error::InternalError(
                "failed to establish a parent directory".to_string(),
            ))?
            .to_path_buf());
    }
    Err(Error::VenvNotFoundError)
}

#[derive(Default)]
/// Data about some environment's Python configuration. This abstraction is modeled after
/// the pyenv.cfg file used for Python virtual environments.
struct VirtualEnvironmentConfig {
    // The version of the environment's Python interpreter.
    version: Option<Version>,
}

impl VirtualEnvironmentConfig {
    pub fn new() -> VirtualEnvironmentConfig {
        VirtualEnvironmentConfig { version: None }
    }

    pub fn from_path(
        path: impl AsRef<Path>,
    ) -> HuakResult<VirtualEnvironmentConfig> {
        let file = File::open(path)?;
        let buff_reader = BufReader::new(file);
        let lines: Vec<String> = buff_reader.lines().flatten().collect();
        let mut version = None;
        lines.iter().for_each(|item| {
            let mut vals = item.splitn(2, " = ");
            let name = vals.next().unwrap_or_default();
            let value = vals.next().unwrap_or_default();
            if name == "version" {
                version = Version::from_str(value).ok()
            }
        });

        Ok(VirtualEnvironmentConfig { version })
    }
}

/// A struct for managing installing packages.
#[derive(Default)]
pub struct Installer {
    /// Configuration for package installation
    config: InstallerConfig,
}

impl Installer {
    pub fn new() -> Installer {
        Installer {
            config: InstallerConfig::new(),
        }
    }

    pub fn config(&self) -> &InstallerConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: InstallerConfig) {
        self.config = config;
    }

    pub fn install(
        &self,
        package: &Package,
        terminal: &mut Terminal,
    ) -> HuakResult<()> {
        let mut cmd = Command::new(self.config.path.clone());
        cmd.arg("install").arg(package.dependency_string());
        terminal.run_command(&mut cmd)
    }

    pub fn uninstall(
        &self,
        package_name: &str,
        terminal: &mut Terminal,
    ) -> HuakResult<()> {
        let mut cmd = Command::new(self.config.path.clone());
        cmd.arg("uninstall").arg(package_name).arg("-y");
        terminal.run_command(&mut cmd)
    }
}

#[derive(Default, Clone)]
pub struct InstallerConfig {
    path: PathBuf,
}

impl InstallerConfig {
    pub fn new() -> InstallerConfig {
        InstallerConfig {
            path: PathBuf::from("pip"),
        }
    }
}

/// The python package compliant with packaging.python.og.
/// See <https://peps.python.org/pep-0440/>
/// # Examples
/// ```
/// use huak::Package;
/// use std::str::FromStr;
///
/// let python_pkg = Package::from_str("request>=2.28.1").unwrap();
/// println!("I've got 99 {} but huak ain't one", python_pkg.name());
/// ```
#[derive(Clone, Default, Debug)]
pub struct Package {
    /// Name designated to the package by the author(s).
    name: String,
    /// Normalized name of the Python package.
    canonical_name: String,
    /// The PEP 440 version of the package.
    version: Option<Version>,
    /// The PEP 440 version specifier.
    version_specifier: Option<VersionSpecifier>,
}

impl Package {
    /// Get the name of the package.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the normalized name of the package.
    pub fn cononical_name(&self) -> &str {
        self.canonical_name.as_ref()
    }

    /// Get the package's PEP440 version.
    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    /// Get the pacakge's PEP440 version specifier.
    pub fn version_specifier(&self) -> Option<&VersionSpecifier> {
        self.version_specifier.as_ref()
    }

    /// Get the pacakge's PEP440 version operator.
    pub fn version_operator(&self) -> Option<&VersionOperator> {
        if let Some(it) = &self.version_specifier {
            return Some(it.operator());
        }
        None
    }

    /// Get the pacakge name with its version specifier as a &str.
    pub fn dependency_string(&self) -> String {
        let specifier = match self.version_specifier() {
            Some(it) => it,
            None => {
                return self.name.clone();
            }
        };
        format!(
            "{}{}{}",
            self.name,
            specifier.operator(),
            specifier.version()
        )
    }
}

/// Instantiate a PythonPackage struct from a String
/// # Arguments
///
/// * 'pkg_string' - A string slice representing PEP-0440 python package
///
/// # Examples
/// ```
/// use huak::Package;
/// use std::str::FromStr;
///
/// let my_pkg = Package::from_str("requests==2.28.1");
/// ```
impl FromStr for Package {
    type Err = Error;

    fn from_str(pkg_string: &str) -> HuakResult<Package> {
        // TODO: Improve the method used to parse the version portion
        // Search for the first character that isn't part of the package's name
        let found = pkg_string
            .chars()
            .enumerate()
            .find(|x| VERSION_OPERATOR_CHARACTERS.contains(&x.1));

        let spec_str = match found {
            Some(it) => &pkg_string[it.0..],
            None => {
                return Ok(Package {
                    name: pkg_string.to_string(),
                    canonical_name: to_package_cononical_name(pkg_string)?,
                    ..Default::default()
                });
            }
        };

        // TODO: More than one specifier
        match parse_version_specifiers(spec_str) {
            Ok(vspec) => match vspec.first() {
                Some(it) => {
                    let name = match pkg_string.strip_suffix(&spec_str) {
                        Some(it) => it,
                        None => pkg_string,
                    };

                    Ok(Package {
                        name: name.to_string(),
                        canonical_name: to_package_cononical_name(name)?,
                        version_specifier: Some(it.clone()),
                        ..Default::default()
                    })
                }
                None => Ok(Package {
                    name: pkg_string.to_string(),
                    canonical_name: to_package_cononical_name(pkg_string)?,
                    version_specifier: None,
                    ..Default::default()
                }),
            },
            Err(e) => Err(Error::PackageFromStringError(e.to_string())),
        }
    }
}

fn to_package_cononical_name(name: &str) -> HuakResult<String> {
    let re = Regex::new("[-_. ]+")?;
    let res = re.replace_all(name, "-");
    Ok(res.into_owned())
}

fn to_importable_package_name(name: &str) -> HuakResult<String> {
    let cononical_name = to_package_cononical_name(name)?;
    Ok(cononical_name.replace('-', "_"))
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.canonical_name == other.canonical_name
            && self.version == other.version
            && self.version_specifier == other.version_specifier
    }
}

impl Eq for Package {}

/// A client used to interact with a package index.
#[derive(Default)]
pub struct PackageIndexClient;

impl PackageIndexClient {
    pub fn new() -> PackageIndexClient {
        PackageIndexClient
    }

    pub fn query(&self, package: &Package) -> HuakResult<PackageIndexData> {
        let url = format!("https://pypi.org/pypi/{}/json", package.name());
        reqwest::blocking::get(url)?
            .json()
            .map_err(Error::ReqwestError)
    }
}

/// Data about a package from a package index.
// TODO: Support more than https://pypi.org/pypi/<package name>/json
//       Ex: See https://peps.python.org/pep-0503/
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageIndexData {
    pub info: PackageInfo,
    last_serial: u64,
    releases: serde_json::value::Value,
    urls: Vec<serde_json::value::Value>,
    vulnerabilities: Vec<serde_json::value::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageInfo {
    pub author: String,
    pub author_email: String,
    pub bugtrack_url: serde_json::value::Value,
    pub classifiers: Vec<String>,
    pub description: String,
    pub description_content_type: String,
    pub docs_url: serde_json::value::Value,
    pub download_url: serde_json::value::Value,
    pub downloads: serde_json::value::Value,
    pub home_page: serde_json::value::Value,
    pub keywords: serde_json::value::Value,
    pub license: serde_json::value::Value,
    pub maintainer: serde_json::value::Value,
    pub maintainer_email: serde_json::value::Value,
    pub name: String,
    pub package_url: String,
    pub platform: serde_json::value::Value,
    pub project_url: String,
    pub project_urls: serde_json::value::Value,
    pub release_url: String,
    pub requires_dist: serde_json::value::Value,
    pub requires_python: String,
    pub summary: String,
    pub version: String,
    pub yanked: bool,
    pub yanked_reason: serde_json::value::Value,
}

/// Get an iterator over available Python interpreter paths parsed from PATH.
/// Inspired by brettcannon/python-launcher
pub fn find_python_interpreter_paths(
) -> impl Iterator<Item = (PathBuf, Version)> {
    let paths = fs::flatten_directories(sys::env_path_values());
    all_python_interpreters_in_paths(paths)
}

fn all_python_interpreters_in_paths(
    paths: impl IntoIterator<Item = PathBuf>,
) -> impl Iterator<Item = (PathBuf, Version)> {
    paths
        .into_iter()
        .map(|item| (item.clone(), python_version_from_path(item.as_path())))
        .filter_map(|(path, version)| version.map(|v| (path, v)))
}

/// Parse a Python interpreter's version from its path if one exists.
fn python_version_from_path(path: impl AsRef<Path>) -> Option<Version> {
    path.as_ref()
        .file_name()
        .or(None)
        .and_then(|raw_file_name| raw_file_name.to_str().or(None))
        .and_then(|file_name| {
            if valid_python_interpreter_file_name(file_name) {
                Version::from_str(&file_name["python".len()..]).ok()
            } else {
                None
            }
        })
}

fn valid_python_interpreter_file_name(file_name: &str) -> bool {
    file_name.len() >= "python3.0".len() && file_name.starts_with("python")
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
        let pyproject_toml = PyProjectToml::from_path(&path).unwrap();

        assert_eq!(pyproject_toml.project_name().unwrap(), "mock_project");
        assert_eq!(pyproject_toml.project_version().unwrap(), "0.0.1");
        assert!(pyproject_toml.dependencies().is_some())
    }

    #[test]
    fn toml_to_string_pretty() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let pyproject_toml = PyProjectToml::from_path(&path).unwrap();

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
        let pyproject_toml = PyProjectToml::from_path(path).unwrap();

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
        let pyproject_toml = PyProjectToml::from_path(path).unwrap();

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
        let mut pyproject_toml = PyProjectToml::from_path(path).unwrap();

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
        let mut pyproject_toml = PyProjectToml::from_path(path).unwrap();

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
        let mut pyproject_toml = PyProjectToml::from_path(path).unwrap();

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
        let mut pyproject_toml = PyProjectToml::from_path(path).unwrap();

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
    /// NOTE: This test depends on local virtual environment.
    fn virtual_environment_executable_dir_name() {
        let venv = VirtualEnvironment::from_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();

        assert!(venv.executables_dir_path().exists());
        #[cfg(unix)]
        assert!(venv.executables_dir_path().join("python").exists());
        #[cfg(windows)]
        assert!(venv.executables_dir_path().join("python.exe").exists());
    }

    #[test]
    fn package_from_str() {
        let package = Package::from_str("package_name==0.0.0").unwrap();

        assert_eq!(package.dependency_string(), "package_name==0.0.0");
        assert_eq!(package.name(), "package_name");
        assert_eq!(package.cononical_name(), "package-name");
        assert_eq!(
            *package.version_operator().unwrap(),
            pep440_rs::Operator::Equal
        );
        assert_eq!(
            package.version_specifier().unwrap().version().to_string(),
            "0.0.0"
        );
        assert_eq!(package.version(), None); // TODO
    }

    #[test]
    fn python_search() {
        let dir = tempdir().unwrap().into_path();
        std::fs::write(dir.join("python3.11"), "").unwrap();
        let path_vals = vec![dir.to_str().unwrap().to_string()];
        std::env::set_var("PATH", path_vals.join(":"));
        let mut interpreter_paths = find_python_interpreter_paths();

        assert_eq!(interpreter_paths.next().unwrap().0, dir.join("python3.11"));
    }
}
