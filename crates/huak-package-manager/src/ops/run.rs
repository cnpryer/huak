use super::add_venv_to_command;
use crate::{shell_name, sys::Terminal, Config, Error, HuakResult};
use huak_pyproject_toml::{sanitize_str, value_to_sanitized_string};
use std::{collections::HashMap, env::consts::OS, ffi::OsStr, ops::Deref, process::Command};
use termcolor::Color;
use toml_edit::{Array, ArrayOfTables, Formatted, InlineTable, Item, Table, Value};

pub fn run_command_str(content: &str, config: &Config) -> HuakResult<()> {
    let ws = config.workspace();
    let manifest = ws.current_local_manifest()?;

    // Get any run commands listed in [tool.huak.run]
    let task_table = manifest
        .manifest_data()
        .tool_table()
        .and_then(|it| it.get("huak"))
        .and_then(Item::as_table)
        .and_then(|it| it.get("task"))
        .and_then(Item::as_table);

    let trimmed = content.trim();

    let trimmed = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    };

    // If there is a task table and there's no program provided just print any available commands
    // from the task table.
    // If there is a task table and the program is found in the task table then attempt to run
    // the command with Huak by building a command from the contents provided.
    if let Some(table) = task_table {
        if trimmed.map_or(true, str::is_empty) {
            return print_task_table(&mut config.terminal(), table);
        };

        // Try to get the program from the content provided.
        let maybe_task = trimmed.as_ref().and_then(|it| it.split(' ').next());

        // If the program is in the task table then run the command from the task table.
        if maybe_task.map_or(false, |name| {
            task_table.map_or(false, |table| table.contains_key(name))
        }) {
            let table = task_table.expect("task table");
            let task = maybe_task.expect("task name");
            return TaskRunner::from_table(table.to_owned()).run(task, config);
        }
    }

    // If a program is found or the contents still contain something to parse/run
    // attempt to run the contents using the shell.
    if let Some(s) = trimmed.filter(|it| !it.is_empty()) {
        run_str(s, config)
    } else {
        Err(Error::InvalidProgram(
            "could not resolve program".to_string(),
        ))
    }
}

fn print_task_table(terminal: &mut Terminal, table: &Table) -> HuakResult<()> {
    let commands = table
        .get_values()
        .into_iter()
        .flat_map(|(k, _)| k)
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    terminal.print_custom("Tasks", "", Color::Cyan, true)?;

    for (i, command) in commands.iter().enumerate() {
        terminal.print_custom(
            format!("{:>5})", i + 1),
            format!("{command:<16}"),
            Color::Green,
            true,
        )?;
    }

    Ok(())
}

struct TaskRunner {
    table: Table,
}

impl TaskRunner {
    fn from_table(table: Table) -> Self {
        Self { table }
    }

    fn get(&self, name: &str) -> Option<&Item> {
        self.table.get(name)
    }

    fn run(&self, name: &str, config: &Config) -> HuakResult<()> {
        match self.get(name) {
            None | Some(Item::None) => Err(Error::InvalidProgram(name.to_string())),
            Some(Item::Value(value)) => run_value_task(self, value, config),
            Some(Item::Table(table)) => run_table_task(self, table, config),
            Some(Item::ArrayOfTables(array)) => run_array_of_tables_task(self, array, config),
        }
    }
}

/// A task configured with a `Value` can include:
///
/// - Strings
/// - Arrays
/// - Inline Tables
///
/// ```toml
/// [tool.huak.task]
/// task1 = "this is a command"  # ('this' is the program)
/// task2 = ["these", "are", "command", "arguments"]  # ('these' is the program)
/// task3 = [
///     { cmd = "this is a command", env = {KEY = "value"} },  # ('this' is the program)
///     { cmd = ["these", "are", "command", "arguments"] },  # ('these' is the program)
///     { chain = ["task1", "task2", "task3"] },
/// ]
/// task4 = { program = "thing", args = ["some", "args"], env = {KEY = "value"} }  # ('this' is the program)
/// task5 = { cmd = "this is a command", env = { KEY = "value" } }  # ('this' is the program)
/// task6 = { chain = ["task1", "task2", "task3" }
/// ```
fn run_value_task(runner: &TaskRunner, value: &Value, config: &Config) -> HuakResult<()> {
    match value {
        Value::String(string) => run_formatted_string_task(runner, string, config),
        Value::Array(array) => run_array_task(runner, array, config),
        Value::InlineTable(table) => run_inline_table_task(runner, table, config),
        _ => Err(Error::InvalidProgram(format!("{value}"))),
    }
}

/// ```toml
/// # [tool.huak.task]
/// # task0 = "some command"
///
/// [[tool.huak.task."task1"]]
/// task1-1 = "this is a command"  # ('this' is the program)
/// task1-2 = ["these", "are", "command", "arguments"]  # ('these' is the program)
/// task1-3 = [
///     { cmd = "this is a command", env = {KEY = "value"} },  # ('this' is the program)
///     { cmd = ["these", "are", "command", "arguments"] },  # ('these' is the program)
///     { chain = ["task1", "task2", "task3"] },
/// ]
/// ```
fn run_array_of_tables_task(
    _runner: &TaskRunner,
    _array: &ArrayOfTables,
    _config: &Config,
) -> HuakResult<()> {
    todo!()
}

/// ```toml
/// [tool.huak.task."task1"]
/// chain = [
///     { cmd = "this is a command", env = {KEY = "value"} },  # ('this' is the program)
///     { cmd = ["these", "are", "command", "arguments"] },  # ('these' is the program)
///     { chain = ["task1", "task2", "task3" },
/// ]
/// ```
fn run_table_task(runner: &TaskRunner, table: &Table, config: &Config) -> HuakResult<()> {
    let env = table.get("env");
    let program = table.get("program");
    let args = table.get("args");
    let cmd = table.get("cmd");
    let chain = table.get("chain");

    // Run the task with configuration data. If no configuration data is provided expect the
    // table to contain sub-tasks (TODO(cnpryer)).
    if chain.is_some() || (program.is_some() || args.is_some() || cmd.is_some()) {
        run_table_task_inner(runner, program, args, cmd, chain, env, config)
    } else {
        todo!()
    }
}

/// ```toml
/// [tool.huak.task]
/// task1 = { cmd = "this is a command", env = {KEY = "value"} }  # ('this' is the program)
/// task2 = { cmd = ["these", "are", "command", "arguments"] }  # ('these' is the program)
/// task3 = { chain = ["task1", "task2", "task3"] }
/// ```
fn run_inline_table_task(
    runner: &TaskRunner,
    table: &InlineTable,
    config: &Config,
) -> HuakResult<()> {
    // TODO(cnpryer): Perf
    let program = table
        .get("program")
        .map(|value| Item::Value(value.to_owned()));
    let args = table.get("args").map(|value| Item::Value(value.to_owned()));
    let cmd = table.get("cmd").map(|value| Item::Value(value.to_owned()));
    let chain = table
        .get("chain")
        .map(|value| Item::Value(value.to_owned()));
    let env = table.get("env").map(|value| Item::Value(value.to_owned()));

    // Run the task with configuration data. If no configuration data is provided expect the
    // table to contain sub-tasks (TODO(cnpryer)).
    if chain.is_some() || (program.is_some() || args.is_some() || cmd.is_some()) {
        run_table_task_inner(
            runner,
            program.as_ref(),
            args.as_ref(),
            cmd.as_ref(),
            chain.as_ref(),
            env.as_ref(),
            config,
        )
    } else {
        todo!()
    }
}

fn run_table_task_inner(
    runner: &TaskRunner,
    program: Option<&Item>,
    args: Option<&Item>,
    cmd: Option<&Item>,
    chain: Option<&Item>,
    env: Option<&Item>,
    config: &Config,
) -> HuakResult<()> {
    if cmd.is_some() && (args.is_some() || program.is_some()) {
        return Err(Error::InvalidRunCommand(
            "'cmd' cannot be used with 'args' or 'program'".to_string(),
        ));
    }

    // TODO(cnpryer): Configuration errors
    let program = program.and_then(Item::as_str);
    let args = args.and_then(item_as_args);
    let chain = chain.and_then(Item::as_array);
    let env = env.and_then(item_as_env);

    // TODO(cnpryer): Propagate env properly
    env.as_ref()
        .map(|it| it.iter().map(|(k, v)| std::env::set_var(k, v)));

    if chain.is_some() && (args.is_some() || program.is_some()) {
        return Err(Error::InvalidRunCommand(
            "only 'env' can be used with 'chain'".to_string(),
        ));
    }

    // Run each chained task
    if let Some(chain) = chain {
        let mut last = None;
        for task in chain.iter().map(Value::as_str) {
            if let Some(it) = task {
                if last.map_or(false, |x| it == x) {
                    return Err(Error::InvalidRunCommand(format!(
                        "'{it}' cannot chain itself"
                    )));
                }

                // TODO(cnpryer): Propagate env
                runner.run(it, config)?;
                last = Some(it);
            } else {
                return Err(Error::InvalidRunCommand("invalid task chain".to_string()));
            }
        }

        return Ok(());
    }

    if let Some(args) = args {
        // If a program is provided we do our best to use it with other configuration.
        // If no program is provided we assume one from the configuration available.
        let program_is_assumed = program.is_none();

        let Some(program) = program.or(args.first().map(Deref::deref)) else {
            return Err(Error::InvalidProgram("could not be resolved".to_string()));
        };

        // We exclude the first argument if the program needed to be assumed.
        if program_is_assumed {
            return run_program(program, &args[1..], env.as_ref(), config);
        }

        return run_program(program, &args, env.as_ref(), config);
    }

    if let Some(Item::Value(value)) = cmd {
        match value {
            Value::String(_) => {
                let string = value_to_sanitized_string(value);
                return run_str(&string, config); // TODO(cnpryer): Environment
            }
            Value::Array(array) => {
                let mut args = Vec::with_capacity(array.len());
                for value in array {
                    match value {
                        Value::Array(_) | Value::InlineTable(_) => {
                            return Err(Error::InvalidRunCommand(
                                "unsupported 'cmd' configuration".to_string(),
                            ))
                        }
                        _ => value.as_str().map(|it| args.push(it.to_string())),
                    };
                }

                if args.is_empty() {
                    return Err(Error::InvalidRunCommand(
                        "could not resolve args".to_string(),
                    ));
                }

                return run_program(&args.remove(0), args, env.as_ref(), config);
            }
            _ => {
                return Err(Error::InvalidRunCommand(
                    "invalid 'cmd' configuration".to_string(),
                ))
            }
        }
    }

    if let Some(program) = program {
        run_program(program, [""], env.as_ref(), config) // TODO(cnpryer): Use Option
    } else {
        Err(Error::InvalidRunCommand(
            "failed to resolve configuration".to_string(),
        ))
    }
}

/// ```toml
/// [tool.huak.task]
/// task1 = ["these", "are", "command", "arguments"]  # ('these' is the program)
/// ```
fn run_array_task(runner: &TaskRunner, array: &Array, config: &Config) -> HuakResult<()> {
    let mut args = Vec::with_capacity(array.len());

    // TODO(cnpryer): Arrays with multiple kinds of Values
    for value in array {
        match value {
            Value::String(_) => args.push(value_to_sanitized_string(value)),
            Value::Array(array) => return run_array_task(runner, array, config),
            Value::InlineTable(table) => return run_inline_table_task(runner, table, config),
            _ => return Err(Error::InvalidProgram(format!("{value}"))),
        }
    }

    if args.is_empty() {
        Err(Error::InvalidRunCommand(
            "failed to resolve program".to_string(),
        ))
    } else {
        run_program(&args.remove(0), args, None, config)
    }
}

/// ```toml
/// [tool.huak.task]
/// task0 = "some command"
/// ````
fn run_formatted_string_task(
    _runner: &TaskRunner,
    string: &Formatted<String>,
    config: &Config,
) -> HuakResult<()> {
    let string = sanitize_str(string.value());

    run_str(string.as_str(), config)
}

fn run_str(s: &str, config: &Config) -> HuakResult<()> {
    let mut cmd = Command::new(shell_name()?);

    let flag = match OS {
        "windows" => "/C",
        _ => "-c",
    };

    add_venv_to_command(&mut cmd, &config.workspace().current_python_environment()?)?;

    cmd.args([flag, s]).current_dir(&config.cwd);

    config.terminal().run_command(&mut cmd)
}

fn run_program<I, S>(
    program: &str,
    args: I,
    env: Option<&HashMap<String, String>>,
    config: &Config,
) -> HuakResult<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(program);

    add_venv_to_command(&mut cmd, &config.workspace().current_python_environment()?)?;

    if let Some(env) = env {
        cmd.envs(env);
    }

    cmd.args(args).current_dir(&config.cwd);
    dbg!(&cmd);

    config.terminal().run_command(&mut cmd)
}

fn item_as_args(item: &Item) -> Option<Vec<String>> {
    if let Item::Value(value) = item {
        match value {
            Value::String(_) => Some(
                value_to_sanitized_string(value)
                    .split(' ')
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>(),
            ),
            Value::Integer(int) => Some(vec![format!("{int}")]),
            Value::Float(float) => Some(vec![format!("{float}")]),
            Value::Boolean(bool) => Some(vec![format!("{bool}")]),
            Value::Datetime(datetime) => Some(vec![datetime.to_string()]),
            Value::Array(array) => {
                let mut args = Vec::new();
                for value in array {
                    if value.is_str() {
                        args.push(value_to_sanitized_string(value));
                    } else {
                        // TODO(cnpryer): Errors
                        return None;
                    }
                }
                Some(args)
            }
            Value::InlineTable(_) => None,
        }
    } else {
        None
    }
}

fn item_as_env(item: &Item) -> Option<HashMap<String, String>> {
    // TODO(cnpryer): Errors?
    let vars = match item {
        Item::Value(Value::InlineTable(table)) => table.to_owned().into_table(),
        Item::Table(table) => table.to_owned(),
        _ => return None,
    };

    let mut env = HashMap::new();

    for (k, v) in vars {
        env.insert(sanitize_str(k.as_str()), sanitize_str(&v.to_string()));
    }

    Some(env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{copy_dir, env_path_string, CopyDirOptions, TerminalOptions, Verbosity};
    use huak_dev::dev_resources_dir;
    use tempfile::tempdir;

    #[test]
    fn test_run_command_str() {
        let dir = tempdir().unwrap();
        copy_dir(
            &dev_resources_dir().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let workspace_root = dir.path().join("mock-project");
        let cwd = workspace_root.clone();
        let terminal_options = TerminalOptions {
            verbosity: Verbosity::Quiet,
            ..Default::default()
        };
        let config = Config {
            workspace_root,
            cwd,
            terminal_options,
            ..Default::default()
        };
        let ws = config.workspace();
        // For some reason this test fails with multiple threads used. Workspace.resolve_python_environment()
        // ends up updating the PATH environment variable causing subsequent Python searches using PATH to fail.
        // TODO
        let env_path = env_path_string().unwrap();
        let venv = ws.resolve_python_environment().unwrap();
        std::env::set_var("PATH", env_path);
        let venv_had_package = venv.contains_module("black").unwrap();

        run_command_str("pip install black", &config).unwrap();

        let venv_contains_package = venv.contains_module("black").unwrap();

        assert!(!venv_had_package);
        assert!(venv_contains_package);
    }
}
