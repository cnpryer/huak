This document contains structured historical plans for developing Huak.


# Huak Workspaces

## Summary

Workspaces define some scope on a users' system. Every workspace has a root. Workspaces are often resolved by searching for a matching file target (or many targets).

## Motivation

I don't like how coupled workspaces currently are with the rest of the system. This pulls it out and defines it as a kind of common context from which things can occur. It's up to the rest of the library to figure out how it might be useful.

## Requirements

- Should simplify code
- Should allow for composability


## Details

Some workspace data includes:

- root

Huak executes different operations often *from* a workspace. So its useful to define operations that occur for and against some place on the filesystem.

A workspace can be initialized to a root, or it can be resolved for a path `workspace.resolve_with_target("pyproject.toml")`.


### Composability

Since workspaces are a simple wrapper for some scope on a system you should be able to construct data that allows for nested scopes.

```rust
// dir (root)
// └── project (member)
// | └── pyproject.toml
// └── pyproject.toml
let dir = TempDir::new().unwrap();
let mock = create_mock_ws(dir.as_ref());
let cwd = mock.join("package");
let ws = resolve_root(cwd, PathMarker::file("pyproject.toml"));

assert!(ws.root().exists());
assert_eq!(ws.root(), dir.path());
```

This is useful for Huak since resolving a workspace can include resolving packages within a workspace. It's on the rest of Huak to make *project experience* good.

### Decoupling

When building the toolchain implementation for Huak it would have been useful to have a common interface for workspace resolution and usage. For example the toolchain can benefit from having an easy way to define a scope for the application. settings.toml will track what scopes in the system have been configured to use a specific toolchain installation.

This would also benefit Huak's development environment experience for projects and virtual environments. The same logic could be used for providing virtual environment mapping for projects.

## Questions

- Should environment data be decoupled from workspaces?
- Should the terminal and other platform concepts also be decoupled?



# Huak's Toolchain


## Summary

Like Cargo, Huak's toolchain would offer common developer workflows support via standard tooling such as Python interpreters, formatters, linters, type-checkers, testing frameworks, and more. 


## Motivation

Being able to streamline the development process has been a major motivator for developing Huak. The goal has been to funnel important DX through Huak, centralizing tooling used to develop Python projects.


## Requirements

An implementation of Huak's toolchain must:

- Include standard tools
  - A Python interpreter
  - A formatter
  - A linter
  - A type-checker
  - A testing framework
- Include channels
  - There should be sound defaults.
  - Projects should be able to *use* any toolchain.
  - Huak should remember what toolchains are used for each project it intends to manage.
  - There should be a reasonable versioning strategy for channels.
- Start minimal
  - If a user doesn't want to use a toolchain they shouldn't have to.
  - Users should be able to add tools to toolchains.
- Maximize performance
- Be able to evolve as Huak does
  - UX should remain relatively non-breaking but underlying tools/implementations can change.
- Be able to be used without Huak's CLI
  - Toolchain tools should be able to be added to PATH.
  - Toolchains can be installed to target paths other than Huak's default toolchain installation behavior.
- Implement self-management capabilities


## Details

### CLI

The following is an incomplete demonstration of the CLI planned:

```
huak toolchain update <tool>           # Update the current toolchain tools
huak toolchain install <channel>       # Install a toolchain via its channel
huak toolchain uninstall <channel>     # Uninstall a toolchain via its channel
huak toolchain use <channel>           # Use a toolchain via its channel
huak toolchain add <tool>              # Add a tool to the current toolchain
huak toolchain remove <tool>           # Remove a tool from the current toolchain
huak toolchain run <tool>              # Run a tool installed to the current toolchain.
huak toolchain info                    # Display the current toolchain's information
huak toolchain list                    # Display available toolchains
```


### Concepts

#### Installing toolchains

Until Huak is distributed as a standalone application (not via PyPI) it will continue to behave how it currently does (using available Python environments). When the user runs `huak toolchain install` Huak will either recognize there is already a toolchain installed and available or it will install the latest toolchain available.

Users can install toolchains using the `huak toolchain install` command.

Without any other arguments `install` will:

- Install the latest toolchain available if Huak doesn't already have one set up.
- Install the toolchain associated with the project's Python environment if it isn't already installed.
  - If a `[tool.huak.toolchain]` table is used in the project's pyproject.toml file it will install it if it isn't already installed.
  - It will attempt to figure out which toolchain to install by checking for a related virtual environment.
  - If no virtual environment is associated with the project it will install a toolchain compatible with the latest version of Python available on the system.
  - If no Python is found on the system it will install the latest available version.

Users can specify channels to install by running `huak toolchain install <channel>`.

So if a user wanted to install the default toolchain associated with Python 3.12 the following command would be used:

```
huak toolchain install 3.12
```

Users can install toolchains for their own non-Huak usage by using `--target` (see *"Without Huak"*).

Toolchains can be uniquely identified by their platform targets -- which can currently be derived by the Python release installed to the toolchain. This includes:
- kind - Defaults to 'cpython'
- version
- os (support windows, macos, linux)
- architecture
- build configuration (optional)

When a toolchain is installed a virtual environment is created to maintain any tools used by Huak. Toolchains installed with Huak are keyed into a settings.toml db found in Huak's home directory.

A toolchain installed with `cpython-3.12.0-apple-aarch64`:
```
❯ huak toolchain list
    Installed 
           1) cpython-3.12.0-apple-aarch64
```

The process of resolving releases for toolchain channels will change over time, so it would be nice to include configuration for resolution behavior (defining a 'nightly'; staying bleeding-edge with certain toolchain tools).

The goal is to maintain the smallest required installation of Python for the projects Huak manages. The first-pass will attempt to install tools to the bin directory as proxies to the original download. Some systems will use hardlinks. And some systems might require full copies of the download.

#### Updating toolchains

Users can update a toolchain by using the `huak toolchain update` command. By default this command will attempt to resolve the latest versions compatible with the current toolchain. If there is no currently active toolchain this command will silently fail.

Users can update individual tools installed to a toolchain by using `huak toolchain update <tool>`. The following command would update `ruff` for the currently active toolchain:

```
huak toolchain update ruff
```

#### Uninstalling toolchains

Toolchains can be uninstalled using `huak toolchain uninstall`. This will remove the currently active toolchain. Using `huak toolchain uninstall <channel>` will attempt to uninstall a toolchain associated with the requested version.

#### Using toolchains

By default Huak will attempt to *use* the latest available toolchain. If one isn't installed it will install the latest available. Telling Huak to *use* a toolchain will:

- Override the default if a project can't be associated with the command.
- Override the relationship defined Huak's settings.toml if one already exists. If one does not exist it will be added to the settings.toml file.

In order to resolve a toolchain this behavior will follow the same logic defined in *"Installing toolchains"*.

#### Using channels

`<channel>` is the the requested toolchain channel to use. Versions can be used for channels. *Using* `"3.12"` would key the current scope with the toolchain channel `"3.12"` into the settings.toml db. The resolved toolchain would include Python installed with the latest default release options available for that channel.

To use a channel for a pyproject.toml-managed project add the `[tool.huak.toolchain]` table:

```toml
[tool.huak.toolchain]
channel = 3.12
```

See *"Pyproject.toml `[tool.huak.toolchain]`"* for more.

##### `Channel`

- Default channel - A channel called 'default' that is installed by default when no toolchin is present. Any time a project uses a toolchain if there isn't a toolchain keyed in the settings.toml for that scope a default would be resolved if available.
- Versioned channel - A channel leading with version numbers. Whenever only major.minor are used the most recently used tooolchain with an exact match or latest patch-version would be used.


#### Adding tools to toolchains

Users can add tools to the currently active toolchain by using `huak toolchain add <tool>`. For now the only tools compatible with this feature are tools installed from Python package repositories (registries). To add `my-package` from *MyURL* to an installed 3.12 toolchain:

```
huak toolchain add my-package --registry MyUrl --channel 3.12
```

#### Removing tools from toolchains

Tools can be removed from toolchains. To remove `ruff` from the currently active toolchain:

```
huak toolchain remove ruff
```

#### Toolchain information

Display information about the currently active toolchain by running `huak toolchain info`. To display information about a specific toolchain that's already installed:

```
huak toolchain info --channel <channel>
```

The information displayed about the toolchain includes:

- The channel
- Installed tools and their info
- Managing environments (example: its virtual environment)

#### Listing available toolchains

Users can list toolchains available for Huak to use.

#### Huak as a proxy

Huak will utilize tools from a toolchain without the user having to manage that process. Until better proxy behavior is developed, users can access the currently active toolchain's tools by running `huak toolchain run <tool>`.

#### Without Huak

Toolchains can be used without Huak by activating their virtual environments. If `huak toolchain install 3.12.0 --target ~/desktop` is used a directory containing an installed Python interpreter is available for use. In order to utilize these tools users can activate the virtual environment by running:

```
. ./desktop/cpython-3.12.0-apple-aarch64/.venv/bin/activate
```

#### Home directory

- Env file: Centralized method for updating PATH and environment for Huak usage. Includes adding bin directory to PATH.
- Bin directory: Contains executable programs often used as proxies for other programs for Huak. For the toolchain usage only Huak is added to this directory.
- Settings file: Settings data for Huak is stored in settings.toml. Stores defaults and project-specific configuration including toolchains and eventually Python environments.
- Toolchains directory: Contains Huak's toolchains with the following naming convention {interpreter kind}-{version}-{os}-{architecture}.

#### Pyproject.toml `[tool.huak.toolchain]`

This table is used to configure the toolchain for a project. The channel field is used to indicate the requested version of the toolchain, but eventually channels can include markers unlike version requests.
