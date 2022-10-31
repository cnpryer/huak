//!## About
//!A Python package manager written in Rust. The [Cargo](https://github.com/rust-lang/cargo) for Python.
//!
//!> ⚠️ Disclaimer: `huak` is currently in its [Alpha phase](https://github.com/cnpryer/huak/milestones).
//!
//!Huak aims to support a base workflow for developing Python packages and projects. The process is linear and purpose oriented, creating better familiarization with the steps.
//!
//!The goal is to create an opinionated tool to support a reliably inviting onboarding experience for the Python ecosystem, that feels responsive and snappy to use.
//!
//!### Milestones and Project Board
//!
//!See either **[this milestone list](https://github.com/cnpryer/huak/milestones)** or **[the issue board](https://github.com/users/cnpryer/projects/5)** to check the status of this project at any point in time.
//!
//!## Contents
//!
//!- [Installation](#installation)
//!- [Goals and Motivation](#goals)
//!- [Architecture and Design](#architecture-and-design)
//!- [Contributing](#contributing)
//!
//!## Installation
//!
//!A PoC, Alpha releases, and an 0.1.0 are expected.
//!
//!During the [Alpha phase](https://github.com/cnpryer/huak/milestones) you'll need to explicitly install the latest pre-release available.
//!
//!`❯ cargo install huak --version` [![crates.io](https://img.shields.io/crates/v/huak.svg?label="")](https://crates.io/crates/huak)
//!
//!Around 0.1.0 you'll be able to install `huak` using `brew` or `pip`. Distribution plans will be finalized closer to 0.1.0.
//!
//!## Shell completion
//!
//!##### Bash
//!First, ensure that you install `bash-completion` using your package manager.
//!After, add this to your `~/.bash_profile`:
//!
//!```bash
//! eval "$(huak config completion bash)"
//!```
//!
//!##### Elvish
//!This shell is supported, but the suggestion as to how this should be added to your shell init file is missing.
//!
//!If you are able to test this please head over to [github](https://github.com/cnpryer/huak/issues) and file an issue

//!##### Fish
//!Generate a `huak.fish` completion script:
//!
//!```bash
//! huak config completion fish > ~/.config/fish/completions/huak.fish
//!```
//!
//!##### Powershell
//!Open your profile script with:
//!
//!```bash
//! mkdir -Path (Split-Path -Parent $profile) -ErrorAction SilentlyContinue
//!```
//!
//!```bash
//! notepad $profile
//!```
//!
//!Add the line and save the file:
//!```bash
//! Invoke-Expression -Command $(huak config completion powershell| Out-String)
//!```
//!
//!##### Zsh
//!Generate a `_huak` completion script and put it somewhere in your `$fpath`:
//!```bash
//! huak config completion zsh > /usr/local/share/zsh/site-functions/_huak
//!```
//!
//!Ensure that the following is present in your `~/.zshrc`:
//!
//!```bash
//! autoload -U compinit
//!```
//!
//!```bash
//! compinit -i
//!```
//!
//!
//!## Goals
//!
//!Besides some of my own experience with the Python ecosystem, there are a few additional guiding principles steering the development of Huak:
//!
//!### 1. Open to open source 📚
//!
//!Open source has done a lot for me both from a tooling and professional development perspective. I'd love to offer Huak as a way to help onboard the absolute and relative newcomers (like myself).
//!
//!### 2. Just use `huak` ✨
//!
//!I love Rust's onboarding experience. Cargo has played a large role. It's a great tool for newcomers to use to get their feet wet. Huak can provide the same experience for Python.
//!
//!### 3. Fast ⚡️
//!
//!There's room for faster tooling in the Python ecosystem. One of the guiding principles will be "Is this the fastest it can be?"
//!
//!### 4. Python 🤝 Rust
//!
//!JavaScript has seen a "Going Rust" sub-community pop up. Python seems to be getting one too. Huak would be able to fuel contributions to the intersection of these two languages.
//!
//!## Contributing
//!
//!Please read our [contributing guide](/docs/CONTRIBUTING.md) before you start contributing.
//!
//!## Architecture and Design
//!
//!
//!This section is constantly changing while Huak is fresh.
//!
//!As I become more comfortable with Rust and knowledgeable of the packaging domain, I'll flesh out the general design for Huak more concretely. Until then, I want to leave its design open to influence.
//!
//!- [Design](#design)
//!  - [Project Workflows](#linear-project-workflows)
//!  - Huak's Design
//! - [The Code](#the-code)
//!
//! ### Design
//!
//! Currently, this only covers high level influence for design of the project.
//!
//! #### Linear project workflows
//!
//! Huak enables and supports a standard *process of developing*. This process is linear. Iteration happens in sequential steps.
//!
//! ```mermaid
//! graph LR
//!     A[Project Bootstrap] --> B[Project Setup]
//!     B --> C[Project Change]
//!     C --> D[Project Test]
//!     D --> E[Project Distribution]
//! ```
//!
//! ##### 1. Project Bootstrap
//!
//! Quick and easy initialization of a project with opinions on topics like structure and configuration.
//!
//! ##### 2. Project Setup
//!
//! Adding dependencies, various metadata, etc. The setup phase is vague but prepares the project for the following steps.
//!
//! ##### 3. Project Change
//!
//! A change is made to the project.
//!
//! ##### 3. Project Test
//!
//! The project is evaluated in some form.
//!
//! ##### 4. Project Distribution
//!
//! The project is distributed for use. This can be publishing to a registry or simply using it locally and executing within its context.
//!
//! ### The Code
//!
//! Currently, the project is structured using the following components:
//!
//! ```txt
//! src
//! ├── bin           # CLI binary `huak`
//! │   ├── commands  # Application subcommand layer
//! │   │   └── ...
//! │   └── main      # Main application
//! └── huak          # Huak's library
//!     ├── config    # Configuration formats
//!     ├── env       # Environment contexts
//!     ├── ops       # Huak operation implementation layer
//!     ├── package   # Packaging logic
//!     ├── project   # The `Project` struct
//!     └── utils     # Library utilities
//! ```
/// Configuration formats for structures and contexts.
pub mod config;
/// Environments for different contexts.
pub mod env;
/// CLI and generic errors.
pub mod errors;
/// Operations for projects, packaging, and environments.
pub mod ops;
/// Packaging namespace for the Huak application.
pub mod package;
/// The Project implementation.
pub mod project;
/// Huak utilities library.
pub mod utils;
