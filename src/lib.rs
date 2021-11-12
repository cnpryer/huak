// huak - Project tree hashing play-thing
//
// Command-line program to interact with your project tree hashing.
//
// Hash the current working directory and its sub-structures, then display the resulting hash value.
// ```sh
// huak . --display
// ```
//
// `huak` saves an ignored .huak file containing the latest hash. You can modify the hash with new files:
// ```sh
// huak {file} --display
// ```
//
// Reset the hash to start fresh:
// ```sh
// huak reset
// huak . --display
// ```
//
// Hash files or folders without saving a .huak:
// ```sh
// huak {file} --dry --display
// huak {folder} --dry --display
// ```
//
// TODO: write custom hash map
use std::collections::HashMap;

#[allow(dead_code)]
struct Hash {
    value: String,
}

#[allow(dead_code)]
struct File {
    name: String,
    contents: String,
}

#[allow(dead_code)]
struct Directory {
    name: String,
    files: Vec<File>,
    folders: Vec<Directory>,
}

#[allow(dead_code)]
struct Project<T> {
    path: String,
    contents: HashMap<String, T>,
}

#[allow(dead_code)]
struct CommandRoot {
    value: String,
}

#[allow(dead_code)]
struct CommandArg {
    value: String,
}
