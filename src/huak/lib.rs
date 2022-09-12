//! A Python package manager written in Rust. The [Cargo](https://github.com/rust-lang/cargo) for Python.
//!
//! See either [this milestone list](https://github.com/cnpryer/huak/milestones) or [the issue board](https://github.com/users/cnpryer/projects/5) to check the status of this project at any point in time.
//!
//! - [Goals and Motivation](#goals)
//! - [Contributing](#contributing)
//! - [Architecture and Design](#architecture-and-design)
//!
//! ## Goals
//!
//! There are a few guiding principles steering the development of Huak:
//!
//! ### 1. Serve as a learning instrument 📚
//!
//! - Python packaging is a very interesting topic right now.
//! - It's challenging and dev tools are some of my favorite products, I'm curious of how they work.
//! - Learn Rust and about building fast, snappy, and opinionated software.
//!
//! ### 2. Just use `huak` ✨
//!
//! I love Rust's onboarding experience. Cargo has played a large role. It's a great tool for newcomers to use to get their feet wet. Huak can provide the same experience for Python.
//!
//! ### 3. Fast ⚡️
//!
//! There's room for faster tooling in the Python ecosystem. One of the guiding principles will be "Is this the fastest it can be?"
//!
//! ### 4. Python 🤝 Rust
//!
//! JavaScript has seen a "Going Rust" sub-community pop up. Python seems to be getting one too. Huak would be able to fuel contributions to the intersection of these two languages.
//!
//! ## Contributing
//!
//! Please read our [contributing guide](./CONTRIBUTING.md) before you start contributing.
//!
//! ## Architecture and Design
//!
//! See [architecture.md](./architecture.md).
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
