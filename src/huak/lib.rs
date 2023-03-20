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
    collections::{hash_map::RandomState, HashMap},
    fs::File,
    ops::Not,
    path::{Path, PathBuf},
    str::FromStr,
};
use sys::Terminal;

mod error;
mod fs;
mod git;
mod ops;
mod sys;

const DEFAULT_VENV_NAME: &str = ".venv";
const DEFAULT_PYPROJECT_TOML_CONTENTS: &str = r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = ""
version = "0.0.1"
description = ""
dependencies = []
"#;
const VERSION_OPERATOR_CHARACTERS: [char; 5] = ['=', '~', '!', '>', '<'];

#[cfg(test)]
/// The resource directory found in the Huak repo used for testing purposes.
pub(crate) fn test_resources_dir_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")
}

/// A Python project can be anything from a script to automate some process to a
/// production web application. Projects consist of Python source code and a
/// project-marking `pyproject.toml` file. PEPs provide Python’s ecosystem with
/// standardization and Huak leverages them to do many things such as identify your
/// project. See PEP 621.
#[derive(Default)]
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

    /// Get the Python project's pyproject.toml file.
    pub fn pyproject_toml(&self) -> &PyProjectToml {
        &self.pyproject_toml
    }

    /// Get the Python project's main dependencies listed in the project file.
    pub fn dependencies(&self) -> HuakResult<Vec<Package>> {
        if let Some(dependencies) = self.pyproject_toml.dependencies() {
            return dependencies
                .iter()
                .map(|dep| Package::from_str(dep))
                .collect();
        }
        Ok(Vec::new())
    }

    /// Get a group of optional dependencies from the Python project's project file.
    pub fn optional_dependencey_group(
        &self,
        group_name: &str,
    ) -> HuakResult<Vec<Package>> {
        if let Some(dependencies) =
            self.pyproject_toml.optional_dependencey_group(group_name)
        {
            return dependencies
                .iter()
                .map(|dep| Package::from_str(dep))
                .collect();
        }
        Ok(Vec::new())
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

    /// Write the current project to some directory path.
    pub fn write_project(&self, dir_path: impl AsRef<Path>) -> HuakResult<()> {
        todo!()
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
#[derive(Default)]
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
            return project.version.as_ref().map(|version| version.as_str());
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
        self.project.as_mut().map(|project| {
            if let Some(dependencies) = project.dependencies.as_mut() {
                dependencies.push(package_str.to_string());
            }
        });
    }

    /// Add a Python package as a dependency to the project.
    pub fn add_optional_dependency(
        &mut self,
        package_str: &str,
        group_name: &str,
    ) {
        self.project.as_mut().map(|project| {
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
        });
    }

    /// Remove a dependency from the project.
    pub fn remove_dependency(&mut self, package_str: &str) {
        self.project.as_mut().map(|project| {
            if let Some(dependencies) = project.dependencies.as_mut() {
                if let Some(i) = dependencies
                    .iter()
                    .position(|item| item.contains(package_str))
                {
                    dependencies.remove(i);
                };
            }
        });
    }

    /// Remove an optional dependency from the project.
    pub fn remove_optional_dependency(
        &mut self,
        package_str: &str,
        group_name: &str,
    ) {
        self.project.as_mut().map(|project| {
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
        });
    }

    /// Get the scripts listed in the toml.
    pub fn scripts(&self) -> Option<&IndexMap<String, String, RandomState>> {
        if let Some(project) = self.project.as_ref() {
            return project.scripts.as_ref();
        }
        None
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
            inner: ProjectToml::new(default_pyproject_toml_contents())
                .expect("could not initilize default pyproject.toml"),
        }
    }
}

pub fn default_pyproject_toml_contents() -> &'static str {
    DEFAULT_PYPROJECT_TOML_CONTENTS
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
}

impl VirtualEnvironment {
    /// Create a new virtual environment.
    pub fn new() -> VirtualEnvironment {
        VirtualEnvironment {
            root: PathBuf::new(),
            installer: Installer::new(),
        }
    }

    /// Get a reference to the absolute path to the virtual environment.
    pub fn root(&self) -> &Path {
        self.root.as_ref()
    }

    /// Create a virtual environment from its root path.
    pub fn from_path(path: impl AsRef<Path>) -> HuakResult<VirtualEnvironment> {
        Ok(VirtualEnvironment {
            root: path.as_ref().to_path_buf(),
            installer: Installer::new(),
        })
    }

    /// Get the python environment config.
    fn python_environment_config(&self) -> VirtualEnvironmentConfig {
        todo!()
    }

    /// Create a Python virtual environment on the system.
    pub fn write_venv(&self) -> HuakResult<()> {
        todo!()
    }

    /// The absolute path to the Python environment's python interpreter binary.
    pub fn python_path(&self) -> PathBuf {
        #[cfg(windows)]
        let file_name = "python.exe";
        #[cfg(unix)]
        let file_name = "python";
        self.executables_dir_path().join(file_name)
    }

    /// The version of the Python environment's Python interpreter.
    pub fn python_version(&self) -> Option<Version> {
        self.python_environment_config().version
    }

    /// The absolute path to the Python interpreter used to create the Python
    /// environment.
    pub fn base_python_path(&self) -> PathBuf {
        todo!()
    }

    /// The version of the Python interpreter used to create the Python environment.
    pub fn base_python_version(&self) -> &Version {
        todo!()
    }

    /// The absolute path to the Python environment's executables directory.
    pub fn executables_dir_path(&self) -> PathBuf {
        #[cfg(windows)]
        let dir_name = "Scripts";
        #[cfg(unix)]
        let dir_name = "bin";
        self.root.join(dir_name)
    }

    /// The absolute path to the system's executables directory.
    pub fn base_executables_dir_path(&self) -> &PathBuf {
        todo!()
    }

    /// The absolute path to the Python environment's site-packages directory.
    pub fn site_packages_dir_path(&self) -> &PathBuf {
        todo!()
    }

    /// The absolute path to the system's site-packages directory.
    pub fn base_site_packages_dir_path(&self) -> &PathBuf {
        todo!()
    }

    /// Install many Python packages to the environment.
    pub fn install_packages(&mut self, packages: &[Package]) -> HuakResult<()> {
        todo!()
    }

    /// Uninstall many Python packages from the environment.
    pub fn uninstall_packages(
        &mut self,
        package_names: &[&str],
    ) -> HuakResult<()> {
        todo!()
    }

    /// Get a package from the site-packages directory if it is already installed.
    pub fn find_site_packages_package(&self, name: &str) -> Option<Package> {
        todo!()
    }

    /// Get a package's dist info from the site-packages directory if it is there.
    pub fn find_site_packages_dist_info(&self, name: &str) -> Option<DistInfo> {
        todo!()
    }

    /// Get a package from the system's site-packages directory if it is already
    /// installed.
    pub fn find_base_site_packages_package(
        &self,
        name: &str,
    ) -> Option<Package> {
        todo!()
    }

    /// Get a package's dist info from the system's site-packages directory if it is
    /// there.
    pub fn find_base_site_packages_dist_info(
        &self,
        name: &str,
    ) -> Option<DistInfo> {
        todo!()
    }

    /// Add a package to the site-packages directory.
    fn add_package_to_site_packages(
        &mut self,
        package: &Package,
    ) -> HuakResult<()> {
        todo!()
    }

    /// Add a package to the system's site-packages directory.
    fn add_package_to_base_site_packages(
        &mut self,
        package: &Package,
    ) -> HuakResult<()> {
        todo!()
    }

    /// Remove a package from the site-packages directory.
    fn remove_package_from_site_packages(
        &mut self,
        package: &Package,
    ) -> HuakResult<()> {
        todo!()
    }

    /// Remove a package from the system's site-packages directory.
    fn remove_package_from_base_site_packages(
        &mut self,
        package: &Package,
    ) -> HuakResult<()> {
        todo!()
    }

    /// Check if the Python environment is isolated from any system site-packages
    /// directory.
    pub fn is_isolated(&self) -> bool {
        self.python_environment_config()
            .include_system_site_packages
            .not()
    }

    /// Check if the environment is already activated.
    pub fn is_activated(&self) -> bool {
        if let Some(path) = sys::active_virtual_env_path() {
            return self.root == path;
        }
        false
    }

    /// Activate the Python environment with a given terminal.
    pub fn activate_with_terminal(
        &self,
        terminal: &mut Terminal,
    ) -> HuakResult<()> {
        todo!()
    }

    /// Get all of the packages installed to the environment.
    pub fn installed_packages(&self) -> HuakResult<Vec<Package>> {
        todo!()
    }

    /// Get the environment's installer.
    pub fn installer(&self) -> &Installer {
        &self.installer
    }

    /// Set the environment's installer.
    pub fn set_installer(&mut self, installer: Installer) {
        self.installer = installer;
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
    if let Some(path) = fs::find_file_bottom_up("pyvenv.cfg", cwd, 10)? {
        return Ok(path
            .parent()
            .ok_or(Error::InternalError(
                "failed to establish a parent directory".to_string(),
            ))?
            .to_path_buf());
    }
    Err(Error::VenvNotFoundError)
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
}

#[derive(Default, Copy, Clone)]
pub struct InstallerConfig;

impl InstallerConfig {
    pub fn new() -> InstallerConfig {
        InstallerConfig
    }
}

/// Data about some environment's Python configuration. This abstraction is modeled after
/// the pyenv.cfg file used for Python virtual environments.
pub struct VirtualEnvironmentConfig {
    /// Path to directory containing the Python installation used to create the
    /// environment.
    home: PathBuf,
    /// Boolean value to indicate if the Python environment is isolated from the global
    /// site.
    include_system_site_packages: bool,
    // The version of the environment's Python interpreter.
    version: Option<Version>,
}

impl ToString for VirtualEnvironmentConfig {
    /// Convert the `VirtualEnvironmentConfig` to str.
    fn to_string(&self) -> String {
        todo!()
    }
}

impl Default for VirtualEnvironmentConfig {
    fn default() -> Self {
        Self {
            home: Default::default(),
            include_system_site_packages: Default::default(),
            version: None,
        }
    }
}

/// The python package compliant with packaging.python.og.
/// See <https://peps.python.org/pep-0440/>
/// # Examples
/// ```
/// use huak::Package;
///
/// let python_pkg = Package::from_str("request>=2.28.1").unwrap();
/// println!("I've got 99 {} but huak ain't one", python_pkg);
/// ```
#[derive(Clone, Default)]
pub struct Package {
    /// Name designated to the package by the author(s).
    name: String,
    /// Normalized name of the Python package.
    canonical_name: String,
    /// The package's core metadata.
    /// https://packaging.python.org/en/latest/specifications/core-metadata/
    core_metadata: Option<PackageMetadata>,
    /// The PEP 440 version of the package.
    version: Option<Version>,
    /// The PEP 440 version specifier.
    version_specifier: Option<VersionSpecifier>,
    /// Tags used to indicate platform compatibility.
    platform_tags: Option<Vec<PlatformTag>>,
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
    let res = re.replace_all(name.clone(), "-");
    Ok(res.into_owned())
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.canonical_name == other.canonical_name
            && self.core_metadata == other.core_metadata
            && self.version == other.version
            && self.version_specifier == other.version_specifier
            && self.platform_tags == other.platform_tags
    }
}

impl Eq for Package {}

/// Core package metadata.
/// https://packaging.python.org/en/latest/specifications/core-metadata/
#[derive(PartialEq, Eq, Clone)]
pub struct PackageMetadata;

/// Tags used to indicate platform compatibility.
/// https://packaging.python.org/en/latest/specifications/platform-compatibility-tags/
#[derive(PartialEq, Eq, Clone)]
pub enum PlatformTag {}

/// Package distribtion info stored in the site-packages directory adjacent to the
/// installed package artifact.
/// https://peps.python.org/pep-0376/#one-dist-info-directory-per-installed-distribution
pub struct DistInfo {
    /// File containing the name of the tool used to install the package.
    installer_file: File,
    /// File containing the package's license information.
    license_file: Option<File>,
    /// File containing metadata about the package.
    /// See
    ///   https://peps.python.org/pep-0345/
    ///   https://peps.python.org/pep-0314/
    ///   https://peps.python.org/pep-0241/
    metadata_file: File,
    /// File containing each file isntalled as part of the package's installation.
    /// See https://peps.python.org/pep-0376/#record
    record_file: File,
    /// File added to the .dist-info directory of the installed distribution if the
    /// package was explicitly requested.
    /// See https://peps.python.org/pep-0376/#requested
    requested_file: Option<File>,
    /// File containing metadata about the archive.
    wheel_file: Option<File>,
}

/// A client used to interact with a package index.
pub struct PackageIndexClient;

impl PackageIndexClient {
    pub fn new() -> PackageIndexClient {
        PackageIndexClient
    }

    pub fn query(&self, package: &Package) -> HuakResult<PackageIndexData> {
        let url = format!("https://pypi.org/pypi/{}/json", package.name());
        reqwest::blocking::get(url)?
            .json()
            .map_err(|e| Error::ReqwestError(e))
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

/// Get a hashmap of Python interpreters. Each entry is stored with the interpreter's
/// version as its key and the absolute path the the interpreter as the value.
/// NOTE: This search implementation is inspired by brettcannon/python-launcher
pub fn find_python_interpreter_paths() -> HashMap<Version, PathBuf> {
    let paths = fs::flatten_directories(sys::env_path_values());
    let interpreters = all_python_interpreters_in_paths(paths);
    interpreters
}

fn all_python_interpreters_in_paths(
    paths: impl IntoIterator<Item = PathBuf>,
) -> HashMap<Version, PathBuf> {
    let mut interpreters = HashMap::new();
    paths.into_iter().for_each(|path| {
        python_version_from_path(&path).map_or((), |version| {
            interpreters.entry(version).or_insert(path);
        })
    });

    interpreters
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
    use std::ops::Deref;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn project_bootstrap() {
        todo!()
    }

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
        dbg!(&path);
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
    /// NOTE: This test depends on local virtual environment.
    fn virtual_environment_python_environment_config() {
        let venv = VirtualEnvironment::from_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv"),
        )
        .unwrap();
        let python_path = venv.python_path();
        let venv_root = python_path.parent().unwrap().parent().unwrap();

        assert_eq!(
            venv.python_environment_config().to_string(),
            std::fs::read_to_string(venv_root.join("pyvenv.cfg")).unwrap()
        );
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

    #[ignore = "currently untestable"]
    #[test]
    fn package_with_multiple_version_specifiers() {
        todo!()
    }

    #[test]
    fn package_platform_tags() {
        todo!()
    }

    #[test]
    fn package_core_metadata() {
        todo!()
    }

    #[test]
    fn package_dist_info() {
        todo!();
    }

    #[test]
    fn python_search() {
        let dir = tempdir().unwrap().into_path();
        std::fs::write(dir.join("python3.11"), "").unwrap();
        let path_vals = vec![dir.to_str().unwrap().to_string()];
        std::env::set_var("PATH", path_vals.join(":"));
        let interpreter_paths = find_python_interpreter_paths();

        assert_eq!(
            interpreter_paths
                .get(&Version::from_str("3.11").unwrap())
                .unwrap()
                .deref(),
            dir.join("python3.11")
        );
    }
}
