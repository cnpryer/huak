# User Guide

!!! Warning
    `huak` is in an experimental state.

## Contents

1. [Getting started](#getting-started)
1. [Manage your dependencies](#manage-your-dependencies)
1. [Support more of your workflow](#support-more-of-your-workflow)
1. [Distribute your project](#distribute-your-project)
1. [Configure Huak](#configure-huak)

## Getting started

### Installation

Use `pip` to install `huak` from [PyPI](https://pypi.org).

```zsh
~/github 
❯ pip install huak
```

### Create a new project

To create a new project use the `new` command.

```zsh
~/github took 2s 
❯ huak new my-project
```

### Or initialize an existing project

```zsh
~/github/existing-project 
❯ huak init
```

`huak` distinguishes between library and application-like projects. Projects default to the library type if a type isn't specified. Specify the type with either the `--lib` or `--app` flag.

Initializing an existing project adds a `pyproject.toml` to the current directory. Bootstrapping the project with the `new` command creates a Python project with the following structure:

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ tree .
.
├── pyproject.toml
├── src
│   └── my_project
│       └── __init__.py
└── tests
    └── test_version.py
```

!!! note
    Without `--no-vcs` `huak` generates a `git`-initialized project.

## Manage your dependencies

### Add a dependency

Use `huak` to add dependencies to your Python project.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak add xlcsv
```

#### Installer Options

Currently `huak` uses `pip` under the hood for package installation. You can pass additional arguments onto `pip`. Any arguments after `--` are handed off to `pip install`.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak add torch torchvision torchaudio -- --extra-index-url https://download.pytorch.org/whl/cu117
```

`huak` will add the packages to your pyproject.toml, so passing [PEP 508](https://peps.python.org/pep-0508/) strings would help persist this behavior for future installs.

!!! Tip
    You can also assign dependencies to a group using `--group`.

### Install dependencies listed in the pyproject.toml

Use the `install` command to install the project's dependencies.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak install
```

#### Using --groups

To install just "required" dependencies:

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak install --groups required
```

If you already have an optional dependency group named "required" then `--groups` will operate as normal and only install the groups provided.

### Update dependencies

To update a dependency use the `update` command.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak update xlcsv
```

Or everything.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak update
```

### Remove dependencies

To remove a dependency from the project use the `remove` command.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak remove xlcsv
```

## Support more of your workflow

Huak ships commands allowing you to format your python code, lint it, and test it.

### Format your code

Use the `fmt` command to format your Python project's code.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak fmt
```

#### Using --check

Use `--check` if all you want to do is verify your code is already formatted. Note that `huak` currently uses a combination of `black` and `ruff` to format your code. This means that `--` can only pass options to `black`. Use the `[tool.ruff]` approach to configure import sorting.

!!! Warning
    `huak` will exit prior to running the `black` *check* if your imports are not sorted. See [#510](https://github.com/cnpryer/huak/issues/510) for the status of this issue.

### Lint your code

Use the `lint` command to lint your Python project's code.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 took 2s 
❯ huak lint
```

The `--fix` flag can be used to address any auto-fixable issues.

`huak` wraps tools like `ruff` for some of its commands. To configure a wrapped tool such as `ruff` use the pyproject.toml file:

```toml
[tool.ruff]
# ...
```

`huak` also uses `mypy` for type-checking. To disable this behavior use `--no-types`.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 took 2s 
❯ huak lint --no-types
```

!!! Note
    Currently, since `ruff` is the default tool used by `huak lint`, passing additional options with `--` is reserved for `ruff`. To configure `mypy` use the `[tool.mypy]` approach. This limitation will be addressed in future versions of `huak` (see [#505](https://github.com/cnpryer/huak/issues/505)).

### Test your code

Use the `test` command to test your project.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0
❯ huak test
```

### Run commands within your project's environment context

You can use `huak` to run a command within the Python environment your project uses.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak run which python
/Users/chrispryer/github/my-project/.venv/bin/python
```

### Activate the virtual environment

`huak` also implements an `activate` command to activate a virtual environment.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak activate

(.venv) bash-3.2$
```

!!! Note
    Currently `huak` just runs `bash --init-file` on unix systems and uses `powershell` on Windows.
    ```powershell
    (.venv) PS C:\Users\chris\github\my-project>
    ```
    See [#486](https://github.com/cnpryer/huak/issues/486) for the status on the `activate` command.

## Manage your Python installations

By default `huak` will use the first Python interpreter found from your `PATH` environment variable. You can `list` these by using the `python` command.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak python list
1: /Users/chrispryer/.pyenv/shims/python3.11
2: /Users/chrispryer/.pyenv/shims/python3.10
...
```

So `huak` would use `/Users/chrispryer/.pyenv/shims/python3.11` out of the box. You can use a different installed Python version with `use`.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak python use 3.10
```

## Distribute your project

### Publish to PyPI

If you're building a Python package you'd like to share, use `huak build` and `huak publish` to build and publish the project to [PyPI](https://pypi.org).

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak build

my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak publish
```

### Cleaning up

Use `huak clean` to clean out the dist/ directory.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 took 26s 
❯ huak clean
```

## Configure Huak

### Configure shell completion

With `huak completion` you can setup shell completion for `huak`.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak completion -h
Generates a shell completion script for supported shells. See the help menu for more information on supported shells

Usage: huak completion [OPTIONS]

Options:
  -s, --shell <shell>  [possible values: bash, elvish, fish, powershell, zsh]
  -i, --install        Installs the completion script in your shell init file. If this flag is passed the --shell is required
  -u, --uninstall      Uninstalls the completion script from your shell init file. If this flag is passed the --shell is required
  -h, --help           Print help
```

### Install shell completion in zsh
The recommended way to install shell completions with zsh shell will look like this (add this to your .zshrc file):
```zsh
eval "$(huak completion --shell zsh)"
```
It's possible you might need to add
```zsh
autoload -Uz compinit
compinit
```
to your .zshrc file before that. Verify it by running the first command in your shell. If it succeeds, your are good to go, otherwise, if you are seeing an. error related to `comp`, try the autload command.

## Providing feedback

Any bugs or suggestions can be submitted as issues [here](https://github.com/cnpryer/huak/issues/new). All feedback is welcome and greatly appreciated ❤️.

```zsh
my-project on master 📦 v0.0.1 via 🐍 v3.11.0 
❯ huak --version
huak 0.0.19
```

*Use the [0.0.16 User Guide](https://github.com/cnpryer/huak/blob/9b2bee0ac3425898394b3d9273105f554c45a744/docs/user_guide.md) for `huak` versions prior to `v0.0.19`.*
