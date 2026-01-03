use crate::errors::ArbError;
use serde_yaml::Value;

/// Render a template string, replacing `{var}path{/var}` with values from `data`.
/// Supports only `{var}` tags in v1 step-1.
pub fn render_var_only(template_path: &str, input: &str, data: &Value) -> Result<String, ArbError> {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i: usize = 0;

    while i < bytes.len() {
        // Find next "{var}"
        if starts_with(bytes, i, b"{var}") {
            let tag_start = i;
            i += 5; // after "{var}"

            // Find closing "{/var}"
            let close = find_subslice(bytes, i, b"{/var}")
                .ok_or_else(|| template_err(template_path, input, tag_start, "missing closing {/var}".to_string()))?;

            let raw_path = &input[i..close];
            let path = raw_path.trim();
            if path.is_empty() {
                return Err(template_err(template_path, input, tag_start, "empty {var} path".to_string()));
            }

            let v = resolve_path(data, path).ok_or_else(|| {
                template_err(
                    template_path,
                    input,
                    tag_start,
                    format!("missing value at path '{path}'"),
                )
            })?;

            out.push_str(&stringify_value(v));
            i = close + 6; // after "{/var}"
            continue;
        }

        // Normal char
        out.push(bytes[i] as char);
        i += 1;
    }

    Ok(out)
}

fn starts_with(hay: &[u8], pos: usize, needle: &[u8]) -> bool {
    hay.len() >= pos + needle.len() && &hay[pos..pos + needle.len()] == needle
}

fn find_subslice(hay: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    hay[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| from + p)
}

/// Resolve a dot path into YAML Value.
/// Supported:
/// - `foo.bar` for mapping keys
/// - `.` for "current" (here: root data)
fn resolve_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    if path == "." {
        return Some(root);
    }

    let mut cur = root;
    for seg in path.split('.') {
        if seg.is_empty() {
            return None;
        }
        match cur {
            Value::Mapping(m) => {
                let key = Value::String(seg.to_string());
                cur = m.get(&key)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

fn stringify_value(v: &Value) -> String {
    match v {
        Value::Null => "".to_string(),
        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        // For non-scalars: deterministic YAML representation, trimmed.
        _ => {
            let s = serde_yaml::to_string(v).unwrap_or_else(|_| "".to_string());
            s.trim().to_string()
        }
    }
}

fn template_err(path: &str, input: &str, at_index: usize, message: String) -> ArbError {
    let (line, col) = index_to_line_col(input, at_index);
    ArbError::Template {
        path: path.to_string(),
        line,
        col,
        message,
    }
}

fn index_to_line_col(s: &str, idx: usize) -> (usize, usize) {
    // 1-based line/col, best-effort.
    let mut line: usize = 1;
    let mut col: usize = 1;
    for (i, ch) in s.chars().enumerate() {
        if i >= idx {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}
