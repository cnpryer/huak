pub use error::{Error, HuakResult};
use indexmap::IndexMap;
use pep440_rs::{
    parse_version_specifiers, Version as Version440, VersionSpecifier,
};
use pyproject_toml::{Contact, License, PyProjectToml as ProjectToml, ReadMe};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::RandomState, HashMap},
    env::consts::OS,
    ffi::{OsStr, OsString},
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};
use sys::{parse_command_output, Terminal};
pub use sys::{TerminalOptions, Verbosity};
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

/// Configuration for Huak.
///
/// This configuration information is used throughout Huak. At times various
/// implementations need to know things like what directory should a process
/// execute from, what environment information to fallback to, how processes
/// should interact with the terminal, and so on.
struct Config {
    /// The current working directory.
    cwd: PathBuf,
    /// A terminal to use.
    terminal: Terminal,
    /// The environment to use.
    env: Environment,
}

impl Config {
    /// Workspace initialization. Inspired by Cargo.
    fn workspace(&self) -> HuakResult<Workspace> {
        let root = find_workspace_root(self)?;
        Workspace::new(root)
    }

    fn terminal(&mut self) -> Terminal {
        self.terminal
    }
}

/// The Workspace struct.
///
/// This struct defines the workspace various processes would interact with.
/// It contains information about where the workspace is located, where to
/// retrieve metadata about the workspace, how to identify it, etc.
struct Workspace {
    /// The absolute path to the root of the workspace.
    root: PathBuf,
    /// The absolute path to the workspace's manifest file.
    manifest_path: PathBuf,
    /// Any projects a part of the workspace.
    projects: Projects,
    /// Any Python environments associated with the workspace.
    python_environments: PythonEnvironments,
}

impl Workspace {
    fn new<T: AsRef<Path>>(path: T) -> HuakResult<Workspace> {
        let root = path.as_ref();
        let manifest_path = root.join(DEFAULT_MANIFEST_FILE_NAME);
        if !manifest_path.exists() {
            return Err(Error::ManifestNotFound);
        }

        let ws = Workspace {
            root: root.to_path_buf(),
            manifest_path,
            projects: Projects::new(),
            python_environments: PythonEnvironments::new(),
        };

        Ok(ws)
    }

    /// Get the current Python environment by attempting to use one from the
    /// configured cwd. Otherwise try to locate one from the workspace root.
    /// Note that this method prioritizes Venvs.
    fn current_python_environment(
        &mut self,
        config: &Config,
    ) -> HuakResult<&PythonEnvironment> {
        let path = find_venv_root(self.root, config);

        // If the Python environment is already associated with the workspace, return it.
        // Otherwise initialize one if it exists and add it to the associated environments.
        let env = match path {
            Ok(p) => {
                if let Some(e) = self.python_environments.envs.get(&p) {
                    e
                } else {
                    let e = PythonEnvironment::new(p)?;
                    self.python_environments.envs.insert(p.to_path_buf(), e);
                    &e
                }
            }
            Err(e) => return Err(e),
        };

        Ok(env)
    }

    /// Get the current Python project by attempting to use one from configured cwd. Otherwise
    /// initialize one if it exists and add it to the associated projects.
    fn current_project(&mut self, config: &Config) -> HuakResult<&Project> {
        let path = find_project_root(config);

        // If the Python project is already associated with the workspace, return it.
        let project = match path {
            Ok(p) => {
                if let Some(e) = self.projects.projects.get(&p) {
                    e
                } else {
                    let prj = Project::new(p)?;
                    self.projects.projects.insert(p.to_path_buf(), prj);
                    &prj
                }
            }
            Err(e) => return Err(e),
        };

        Ok(project)
    }
}

/// A struct containing multiple Projects.
struct Projects {
    /// A collection of projects identified by their paths.
    projects: HashMap<PathBuf, Project>,
}

impl Projects {
    fn new() -> Projects {
        Projects {
            projects: HashMap::new(),
        }
    }
}

/// A Python project can be anything from a script to automate some process to a
/// production web application. Projects consist of Python source code and a
/// project-marking `pyproject.toml` file. PEPs provide Python’s ecosystem with
/// standardization and Huak leverages them to do many things such as identify your
/// project. See PEP 621.
#[derive(Default, Debug)]
pub struct Project {
    /// A value to indicate the kind of the project (app, library, etc.).
    kind: ProjectKind,
    /// The project's manifest data.
    manifest: Manifest,
    /// The absolute path to the project's manifest file.
    manifest_path: PathBuf,
}

impl Project {
    /// Initialize a `Project` from its root path.
    pub fn new<T: AsRef<Path>>(path: T) -> HuakResult<Project> {
        let root = std::fs::canonicalize(path)?;

        let manifest_path = root.join(DEFAULT_MANIFEST_FILE_NAME);
        if !manifest_path.exists() {
            return Err(Error::ManifestNotFound);
        }

        let pyproject_toml =
            PyProjectToml::new(root.join(DEFAULT_MANIFEST_FILE_NAME))?;

        let mut project = Project {
            kind: ProjectKind::Library,
            manifest: Manifest::from(pyproject_toml),
            manifest_path: manifest_path,
        };

        // If the manifest contains any scripts the project is considered an application.
        // TODO: Should be entry points.
        if project.manifest.scripts.is_some() {
            project.kind = ProjectKind::Application;
        }

        Ok(project)
    }

    /// Get the absolute path to the root directory of the project.
    pub fn root(&self) -> Option<&Path> {
        self.manifest_path.parent()
    }

    /// Get the name of the project.
    pub fn name(&self) -> &String {
        &self.manifest.name
    }

    /// Get the version of the project.
    pub fn version(&self) -> Option<&String> {
        self.manifest.version.as_ref()
    }

    /// Get the path to the manifest file.
    pub fn manifest_path(&self) -> &PathBuf {
        &self.manifest_path
    }

    /// Get the project type.
    pub fn kind(&self) -> &ProjectKind {
        &self.kind
    }

    /// Get the Python project's pyproject.toml file.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Get the Python project's main dependencies listed in the manifest.
    pub fn dependencies(&self) -> Option<&Vec<Dependency>> {
        self.manifest.dependencies.as_ref()
    }

    /// Get the Python project's optional dependencies listed in the manifest.
    pub fn optional_dependencies(
        &self,
    ) -> Option<&IndexMap<String, Vec<Dependency>>> {
        self.manifest.optional_dependencies.as_ref()
    }

    /// Get a group of optional dependencies from the Python project's manifest.
    pub fn optional_dependencey_group(
        &self,
        group: &str,
    ) -> Option<&Vec<Dependency>> {
        self.manifest
            .optional_dependencies
            .map(|item| *item.get(group).unwrap_or(&Vec::new()))
            .as_ref()
    }

    /// Add a Python package as a dependency to the project's manifest.
    pub fn add_dependency(&mut self, dependency: Dependency) -> HuakResult<()> {
        if self.contains_dependency(&dependency)? {
            return Ok(());
        }
        self.manifest
            .dependencies
            .get_or_insert(Vec::new())
            .push(dependency);
        Ok(())
    }

    /// Add a Python package as a dependency to the project' manifest.
    pub fn add_optional_dependency(
        &mut self,
        dependency: Dependency,
        group: &str,
    ) -> HuakResult<()> {
        if self.contains_optional_dependency(&dependency, group)? {
            return Ok(());
        }
        self.manifest
            .optional_dependencies
            .get_or_insert(IndexMap::new())
            .get_mut(group)
            .unwrap_or(&mut Vec::new())
            .push(dependency);
        Ok(())
    }

    /// Remove a dependency from the project's manifest.
    pub fn remove_dependency(
        &mut self,
        dependency: &Dependency,
    ) -> HuakResult<()> {
        if !self.contains_dependency(dependency)? {
            return Ok(());
        }
        if let Some(deps) = self.manifest.dependencies.as_mut() {
            if let Some(i) = deps.iter().position(|item| item.eq(dependency)) {
                deps.remove(i);
            };
        }
        Ok(())
    }

    /// Remove an optional dependency from the project's manifest.
    pub fn remove_optional_dependency(
        &mut self,
        dependency: &Dependency,
        group: &str,
    ) -> HuakResult<()> {
        if !self.contains_optional_dependency(dependency, group)? {
            return Ok(());
        }
        if let Some(deps) = self.manifest.optional_dependencies.as_mut() {
            if let Some(g) = deps.get_mut(group) {
                if let Some(i) = g.iter().position(|item| item.eq(dependency)) {
                    g.remove(i);
                };
            };
        }
        Ok(())
    }

    /// Check if the project has a dependency listed in its manifest.
    pub fn contains_dependency(
        &self,
        dependency: &Dependency,
    ) -> HuakResult<bool> {
        if let Some(deps) = self.dependencies() {
            for d in deps {
                if d.eq(dependency) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Check if the project has an optional dependency listed in its manifest.
    pub fn contains_optional_dependency(
        &self,
        dependency: &Dependency,
        group: &str,
    ) -> HuakResult<bool> {
        if let Some(deps) = self.manifest.optional_dependencies {
            if let Some(g) = deps.get(group) {
                if deps.is_empty() {
                    return Ok(false);
                }
                for d in g {
                    if d.eq(&dependency) {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Check if the project has a dependency listed in its manifest as part of any group.
    pub fn contains_dependency_any(
        &self,
        dependency: &Dependency,
    ) -> HuakResult<bool> {
        if self.contains_dependency(dependency).unwrap_or_default() {
            return Ok(true);
        }

        if let Some(deps) = self.manifest.optional_dependencies {
            if deps.is_empty() {
                return Ok(false);
            }
            for d in deps.values().flatten() {
                if d.eq(&dependency) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Write the manifest file.
    /// Note that this method currently only supports writing a pyproject.toml.
    pub fn write_manifest(&self) -> HuakResult<()> {
        // If the manifest file isn't a pyproject.toml then fail. (TODO: other manifests)
        if self
            .manifest_path
            .file_name()
            .and_then(|raw_file_name| raw_file_name.to_str())
            != Some(DEFAULT_MANIFEST_FILE_NAME)
        {
            return Err(Error::UnimplementedError(format!(
                "unsupported manifest file {}",
                self.manifest_path.display()
            )));
        }

        // If a valie file already exists merge with it and write the file.
        let file = if self.manifest_path.exists() {
            self.merge_pyproject_toml(PyProjectToml::new(self.manifest_path)?)
        } else {
            self.merge_pyproject_toml(PyProjectToml::default())
        };

        file.write_file(self.manifest_path)
    }

    /// Merge the project's manifest data with other pyproject.toml data.
    /// This method prioritizes manfiest data the project utilizes. For everything else
    /// the other data is retained.
    /// 1. toml <- manifest
    /// 2. toml <- other.exclude(manfiest)
    fn merge_pyproject_toml(&self, other: PyProjectToml) -> PyProjectToml {
        let mut pyproject_toml = other.clone();
        pyproject_toml.set_project_name(self.manifest.name);
        if self.manifest.version.is_some() {
            pyproject_toml.set_project_version(self.manifest.version);
        }
        if self.manifest.description.is_some() {
            pyproject_toml.set_project_description(self.manifest.description);
        }
        if self.manifest.authors.is_some() {
            pyproject_toml.set_project_authors(self.manifest.authors);
        }
        if self.manifest.scripts.is_some() {
            pyproject_toml.set_project_scripts(self.manifest.scripts);
        }
        if self.manifest.license.is_some() {
            pyproject_toml.set_project_license(self.manifest.license);
        }
        if self.manifest.readme.is_some() {
            pyproject_toml.set_project_readme(self.manifest.readme);
        }
        if self.manifest.dependencies.is_some() {
            pyproject_toml.set_project_dependencies(
                self.manifest.dependencies.map(|deps| {
                    deps.iter().map(|dep| dep.to_string()).collect()
                }),
            );
        }
        if self.manifest.optional_dependencies.is_some() {
            pyproject_toml.set_project_optional_dependencies(
                self.manifest.optional_dependencies.and_then(|groups| {
                    Some(IndexMap::from_iter(groups.iter().map(
                        |(group, deps)| {
                            (
                                group.clone(),
                                deps.iter()
                                    .map(|dep| dep.to_string())
                                    .collect(),
                            )
                        },
                    )))
                }),
            );
        }
        pyproject_toml
    }
}

/// A project type might indicate if a project is an application-like project or a
/// library-like project.
#[derive(Default, Eq, PartialEq, Debug)]
pub enum ProjectKind {
    /// Library-like projects are essentially anything that isn’t an application. An
    /// example would be a typical Python package distributed to PyPI.
    #[default]
    Library,
    /// Application-like projects are projects intended to be distributed as an executed
    /// process. Examples would include web applications, automated scripts, etc..
    Application,
}

/// Manifest data for `Project`s.
///
/// The manifest contains information about the project including its name, version,
/// dependencies, etc.
#[derive(Default, Debug)]
struct Manifest {
    authors: Option<Vec<Contact>>,
    dependencies: Option<Vec<Dependency>>,
    description: Option<String>,
    scripts: Option<IndexMap<String, String>>,
    license: Option<License>,
    name: String,
    optional_dependencies: Option<IndexMap<String, Vec<Dependency>>>,
    readme: Option<ReadMe>,
    version: Option<String>,
}

/// Initialize a `Manifest` from `PyProjectToml`.
impl From<PyProjectToml> for Manifest {
    fn from(value: PyProjectToml) -> Self {
        let project = match value.project {
            Some(it) => it,
            None => return Self::default(),
        };

        Self {
            authors: project.authors,
            dependencies: project.dependencies.and_then(|items| {
                Some(
                    items
                        .iter()
                        .map(|item| {
                            Dependency::from_str(item)
                                .expect("failed to parse toml dependencies")
                        })
                        .collect::<Vec<Dependency>>(),
                )
            }),
            description: project.description,
            scripts: project.scripts,
            license: project.license,
            name: project.name,
            optional_dependencies: project.optional_dependencies.and_then(|groups| {
                Some(IndexMap::from_iter(groups.iter().map(|(group, deps)| (group.clone(), deps.iter().map(|dep| Dependency::from_str(dep).expect("failed to parse toml optinoal dependencies")).collect()))))
            }),
            readme: project.readme,
            version: project.version,
        }
    }
}

#[derive(Debug)]
/// A Python `Dependency` struct.
struct Dependency {
    /// The dependency's name unmodified.
    name: String,
    /// The canonical dependency name.
    canonical_name: String,
    /// The dependency's PEP440 version specifiers.
    version_specifiers: Option<Vec<VersionSpecifier>>,
}

impl Dependency {
    /// Get the dependency name with its version specifiers as a &str.
    pub fn dependency_string(&self) -> String {
        let specs = match self.version_specifiers {
            Some(it) => it,
            None => {
                return self.name.to_string();
            }
        };

        format!(
            "{}{}",
            self.name,
            specs
                .iter()
                .map(|spec| spec.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }

    fn importable_name(&self) -> HuakResult<String> {
        importable_package_name(&self.canonical_name)
    }
}

/// Display the dependency with the following format "{name}{version specs}"
/// where version specs are comma-delimited.
impl Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dependency_string())
    }
}

impl AsRef<OsStr> for Dependency {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.to_string().as_str()).as_ref()
    }
}

/// Initilize a `Dependency` from a `&str`.
impl FromStr for Dependency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let found = s
            .chars()
            .enumerate()
            .find(|x| VERSION_OPERATOR_CHARACTERS.contains(&x.1));

        let spec = match found {
            Some(it) => &s[it.0..],
            None => {
                return Ok(Dependency {
                    name: s.to_string(),
                    canonical_name: canonical_package_name(s)?,
                    version_specifiers: None,
                });
            }
        };

        let name = s.strip_suffix(&spec).unwrap_or(s).to_string();
        let specs = parse_version_specifiers(spec)
            .map_err(|e| Error::DependencyFromStringError(e.to_string()))?;

        let dependency = Dependency {
            name,
            canonical_name: canonical_package_name(name.as_str())?,
            version_specifiers: Some(specs),
        };

        Ok(dependency)
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_name == other.canonical_name
    }
}

impl Eq for Dependency {}

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
    /// Initilize a `PyProjectToml` from its path.
    pub fn new<T: AsRef<Path>>(path: T) -> HuakResult<PyProjectToml> {
        let contents = std::fs::read_to_string(path)?;
        let pyproject_toml: PyProjectToml = toml::from_str(&contents)?;
        Ok(pyproject_toml)
    }

    pub fn project_name(&self) -> Option<&str> {
        self.project.as_ref().map(|project| project.name.as_str())
    }

    pub fn set_project_name(&mut self, name: String) {
        if let Some(project) = self.project.as_mut() {
            project.name = name;
        }
    }

    pub fn project_version(&self) -> Option<&str> {
        if let Some(project) = self.project.as_ref() {
            return project.version.as_deref();
        }
        None
    }

    pub fn set_project_version(&mut self, version: Option<String>) {
        if let Some(project) = self.project.as_mut() {
            project.version = version;
        }
    }

    pub fn dependencies(&self) -> Option<&Vec<String>> {
        if let Some(project) = self.project.as_ref() {
            return project.dependencies.as_ref();
        }
        None
    }

    pub fn set_project_dependencies(
        &mut self,
        dependencies: Option<Vec<String>>,
    ) {
        if let Some(project) = self.project.as_mut() {
            project.dependencies = dependencies;
        }
    }

    pub fn optional_dependencies(
        &self,
    ) -> Option<&IndexMap<String, Vec<String>>> {
        if let Some(project) = self.project.as_ref() {
            return project.optional_dependencies.as_ref();
        }
        None
    }

    pub fn set_project_optional_dependencies(
        &mut self,
        optional_dependencies: Option<IndexMap<String, Vec<String>>>,
    ) {
        if let Some(project) = self.project.as_mut() {
            project.optional_dependencies = optional_dependencies;
        }
    }

    pub fn set_project_license(&mut self, license: Option<License>) {
        if let Some(project) = self.project.as_mut() {
            project.license = license;
        }
    }

    pub fn set_project_readme(&mut self, readme: Option<ReadMe>) {
        if let Some(project) = self.project.as_mut() {
            project.readme = readme;
        }
    }

    pub fn set_project_scripts(
        &mut self,
        scripts: Option<IndexMap<String, String>>,
    ) {
        if let Some(project) = self.project.as_mut() {
            project.scripts = scripts;
        }
    }

    pub fn set_project_authors(&mut self, authors: Option<Vec<Contact>>) {
        if let Some(project) = self.project.as_mut() {
            project.authors = authors;
        }
    }

    pub fn set_project_description(&mut self, description: Option<String>) {
        if let Some(project) = self.project.as_mut() {
            project.description = description;
        }
    }

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

    pub fn add_dependency(&mut self, dependency: &str) {
        if let Some(project) = self.project.as_mut() {
            if let Some(dependencies) = project.dependencies.as_mut() {
                dependencies.push(dependency.to_string());
            }
        };
    }

    pub fn add_optional_dependency(&mut self, dependency: &str, group: &str) {
        if let Some(project) = self.project.as_mut() {
            let deps =
                project.optional_dependencies.get_or_insert(IndexMap::new());
            if let Some(g) = deps.get_mut(group) {
                g.push(dependency.to_string());
            } else {
                deps.insert(group.to_string(), vec![dependency.to_string()]);
            }
        }
    }

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

    pub fn remove_optional_dependency(
        &mut self,
        dependency: &str,
        group: &str,
    ) {
        if let Some(project) = self.project.as_mut() {
            if let Some(g) = project.optional_dependencies.as_mut() {
                if let Some(deps) = g.get_mut(group) {
                    if let Some(i) =
                        deps.iter().position(|item| item.contains(dependency))
                    {
                        deps.remove(i);
                    };
                };
            }
        };
    }

    pub fn scripts(&self) -> Option<&IndexMap<String, String, RandomState>> {
        if let Some(project) = self.project.as_ref() {
            return project.scripts.as_ref();
        }
        None
    }

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
            inner: ProjectToml::new(&default_pyproject_toml_contents(""))
                .expect("could not initilize default pyproject.toml"),
            tool: None,
        }
    }
}

impl From<Manifest> for PyProjectToml {
    fn from(value: Manifest) -> Self {
        todo!()
    }
}

fn default_project_version_str() -> &'static str {
    DEFAULT_PROJECT_VERSION_STR
}

fn default_virtual_environment_name() -> &'static str {
    DEFAULT_VENV_NAME
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

/// A struct containing multiple PythonEnvironments.
struct PythonEnvironments {
    /// A collection of Python environments identified by their paths.
    envs: HashMap<PathBuf, PythonEnvironment>,
}

impl PythonEnvironments {
    fn new() -> PythonEnvironments {
        PythonEnvironments {
            envs: HashMap::new(),
        }
    }
}

/// The PythonEnvironment struct.
///
/// Python environments are used to execute Python-based processes. Python
/// environments contain a Python interpreter, an executables directory,
/// installed Python packages, etc. This struct is an abstraction for that
/// environment, allowing various processes to interact with Python.
struct PythonEnvironment {
    /// The absolute path to the Python environment's root.
    root: PathBuf,
    /// The absolute path to the Python environment's Python interpreter.
    python_path: PathBuf,
    /// The version of the Python environment's Python interpreter.
    python_version: Version,
    /// The absolute path to the Python environment's executables directory.
    executables_dir_path: PathBuf,
    /// The absolute path to the Python environment's site-packages directory.
    site_packages_path: PathBuf,
    /// The Python package installer associated with the Python environment.
    installer: Option<PackageInstaller>,
    /// The kind of Python environment the environment is.
    kind: PythonEnvironmentKind,
}

impl PythonEnvironment {
    /// Initialize a new `PythonEnvironment`.
    pub fn new<T: AsRef<Path>>(path: T) -> HuakResult<PythonEnvironment> {
        if Some(VENV_CONFIG_FILE_NAME)
            != path
                .as_ref()
                .file_name()
                .and_then(|raw_file_name| raw_file_name.to_str())
        {
            return Err(Error::UnimplementedError(format!(
                "{} is not supported",
                path.as_ref().display()
            )));
        }
        PythonEnvironment::venv(path)
    }

    // TODO: Could instead construct the config and do PythonEnvironment::new(config)
    fn venv<T: AsRef<Path>>(path: T) -> HuakResult<PythonEnvironment> {
        let kind = PythonEnvironmentKind::Venv;
        let root = std::fs::canonicalize(path)?;
        let config =
            VenvConfig::from(root.join(VENV_CONFIG_FILE_NAME).as_ref());
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

        let python_version = if python_version.semver.is_some() {
            python_version
        } else {
            parse_python_interpreter_version(python_path)?.unwrap_or(Version {
                release: python_version.release,
                semver: Some(SemVerVersion {
                    major: python_version.release[0],
                    minor: *python_version.release.get(1).unwrap_or(&0),
                    patch: *python_version.release.get(2).unwrap_or(&0),
                }),
            })
        };

        let semver = match python_version.semver {
            Some(it) => it,
            None => {
                return Err(Error::VenvInvalidConfigFile(format!(
                    "could not parse version from {VENV_CONFIG_FILE_NAME}"
                )))
            }
        };

        // On Unix systems the Venv's site-package directory depends on the Python version.
        // The path is root/lib/pythonX.X/site-packages.
        #[cfg(unix)]
        let site_packages_path = root
            .join("lib")
            .join(format!("python{}.{}", semver.major, semver.minor))
            .join("site-packages");
        #[cfg(unix)]
        let installer = PackageInstaller::Pip(executables_dir_path.join("pip"));
        #[cfg(windows)]
        let site_packages_path = self.root.join("Lib").join("site-packages");
        #[cfg(windows)]
        let installer =
            PackageInstaller::Pip(executables_dir_path.join("pip.exe"));

        let venv = PythonEnvironment {
            root,
            python_path,
            python_version,
            executables_dir_path,
            site_packages_path,
            installer: Some(installer),
            kind,
        };

        Ok(venv)
    }

    /// Get a reference to the absolute path to the python environment.
    pub fn root(&self) -> &Path {
        self.root.as_ref()
    }

    /// The absolute path to the Python environment's python interpreter binary.
    pub fn python_path(&self) -> &PathBuf {
        &self.python_path
    }

    /// Get the version of the Python environment's Python interpreter.
    pub fn python_version(&self) -> &Version {
        &self.python_version
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
    pub fn install_packages<T>(
        &self,
        packages: &[T],
        installer_options: Option<&PackageInstallerOptions>,
        terminal: &mut Terminal,
    ) -> HuakResult<()>
    where
        T: Display + AsRef<OsStr>,
    {
        if let Some(installer) = self.installer {
            installer.install(packages, installer_options, terminal)?;
        }
        Ok(())
    }

    /// Uninstall many Python packages from the environment.
    pub fn uninstall_packages<T>(
        &self,
        packages: &[T],
        installer_options: Option<&PackageInstallerOptions>,
        terminal: &mut Terminal,
    ) -> HuakResult<()>
    where
        T: Display + AsRef<OsStr>,
    {
        if let Some(installer) = self.installer {
            installer.uninstall(packages, installer_options, terminal)?;
        }
        Ok(())
    }

    /// Update many Python packages in the environment.
    pub fn update_packages<T>(
        &self,
        packages: &[T],
        installer_options: Option<&PackageInstallerOptions>,
        terminal: &mut Terminal,
    ) -> HuakResult<()>
    where
        T: Display + AsRef<OsStr>,
    {
        if let Some(installer) = self.installer {
            installer.update(packages, installer_options, terminal)?;
        }
        Ok(())
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

    /// Check if the environment has a package already installed.
    pub fn contains_package(&self, package: &Package) -> HuakResult<bool> {
        Ok(self
            .site_packages_dir_path()
            .join(package.importable_name)
            .exists())
    }
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
            Version::from_str(&parse_command_output(output)?).ok()
        }
    };
    Ok(version)
}

/// Kinds of Python environments.
///
/// Venv
///   executables directory (unix: bin; windows: Scripts)
///   include (windows: Include)
///   lib
///    └── pythonX.Y
///      └── site-packages (windows: Lib/site-packages)
///        ├── some_pkg
///        └── some_pkg-X.X.X.dist-info
///   pyvenv.cfg
///
/// Global
///   An environment created from a global install of Python.
enum PythonEnvironmentKind {
    Venv,
    Global,
}

/// Search for a workspace.
/// 1. If manifest file exists use its directory.
/// 2. Walk from configured cwd up searching for dir containing containing manifest file.
/// 3. Stop after searching the system root (TODO: something more restrictive).
/// Note this is the same search as find_project_root. This will change in the future.
pub fn find_workspace_root(config: &Config) -> HuakResult<PathBuf> {
    let path = match fs::find_root_file_bottom_up(
        DEFAULT_MANIFEST_FILE_NAME,
        config.cwd.as_ref(),
        Path::new("/"),
    ) {
        Ok(it) => it
            .ok_or(Error::ManifestNotFound)?
            .parent()
            .ok_or(Error::InternalError(
                "failed to parse parent directory".to_string(),
            ))?
            .to_path_buf(),
        Err(_) => return Err(Error::ManifestNotFound),
    };
    Ok(path)
}

/// Search for a Python project.
/// 1. If manifest file exists use its directory.
/// 2. Walk from configured cwd up searching for dir containing containing manifest file.
/// 3. Stop after searching the system root (TODO: something more restrictive).
pub fn find_project_root(config: &Config) -> HuakResult<PathBuf> {
    let path = match fs::find_root_file_bottom_up(
        DEFAULT_MANIFEST_FILE_NAME,
        config.cwd.as_ref(),
        Path::new("/"),
    ) {
        Ok(it) => it
            .ok_or(Error::ManifestNotFound)?
            .parent()
            .ok_or(Error::InternalError(
                "failed to parse parent directory".to_string(),
            ))?
            .to_path_buf(),
        Err(_) => return Err(Error::ManifestNotFound),
    };
    Ok(path)
}

/// Search for a Python virtual environment.
/// 1. If VIRTUAL_ENV exists then a venv is active; use it.
/// 2. Walk from configured cwd up searching for dir containing the Python environment config file.
/// 3. Stop after searching `stop_after`.
pub fn find_venv_root<T: AsRef<Path>>(
    stop_after: T,
    config: &Config,
) -> HuakResult<PathBuf> {
    if let Ok(path) = std::env::var("VIRTUAL_ENV") {
        return Ok(PathBuf::from(path));
    }

    let file_path = match fs::find_root_file_bottom_up(
        VENV_CONFIG_FILE_NAME,
        config.cwd.as_ref(),
        stop_after.as_ref(),
    ) {
        Ok(it) => it.ok_or(Error::PythonEnvironmentNotFoundError)?,
        Err(_) => return Err(Error::PythonEnvironmentNotFoundError),
    };

    let root = file_path.parent().ok_or(Error::InternalError(
        "failed to establish parent directory".to_string(),
    ))?;

    Ok(root.to_path_buf())
}

/// Data about some environment's Python configuration. This abstraction is modeled after
/// the pyenv.cfg file used for Python virtual environments.
struct VenvConfig {
    /// The version of the environment's Python interpreter.
    version: Version,
}

impl From<&Path> for VenvConfig {
    fn from(value: &Path) -> Self {
        // Read the file and flatten the lines for parsing.
        let file = File::open(value)
            .expect(format!("failed to open {}", value.display()).as_str());
        let buff_reader = BufReader::new(file);
        let lines: Vec<String> = buff_reader.lines().flatten().collect();

        // Search for version = "X.X.X"
        let mut version;
        lines.iter().for_each(|item| {
            let mut split = item.splitn(2, '=');
            let key = split.next().unwrap_or_default().trim();
            let val = split.next().unwrap_or_default().trim();
            if key == "version" {
                version = Version::from_str(val).expect(
                    format!("failed to parse version from {}", value.display())
                        .as_str(),
                );
            }
        });

        VenvConfig { version }
    }
}

/// Kinds of Python package installers.
///
/// Pip
///   The absolute path to `pip`.
enum PackageInstaller {
    /// The `pip` Python package installer.
    Pip(PathBuf),
}

impl PackageInstaller {
    pub fn new() -> PackageInstaller {
        PackageInstaller::new()
    }

    pub fn install<T>(
        &self,
        packages: &[T],
        options: Option<&PackageInstallerOptions>,
        terminal: &mut Terminal,
    ) -> HuakResult<()>
    where
        T: Display + AsRef<OsStr>,
    {
        match self {
            PackageInstaller::Pip(path) => {
                let mut cmd = Command::new(path);
                cmd.arg("install")
                    .args(packages.iter().map(|item| item.to_string()));

                if let Some(PackageInstallerOptions::Pip { args }) = options {
                    if let Some(args) = args.as_ref() {
                        cmd.args(args.iter().map(|item| item.as_str()));
                    }
                }

                terminal.run_command(&mut cmd)
            }
            _ => Err(Error::UnimplementedError(
                "unsupported package installer".to_string(),
            )),
        }
    }

    pub fn uninstall<T>(
        &self,
        packages: &[T],
        options: Option<&PackageInstallerOptions>,
        terminal: &mut Terminal,
    ) -> HuakResult<()>
    where
        T: Display + AsRef<OsStr>,
    {
        match self {
            PackageInstaller::Pip(path) => {
                let mut cmd = Command::new(path);
                cmd.arg("uninstall").args(packages).arg("-y");

                if let Some(PackageInstallerOptions::Pip { args }) = options {
                    if let Some(args) = args.as_ref() {
                        cmd.args(args.iter().map(|item| item.as_str()));
                    }
                }

                terminal.run_command(&mut cmd)
            }
            _ => Err(Error::UnimplementedError(
                "unsupported package installer".to_string(),
            )),
        }
    }

    pub fn update<T>(
        &self,
        packages: &[T],
        options: Option<&PackageInstallerOptions>,
        terminal: &mut Terminal,
    ) -> HuakResult<()>
    where
        T: Display + AsRef<OsStr>,
    {
        match self {
            PackageInstaller::Pip(path) => {
                let mut cmd = Command::new(path);
                cmd.args(["install", "--upgrade"]).args(packages);

                if let Some(PackageInstallerOptions::Pip { args }) = options {
                    if let Some(args) = args.as_ref() {
                        cmd.args(args.iter().map(|item| item.as_str()));
                    }
                }

                terminal.run_command(&mut cmd)
            }
            _ => Err(Error::UnimplementedError(
                "unsupported package installer".to_string(),
            )),
        }
    }
}

/// `PacakgeInstaller` options.
///
/// Use `PackageInstallerOptions` to modify configuration used to install packages.
/// Pip can be given a vector of CLI args.
pub enum PackageInstallerOptions {
    Pip { args: Option<Vec<String>> },
}

/// A Context that can be utilized by any process.
///
/// This is *additional* context (other than the implied context) that various
/// processes could utilize. This would include extra information about the
/// environment.
pub struct Context {
    /// The context's environment data.
    env: Environment,
}

/// Enrionment data.
///
/// A generic struct containing information about the environment. This includes
/// environment variables and their values.
pub struct Environment {
    env: HashMap<OsString, OsString>,
}

/// The Version struct.
///
/// This is a generic version abstraction.
struct Version {
    release: Vec<usize>,
    semver: Option<SemVerVersion>,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^(\d+)(?:\.(\d+))?(?:\.(\d+))?$")?;
        let captures = match re.captures(s) {
            Some(captures) => captures,
            None => return Err(Error::InvalidVersionString(s.to_string())),
        };

        let major = captures[1]
            .parse::<usize>()
            .map_err(|e| Error::InternalError(e.to_string()))?;
        let minor = captures[2]
            .parse::<usize>()
            .map_err(|e| Error::InternalError(e.to_string()))?;
        let patch = captures[3]
            .parse::<usize>()
            .map_err(|e| Error::InternalError(e.to_string()))?;

        let release = vec![major, minor, patch];
        let semver = Some(SemVerVersion {
            major,
            minor,
            patch,
        });

        Ok(Version { release, semver })
    }
}

/// A `SemVerVersion` struct for Semantic Version numbers.
///
/// Example `SemVerVersion { major: 3, minor: 11, patch: 0}
struct SemVerVersion {
    major: usize,
    minor: usize,
    patch: usize,
}

/// The python package compliant with packaging.python.og.
/// See <https://peps.python.org/pep-0440/>
#[derive(Clone, Debug)]
pub struct Package {
    /// Name designated to the package by the author(s).
    name: String,
    /// Normalized name of the Python package.
    canonical_name: String,
    /// Importable name of the Python package.
    importable_name: String,
    /// The PEP 440 version of the package.
    version: Version440,
}

impl Package {
    /// Get the name of the package.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the normalized name of the package.
    pub fn canonical_name(&self) -> &str {
        self.canonical_name.as_ref()
    }

    /// Get the importable version of the package's name.
    pub fn importable_name(&self) -> &str {
        self.importable_name.as_ref()
    }

    /// Get the package's PEP440 version.
    pub fn version(&self) -> &Version440 {
        &self.version
    }
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

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_name == other.canonical_name
    }
}

impl Eq for Package {}

/// Collect and return an iterator over `Dependency`s.
fn dependency_iter<I>(strings: I) -> impl Iterator<Item = Dependency>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    strings
        .into_iter()
        .filter_map(|item| Dependency::from_str(item.as_ref()).ok())
}

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

pub struct WorkspaceOptions {
    pub uses_git: bool,
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
        let pyproject_toml = PyProjectToml::new(&path).unwrap();

        assert_eq!(pyproject_toml.project_name().unwrap(), "mock_project");
        assert_eq!(pyproject_toml.project_version().unwrap(), "0.0.1");
        assert!(pyproject_toml.dependencies().is_some())
    }

    #[test]
    fn toml_to_string_pretty() {
        let path = test_resources_dir_path()
            .join("mock-project")
            .join("pyproject.toml");
        let pyproject_toml = PyProjectToml::new(&path).unwrap();

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
    /// NOTE: This test depends on local virtual environment.
    fn virtual_environment_executable_dir_name() {
        let venv = PythonEnvironment::new(
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
    fn dependency_from_str() {
        let dep = Dependency::from_str("package_name==0.0.0").unwrap();

        assert_eq!(dep.dependency_string(), "package_name==0.0.0");
        assert_eq!(dep.name, "package_name");
        assert_eq!(dep.canonical_name, "package-name");
        assert_eq!(
            *dep.version_specifiers.unwrap(),
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
