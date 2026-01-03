use crate::errors::ArbError;
use serde_yaml::Value;

/// Render a template string, supporting only `{var}` and `{if}`.
/// - `{var}path{/var}` inserts a required value (missing => error)
/// - `{if}path ... {/if}` conditionally renders body (missing => false)
///
/// Notes:
/// - `{if}` blocks may contain nested `{if}` blocks and `{var}` tags.
/// - `{rep}` and `{inc}` are NOT supported yet.
pub fn render_var_if_only(template_path: &str, input: &str, data: &Value) -> Result<String, ArbError> {
    render_inner(template_path, input, data, 0)
}

const MAX_EXPANSION_DEPTH: usize = 128;

fn render_inner(template_path: &str, input: &str, data: &Value, depth: usize) -> Result<String, ArbError> {
    if depth > MAX_EXPANSION_DEPTH {
        return Err(template_err(
            template_path,
            input,
            0,
            format!("expansion depth limit exceeded (>{MAX_EXPANSION_DEPTH})"),
        ));
    }

    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i: usize = 0;

    while i < bytes.len() {
        // {var}
        if starts_with(bytes, i, b"{var}") {
            let tag_start = i;
            i += 5;

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
            i = close + 6;
            continue;
        }

        // {if}
        if starts_with(bytes, i, b"{if}") {
            let tag_start = i;
            i += 4;

            // Skip whitespace after {if}
            while i < bytes.len() && is_ws(bytes[i]) {
                i += 1;
            }

            // Read path token until whitespace
            let path_start = i;
            while i < bytes.len() && !is_ws(bytes[i]) {
                i += 1;
            }
            let path = input[path_start..i].trim();

            if path.is_empty() {
                return Err(template_err(template_path, input, tag_start, "empty {if} path".to_string()));
            }

            // Consume optional single newline after the path for readability
            if i < bytes.len() && bytes[i] == b'\r' {
                i += 1;
                if i < bytes.len() && bytes[i] == b'\n' {
                    i += 1;
                }
            } else if i < bytes.len() && bytes[i] == b'\n' {
                i += 1;
            } else {
                // otherwise skip spaces/tabs before body
                while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                    i += 1;
                }
            }

            // Find matching {/if}, allowing nested {if} ... {/if}
            let (body_end, close_end) = find_matching_if_close(bytes, i).ok_or_else(|| {
                template_err(template_path, input, tag_start, "missing closing {/if}".to_string())
            })?;

            let body = &input[i..body_end];

            let cond = match resolve_path(data, path) {
                Some(v) => is_truthy(v),
                None => false, // per spec: missing => false
            };

            if cond {
                let rendered_body = render_inner(template_path, body, data, depth + 1)?;
                out.push_str(&rendered_body);
            }

            i = close_end;
            continue;
        }

        // Normal char
        out.push(bytes[i] as char);
        i += 1;
    }

    Ok(out)
}

fn is_ws(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\r' || b == b'\n'
}

/// Finds the matching {/if} for a block starting at `from` (the body start),
/// supporting nested {if} ... {/if}.
/// Returns (body_end_index, index_after_close_tag).
fn find_matching_if_close(hay: &[u8], from: usize) -> Option<(usize, usize)> {
    let mut i = from;
    let mut depth: usize = 1;

    while i < hay.len() {
        if starts_with(hay, i, b"{if}") {
            depth += 1;
            i += 4;
            continue;
        }
        if starts_with(hay, i, b"{/if}") {
            depth -= 1;
            if depth == 0 {
                let body_end = i;
                let close_end = i + 5; // after "{/if}"
                return Some((body_end, close_end));
            }
            i += 5;
            continue;
        }
        i += 1;
    }
    None
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
/// Supported (for now):
/// - `foo.bar` mapping traversal
/// - `.` for root
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

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => {
            // non-zero is true
            if let Some(i) = n.as_i64() {
                i != 0
            } else if let Some(u) = n.as_u64() {
                u != 0
            } else if let Some(f) = n.as_f64() {
                f != 0.0
            } else {
                false
            }
        }
        Value::String(s) => !s.is_empty(),
        Value::Sequence(seq) => !seq.is_empty(),
        Value::Mapping(map) => !map.is_empty(),
        Value::Tagged(tv) => is_truthy(&tv.value),
        // safe default
        //_ => false,
    }
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


