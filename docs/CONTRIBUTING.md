# Contributing

We appreciate your interest in contributing to our project!

You can contribute to Huak by checking for unassigned issues or even just documenting features and bugs by creating new issues.

## Communication

You can reach out to me on discord in our [discord server](https://discord.gg/St3menxFZT) if you have any questions. Otherwise, please feel free to [start a discussion](https://github.com/cnpryer/huak/discussions/new) or [open an issue](https://github.com/cnpryer/huak/issues/new/).

## Testing

Use cargo to run the tests.

```zsh
❯ cargo test
```

Note that since we are dedicating a `.venv` to testing Huak, you should expect to have a `.venv` exist in your local project. Huak needs a Python environment (specifically a venv) to run its commands from the context of. #123 will resolve this.

## Making a contribution

We follow the standard [GitHub flow](https://docs.github.com/en/get-started/quickstart/github-flow) when making contributions to Huak. Digital Ocean published another great resource for [getting started with open source](https://www.digitalocean.com/community/tutorial_series/an-introduction-to-open-source).

Please make sure your PR is associated with an issue. Create one [here](https://github.com/cnpryer/huak/issues/new).

1. After being assigned to an issue or communicating your interest, fork the repository and get started by creating a new branch for your work.
2. When you're ready to create your PR, feel free to use our PULL_REQUEST_TEMPLATE.md to indicate what issue your PR is closing and the changes made to close it.

We also use `pre-commit` to ensure changes will pass CI from our local environments. We recommend using it.

To use `pre-commit` [install it](https://pre-commit.com/#install) and run `pre-commit install` from the project directory's root.

You'll notice the main checks are:

- `cargo fmt --check`
- `cargo clippy`
- `cargo test`

## Commits

✨ Please practice clean commit hygiene.

TODO: See [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) for more.
