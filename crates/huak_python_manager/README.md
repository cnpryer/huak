# Python Manager

A Python interpreter management system for Huak.

## Usage

```
huak_python_manager install 3.11
```

## How it works

### Installing a Python interpreter

1. Fetch the interpreter from https://github.com/indygreg/python-build-standalone using GitHub API.
1. Validate the checksum of the interpreter.
1. Extract the interpreter using `tar`.
1. Place the interpreter in Huak's home directory (~/.huak/bin/).
