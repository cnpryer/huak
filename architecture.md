# Architecture

This document is constantly changing while Huak is fresh.

At the moment this page covers code structure rather than generally the software's design. As I become more comfortable with Rust and knowledgeable of the packaging domain, I'll flesh out the general design for Huak more concretely. Until then I want to leave it's design open to influence.

## The Code

Currently the project is structured using the following components:

- A CLI binary (The Huak *Application*)
- Huak's library
  - `configuration` formats
  - `environment`s for contexts
  - `project`s for operation
  - `packaging` for all packaging needs


## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).
