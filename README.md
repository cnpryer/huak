
<!-- [![pypi-release](https://img.shields.io/pypi/v/huak.svg)](https://pypi.org/project/huak/) -->
[![ci](https://github.com/cnpryer/huak/actions/workflows/ci.yaml/badge.svg)](https://github.com/cnpryer/huak/actions/workflows/ci.yaml)

# huak

<div align="center">

<img src="https://raw.githubusercontent.com/cnpryer/huak/master/docs/assets/img/logo.png" alt="Huak logo" width="300" role="img"/>

</div>

<br>

## About

A Python package manager written in Rust. The [Cargo](https://github.com/rust-lang/cargo) for Python.

> ⚠️ Disclaimer: `huak` is in an experimental state (see [#602](https://github.com/cnpryer/huak/issues/602)).

Huak ("hwok") aims to support a base workflow for developing Python packages and projects. The process is linear and purpose oriented, establishing better familiarization with the steps.

The goal is to create an opinionated tool to support a reliably inviting onboarding experience for the Python ecosystem, that feels responsive and snappy to use.

## README Contents

- [Installation](#installation)
- [Usage](#usage)
- [Documentation](https://cnpryer.github.io/huak/user_guide/)
- [Goals and Motivation](#goals)
- [Contributing](#contributing)

## Installation


To install Huak from source using Cargo:

```
cargo install --git https://github.com/cnpryer/huak.git huak
```

> ⚠️ WARNING: The PyPI distribution is outdated.

## Usage

```console
A Python package manager written in Rust and inspired by Cargo.

Usage: huak [OPTIONS] <COMMAND>

Commands:
  activate    Activate the virtual environment
  add         Add dependencies to the project
  build       Build tarball and wheel for the project
  clean       Remove tarball and wheel from the built project
  completion  Generates a shell completion script for supported shells
  fix         Auto-fix fixable lint conflicts
  fmt         Format the project's Python code
  init        Initialize the current project
  install     Install a Python package (defaults to $HOME/.huak/bin)
  lint        Lint the project's Python code
  new         Create a new project at <path>
  publish     Builds and uploads current project to a registry
  python      Manage Python installations
  remove      Remove dependencies from the project
  run         Run a command with Huak
  test        Test the project's Python code
  toolchain   Manage toolchains
  update      Update the project's dependencies
  version     Display the version of the project
  help        Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet     
      --no-color  
  -h, --help      Print help
  -V, --version   Print version
```

## Goals

### 1. Just use `huak` ✨

The Rust ecosystem has a fantastic onboarding experience. Cargo plays a large role. Huak can provide the same experience for Python.

### 2. Fast ⚡️

There's room for faster tooling in the Python ecosystem. One of the guiding principles will be *"Is this the fastest it can be?"*

### 3. Python 🤝 Rust

JavaScript has seen a "Going Rust" sub-community pop up. Huak is positioned well to help sustain future development of Rust-based software for the Python ecosystem.

## Contributing

Please read our [contributing guide](/docs/CONTRIBUTING.md) before you start contributing.
