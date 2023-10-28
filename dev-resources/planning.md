This document contains structured historical plans for developing Huak.



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
huak toolchain install <version>       # Install a toolchain via its channel version
huak toolchain uninstall <version>     # Uninstall a toolchain via its channel version
huak toolchain use <version>           # Use a toolchain via its channel version
huak toolchain add <tool>              # Add a tool to the current toolchain
huak toolchain remove <tool>           # Remove a tool from the current toolchain
huak toolchain info                    # Display the current toolchain's information
huak toolchain fetch                   # Fetch toolchains available for installation
huak toolchain list                    # Display available toolchains
```


### Concepts

#### Installing toolchains

Until Huak is distributed as a standalone application (not via PyPI) it will continue to behave how it currently does (using available Python environments). When the user runs `huak toolchain install` Huak will either recognize there is already a toolchain installed and available or it will install the latest toolchain available (see [Fetching available toolchains](#fetching-available-toolchains)).

Users can install toolchains using the `huak toolchain install` command.

Without any other arguments `install` will:

- Install the latest toolchain available if Huak doesn't already have one set up.
- Install the toolchain associated with the project's Python environment if it isn't already installed.
  - If a `[huak.toolchain]` table is used in the project's pyproject.toml file it will install it if it isn't already installed.
  - It will attempt to figure out which toolchain to install by checking for a related virtual environment.
  - If no virtual environment is associated with the project it will install a toolchain compatible with the latest version of Python available on the system.
  - If no Python is found on the system it will install the latest available version (see *"Fetching available toolchains"*).

Users can specify versions to install by running `huak toolchain install <version>`.

So if a user wanted to install the default toolchain associated with Python 3.12 the following command would be used:

```
huak toolchain install 3.12
```

`<version>` is the requested version of the Python release associated with the toolchain. If major.minor.patch is used it will attempt to resolve the toolchain for that specific version. If major.minor or just major is used it will resolve for the latest version available associated with the requested version. See *"Using channels"* for more.

Users can install toolchains for their own non-Huak usage by using `--target` (see *"Without Huak"*).

When a toolchain is installed a minimal virtual environment is generated to maintain any Python environment dependent tools installed to the toolchain.

When a toolchain is installed for a specific project Huak is managing then that relationship is added to Huak's settings.toml file (see *"Home directory"*).

#### Updating toolchains

Users can update a toolchain by using the `huak toolchain update` command. By default this command will attempt to resolve the latest versions compatible with the current toolchain. If there is no currently active toolchain this command will silently fail.

Users can update individual tools installed to a toolchain by using `huak toolchain update <tool>`. The following command would update `ruff` for the currently active toolchain:

```
huak toolchain update ruff
```

#### Uninstalling toolchains

Toolchains can be uninstalled using `huak toolchain uninstall`. This will remove the currently active toolchain. Using `huak toolchain uninstall <version>` will attempt to uninstall a toolchain associated with the requested version.

#### Using toolchains

By default Huak will attempt to *use* the latest available toolchain. If one isn't installed it will fetch and install the latest available. Telling Huak to *use* a toolchain will:

- Override the default if a project can't be associated with the command.
- Override the relationship defined Huak's settings.toml if one already exists. If one does not exist it will be added to the settings.toml file.

In order to resolve a toolchain this behavior will follow the same logic defined in *"Installing toolchains"*.

#### Using channels

As mentioned in *"Installing toolchains"*, toolchains have *versions*. Versions are paired with Python release versions. Channels can be further differentiated by information such as the release source or build type, but for now channels available to Huak users will remain the default <major.minor.patch> matching a CPython release.

To use a channel for a pyproject.toml-managed project add the `[huak.toolchain]` table:

```toml
[huak.toolchain]
channel = 3.12
```

See *"Pyproject.toml `[huak.toolchain]`"* for more.

Eventually channels won't be limited to version identifiers.

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
huak toolchain info --channel <version>
```

Users can display information about toolchains that aren't installed by running `huak toolchain fetch <version>` and then running `huak toolchain info <version>`. See *"Fetching available toolchains"* for more.

The information displayed about the toolchain includes:

- The channel
- Installed tools and their info
- Managing environments (example: its virtual environment)

#### Fetching available toolchains

A *fetch* can be used to request information about toolchains that are available to Huak but maybe not currently installed. Toolchains become available to Huak as new Python releases are published. Fetching this data will update Huak's knowledge of what's available.

#### Listing available toolchains

Users can list toolchains available for Huak to use. This does not perform a *fetch* unless `--fetch` is used.

#### Huak as a proxy

Huak will utilize tools from a toolchain without the user having to manage that process. Until better proxy behavior is developed, users can access the currently active toolchain's tools by running `huak toolchain run <tool>`.

#### Without Huak

Toolchains can be used without Huak by activating their virtual environments. If `huak toolchain install 3.12.0 --target ~/desktop` is used a directory containing an installed Python interpreter is available for use. In order to utilize these tools users can activate the virtual environment by running:

```
. ./desktop/huak-cpython-3.12.0-apple-aarch64/.venv/bin/activate
```

#### Home directory

- Env file: Centralized method for updating PATH and environment for Huak usage. Includes adding bin directory to PATH.
- Bin directory: Contains executable programs often used as proxies for other programs for Huak. For the toolchain usage only Huak is added to this directory.
- Settings file: Settings data for Huak is stored in settings.toml. Stores defaults and project-specific configuration including toolchains and eventually Python environments.
- Toolchains directory: Contains Huak's toolchains with the following naming convention huak-{interpreter kind}-{version}-{os}-{architecture}.

#### Pyproject.toml `[huak.toolchain]`

This table is used to configure the toolchain for a project. The channel field is used to indicate the requested version of the toolchain, but eventually channels can include markers unlike version requests.
