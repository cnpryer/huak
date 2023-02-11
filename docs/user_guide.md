# User Guide

## Contents

1. [Getting started](#getting-started)
1. [Manage your dependencies](#manage-your-dependencies)
1. [Support more of your workflow](#support-more-of-your-workflow)
1. [Distribute your project](#distribute-your-project)
1. [Configure Huak](#configure-huak)

## Getting started

### Installation

Use `pip` to install `huak` from [PyPI](https://pypi.org).

```
~/github 
❯ pip install huak
```

### Create a new project

To create a new project use the `new` command.

```
~/github took 2s 
❯ huak new my-project
```

### Or initialize an existing project

```
~/github/existing-project 
❯ huak init
```

`huak` distinguishes between library and application-like projects. Projects default to the library type if a type isn't specified. Specify the type with either the `--lib` or `--app` flag.

Initializing an existing project adds a `pyproject.toml` to the current directory. Bootstrapping the project with the `new` command creates a Python project with the following structure:

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ tree .
.
├── pyproject.toml
├── src
│   └── my_project
│       └── __init__.py
└── tests
    └── test_version.py
```

Note that without `--no-vcs` `huak` generates a `git`-initialized project.

## Manage your dependencies

### Add a dependency

Use `huak` to add dependencies to your Python project.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak add xlcsv
```

You can assign the dependency to a group using `--group`.

### Install dependencies listed in the pyproject.toml

Use the `install` command to install all of the project's dependencies.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak install
```

With `--groups` you can install dependencies from specific groups.

### Remove dependencies

To remove a dependency from the project use the `remove` command.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak remove xlcsv
```

## Support more of your workflow

Huak ships commands allowing you to format your python code, lint it, and test it.

### Format your code

Use the `fmt` command to format your Python project's code.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak fmt
```

Use `--check` if all you want to do is verify your code is already formatted.

### Lint your code

Use the `lint` command to lint your Python project's code.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 took 2s 
❯ huak lint
```

The `--fix` flag can be used to address any auto-fixable issues.

`huak` wraps tools like `ruff` for some of its commands. To configure a wrapped tool such as `ruff` use the pyproject.toml file:

```toml
[tool.ruff]
# ...
```

Some wrapped tools' features may not be accessible using this method. See [#467](https://github.com/cnpryer/huak/issues/467) for the status on this issue.

### Test your code

Use the `test` command to test your project.

> ⚠️ Currently your Python project won't be recognized by the interpreter without installing it to the environment. See [#459](https://github.com/cnpryer/huak/issues/459).

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0
❯ huak run pip install -e .
```

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0
❯ huak test
```

### Run commands within your project's environment context

You can use `huak` to run a command within the Python environment your project uses.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak run which python
/Users/chrispryer/github/my-project/.venv/bin/python
```

### Activate your project's virtual environment

Use the `activate` command to launch a shell with your project's virtual environment activated.

> ⚠️ Note that `huak activate` is currently only partially supported on Windows. See [#302](https://github.com/cnpryer/huak/issues/302).

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak activate
```

This feature is currently limited and at times buggy. It's recommended to use `huak run` until `activate` stabilizes.

To deactivate the virtual environment run `deactivate`.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 (.venv) 
❯ deactivate
```

## Distribute your project

### Publish to PyPI

If you're building a Python package you'd like to share, use `huak publish` to build and publish the project to [PyPI](https://pypi.org).

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak publish
```

If you just want to build the project use the `build` command.

### Cleaning up

Use `huak clean` to clean up the dist/ directory.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 took 26s 
❯ huak clean
```

## Configure Huak

Use `huak config` commands to configure `huak`.

### Configure shell completion

With `huak config completion` you can setup shell completion for `huak`.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak config completion -h
Generates a shell completion script for supported shells. See the help menu for more information on supported shells

Usage: huak config completion [OPTIONS]

Options:
  -s, --shell <shell>  [possible values: bash, elvish, fish, powershell, zsh]
  -i, --install        Installs the completion script in your shell init file. If this flag is passed the --shell is required
  -u, --uninstall      Uninstalls the completion script from your shell init file. If this flag is passed the --shell is required
  -h, --help           Print help
```

## Providing feedback

Any bugs or suggestions can be submitted as issues [here](https://github.com/cnpryer/huak/issues/new). All feedback is welcome and greatly appreciated ❤️.

```
my-project on  master [?] is 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak --version
huak 0.0.10-alpha.6
```