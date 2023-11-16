use toml_edit::{Array, RawString, Table, Value};

#[must_use]
pub fn value_to_sanitized_string(value: &Value) -> String {
    match value {
        Value::String(string) => sanitize_str(string.value()),
        _ => value.to_string(),
    }
}

pub(crate) fn sanitize_str(s: &str) -> String {
    s.trim_matches('\n')
        .trim()
        .trim_start_matches(['\\', '\'', '"'])
        .to_string()
}

pub fn format_table(table: &mut Table) {
    for array in table.iter_mut().filter_map(|(_, v)| v.as_array_mut()) {
        format_array(array);
    }
}

/// See Rye for original implementation
/// Reformats a TOML array to multi line while trying to
/// preserve all comments and move them around.  This also makes
/// the array to have a trailing comma.
pub fn format_array(array: &mut Array) {
    if array.is_empty() {
        return;
    }

    for item in array.iter_mut() {
        let decor = item.decor_mut();
        let mut prefix = String::new();
        for comment in find_comments(decor.prefix()).chain(find_comments(decor.suffix())) {
            prefix.push_str("\n    ");
            prefix.push_str(comment);
        }
        prefix.push_str("\n    ");
        decor.set_prefix(prefix);
        decor.set_suffix("");
    }

    array.set_trailing(&{
        let mut comments = find_comments(Some(array.trailing())).peekable();
        let mut rv = String::new();
        if comments.peek().is_some() {
            for comment in comments {
                rv.push_str("\n    ");
                rv.push_str(comment);
            }
        }
        rv.push('\n');
        rv
    });

    array.set_trailing_comma(true);
}

fn find_comments(s: Option<&RawString>) -> impl Iterator<Item = &str> {
    s.and_then(|x| x.as_str())
        .unwrap_or("")
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.starts_with('#').then_some(line)
        })
}
