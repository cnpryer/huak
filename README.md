[![ci-rust](https://github.com/cnpryer/huak/actions/workflows/ci-rust.yaml/badge.svg)](https://github.com/cnpryer/huak/actions/workflows/ci-rust.yaml)
[![ci-python](https://github.com/cnpryer/huak/actions/workflows/ci-python.yaml/badge.svg)](https://github.com/cnpryer/huak/actions/workflows/ci-python.yaml)

# huak

<div align="center">

<img src="docs/assets/img/logo.png" alt="Huak logo" width="300" role="img">

</div>

</br>

## About

A Python package manager written in Rust. The [Cargo](https://github.com/rust-lang/cargo) for Python.

### Milestones and Project Board

See either **[this milestone list](https://github.com/cnpryer/huak/milestones)** or **[the issue board](https://github.com/users/cnpryer/projects/5)** to check the status of this project at any point in time.

## README Contents

- [A Package Manager](#a-package-manager)
- [Try it out](#try-huak)
- [Documentation](#documentation)
- [Goals and Motivation](#goals)
- [Contributing](#contributing)
- [Architecture and Design](#architecture-and-design)

## A Package Manager

Huak aims to be a snappy package manager for Python, supporting a standard base workflow with bootstrapping, configuration, dependency management, building and publishing tools. This package manager can be used for Python packages and Python projects.

Onboarding and speed are focal points for Huak.

## Try Huak

A Huak PoC will be developed prior to 0.1.0. See [milestones](https://github.com/cnpryer/huak/milestones) for the status on those.

In each phase you'll be able to:

- PoC: Clone the repository and build the release with `cargo build --release`
- 0.1.0: Install with `cargo install huak`
- Future plans include
  - `brew install huak`
  - `pip install huak`
  - GitHub release artifacts
  - ...

```console
$ huak help

A Python package manager written in Rust inspired by Cargo

USAGE:
    huak [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    activate         Activate the project's virtual environment.*
    add              Add a Python module to the existing project.*
    build            Build tarball and wheel for the project.*
    clean            Remove tarball and wheel from the built project.
    clean-pycache    Remove all .pyc files and __pycache__ directories.
    fmt              Format Python code.
    help             Display Huak commands and general usage information.
    init             Initialize the existing project.
    install          Install the dependencies of an existing project.
    lint             Lint Python code.
    new              Create a project from scratch.
    remove           Remove a dependency from the project.
    run              Run a command within the project's environment context.*
    test             Test Python code.
    update           Update dependencies added to the project.*
    version          Display the version of the project.
```
_"*" indicates incomplete first-pass but planned for PoC_

## Documentation

Some documentation for 0.0.x releases will land on docs.rs. Around the 0.1.0 release I'd like to have more robust documentation about Huak for users, contributors, and the curious.

## Goals

Besides some of my own experience with the Python ecosystem, there are a few additional guiding principles steering the development of Huak:

### 1. Open to open source 📚

Open source has done a lot for me both from a tooling and professional development perspective. I'd love to offer Huak as a way to help onboard the absolute and relative newcomers (like myself).

### 2. Just use `huak` ✨

I love Rust's onboarding experience. Cargo has played a large role. It's a great tool for newcomers to use to get their feet wet. Huak can provide the same experience for Python.

### 3. Fast ⚡️

There's room for faster tooling in the Python ecosystem. One of the guiding principles will be "Is this the fastest it can be?"

### 4. Python 🤝 Rust

JavaScript has seen a "Going Rust" sub-community pop up. Python seems to be getting one too. Huak would be able to fuel contributions to the intersection of these two languages.

## Contributing

Please read our [contributing guide](./CONTRIBUTING.md) before you start contributing.

## Architecture and Design

See [architecture.md](./architecture.md).
