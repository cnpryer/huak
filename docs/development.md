# Contents

- [Contributing](#contributing)
- [Project Workflows](#project-workflows)
- [The Code](#the-code)

## Contributing

See [CONTRIBUTING.md](/docs/CONTRIBUTING.md).

## Project workflows

Huak enables and supports a standard *process of developing*. This process is linear. Iteration happens in sequential steps.

```mermaid
graph LR
    A[Project Bootstrap] --> B[Project Setup]
    B --> C[Project Change]
    C --> D[Project Test]
    D --> E[Project Distribution]
```

### 1. Project Bootstrap

Quick and easy initialization of a project with opinions on topics like structure and configuration.

### 2. Project Setup

Adding dependencies, various metadata, etc. The setup phase is vague but prepares the project for the following steps.

### 3. Project Change

A change is made to the project.

### 3. Project Test

The project is evaluated in some form.

### 4. Project Distribution

The project is distributed for use. This can be publishing to a registry or simply using it locally and executing within its context.

## The Code

Currently, the project is structured using the following crates:

```bash
crates
├── `huak_cli`  # The `huak` application
└── `huak_python_package_manager`  # `huak`'s operations library
```
