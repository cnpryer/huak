
[![pypi-release](https://img.shields.io/pypi/v/huak.svg)](https://pypi.org/project/huak/)
[![ci-rust](https://github.com/cnpryer/huak/actions/workflows/ci-rust.yaml/badge.svg)](https://github.com/cnpryer/huak/actions/workflows/ci-rust.yaml)
[![discord](https://img.shields.io/discord/1022879330470199347?color=7289DA&logo=discord)](https://discord.gg/St3menxFZT)

# huak

<div align="center">

<img src="https://raw.githubusercontent.com/cnpryer/huak/master/docs/assets/img/logo.png" alt="Huak logo" width="300" role="img"/>

</div>

<br>

## About

A Python package manager written in Rust. The [Cargo](https://github.com/rust-lang/cargo) for Python.

> ⚠️ Disclaimer: `huak` is in an experimental state.

Huak aims to support a base workflow for developing Python packages and projects. The process is linear and purpose oriented, establishing better familiarization with the steps.

The goal is to create an opinionated tool to support a reliably inviting onboarding experience for the Python ecosystem, that feels responsive and snappy to use.

## README Contents

- [Installation](#installation)
- [Usage](#usage)
- [Documentation](#documentation)
- [Goals and Motivation](#goals)
- [Contributing](#contributing)

## Installation

```
❯ pip install huak
```

## Usage

```console
❯ huak help

A Python package manager written in Rust inspired by Cargo.

Usage: huak [OPTIONS] <COMMAND>

Commands:
  activate    Activate the virtual envionrment
  add         Add dependencies to the project
  build       Build tarball and wheel for the project
  completion  Generates a shell completion script for supported shells
  clean       Remove tarball and wheel from the built project
  fix         Auto-fix fixable lint conflicts
  fmt         Format the project's Python code
  init        Initialize the existing project
  install     Install the dependencies of an existing project
  lint        Lint the project's Python code
  new         Create a new project at <path>
  publish     Builds and uploads current project to a registry
  python      Manage python installations
  remove      Remove dependencies from the project
  run         Run a command within the project's environment context
  test        Test the project's Python code
  update      Update the project's dependencies
  version     Display the version of the project
  help        Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet    
  -h, --help     Print help
  -V, --version  Print version
```

## Documentation

- [User Guide](./docs/user_guide.md)
- [Development](/docs/development.md)

## Goals

### 1. Just use `huak` ✨

The Rust ecosystem has a fantastic onboarding experience. Cargo plays a large role. Huak can provide the same experience for Python.

### 2. Fast ⚡️

There's room for faster tooling in the Python ecosystem. One of the guiding principles will be *"Is this the fastest it can be?"*

### 3. Python 🤝 Rust

JavaScript has seen a "Going Rust" sub-community pop up. Huak is positioned well to help sustain future development of Rust-based software for the Python ecosystem.

## Contributing

Please read our [contributing guide](/docs/CONTRIBUTING.md) before you start contributing.
