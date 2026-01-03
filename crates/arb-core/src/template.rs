use crate::errors::ArbError;
use serde_yaml::Value;

/// Render a template string, supporting only `{var}` and `{if}`.
/// - `{var}path{/var}` inserts a required value (missing => error)
/// - `{if}path ... {/if}` conditionally renders body (missing => false)
///
/// Notes:
/// - `{if}` blocks may contain nested `{if}` blocks and `{var}` tags.
/// - `{rep}` blocks may contain other nested blocks
/// - `{inc}` are NOT supported yet.

use std::path::Path;

pub fn render_var_if_rep_inc(
    templates_root: &Path,
    template_rel: &str,
    input: &str,
    data: &Value,
) -> Result<String, ArbError> {
    let mut stack: Vec<String> = vec![template_rel.to_string()];
    render_inner(templates_root, template_rel, input, data, 0, &mut stack)
}


const MAX_EXPANSION_DEPTH: usize = 128;
const MAX_INCLUDE_DEPTH: usize = 32;

fn render_inner(
    templates_root: &std::path::Path,
    template_rel: &str,
    input: &str,
    data: &Value,
    depth: usize,
    include_stack: &mut Vec<String>,
) -> Result<String, ArbError> {

    if depth > MAX_EXPANSION_DEPTH {
        return Err(template_err(
            template_rel,
            input,
            0,
            format!("expansion depth limit exceeded (>{MAX_EXPANSION_DEPTH})"),
        ));
    }

    if depth > MAX_EXPANSION_DEPTH {
        return Err(template_err(
            template_rel,
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
                .ok_or_else(|| template_err(template_rel, input, tag_start, "missing closing {/var}".to_string()))?;

            let raw_path = &input[i..close];
            let path = raw_path.trim();
            if path.is_empty() {
                return Err(template_err(template_rel, input, tag_start, "empty {var} path".to_string()));
            }

            let v = resolve_path(data, path).ok_or_else(|| {
                template_err(
                    template_rel,
                    input,
                    tag_start,
                    format!("missing value at path '{path}'"),
                )
            })?;

            out.push_str(&stringify_value(v));
            i = close + 6;
            continue;
        }

        // {rep}
        if starts_with(bytes, i, b"{rep}") {
            let tag_start = i;
            i += 5;

            // Skip whitespace after {rep}
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
                return Err(template_err(template_rel, input, tag_start, "empty {rep} path".to_string()));
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
                while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                    i += 1;
                }
            }

            // Find matching {/rep}, allowing nested {rep}...{/rep}
            let (body_end, close_end) = find_matching_rep_close(bytes, i).ok_or_else(|| {
                template_err(template_rel, input, tag_start, "missing closing {/rep}".to_string())
            })?;

            let body = &input[i..body_end];

            // Per spec: invalid paths are errors for {rep}
            let v = resolve_path(data, path).ok_or_else(|| {
                template_err(
                    template_rel,
                    input,
                    tag_start,
                    format!("missing value at path '{path}'"),
                )
            })?;

            let seq = v.as_sequence().ok_or_else(|| {
                template_err(
                    template_rel,
                    input,
                    tag_start,
                    format!("expected list at path '{path}'"),
                )
            })?;

            for item in seq {
                // Context becomes the list item during each iteration
                let rendered_body = render_inner(
                    templates_root,
                    template_rel,
                    body,
                    item,
                    depth + 1,
                    include_stack,
                )?;
                out.push_str(&rendered_body);
            }

            i = close_end;
            continue;
        }

        // {inc}
        if starts_with(bytes, i, b"{inc}") {
            let tag_start = i;
            i += 5; // after "{inc}"

            let close = find_subslice(bytes, i, b"{/inc}")
                .ok_or_else(|| template_err(template_rel, input, tag_start, "missing closing {/inc}".to_string()))?;

            let raw_path = input[i..close].trim();
            if raw_path.is_empty() {
                return Err(template_err(template_rel, input, tag_start, "empty {inc} path".to_string()));
            }

            // Resolve include path relative to current template
            let (inc_rel, inc_abs) = resolve_include_path(templates_root, template_rel, raw_path)
                .map_err(|msg| template_err(template_rel, input, tag_start, msg))?;

            // Cycle detection
            if include_stack.iter().any(|p| p == &inc_rel) {
                let mut chain = include_stack.clone();
                chain.push(inc_rel.clone());
                return Err(template_err(
                    template_rel,
                    input,
                    tag_start,
                    format!("include cycle detected: {}", chain.join(" -> ")),
                ));
            }

            // Include depth limit
            if include_stack.len() >= MAX_INCLUDE_DEPTH {
                return Err(template_err(
                    template_rel,
                    input,
                    tag_start,
                    format!("include depth limit exceeded (>{MAX_INCLUDE_DEPTH})"),
                ));
            }

            let inc_text = std::fs::read_to_string(&inc_abs)
                .map_err(|e| template_err(template_rel, input, tag_start, format!("include read failed: {e}")))?;

            // Render included template with the SAME current context (`data`)
            include_stack.push(inc_rel.clone());
            let rendered_inc = render_inner(
                templates_root,
                &inc_rel,
                &inc_text,
                data,
                depth + 1,
                include_stack,
            )?;
            include_stack.pop();

            out.push_str(&rendered_inc);
            i = close + 6; // after "{/inc}"
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
                return Err(template_err(template_rel, input, tag_start, "empty {if} path".to_string()));
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
                template_err(template_rel, input, tag_start, "missing closing {/if}".to_string())
            })?;

            let body = &input[i..body_end];

            let cond = match resolve_path(data, path) {
                Some(v) => is_truthy(v),
                None => false, // per spec: missing => false
            };

            if cond {
                let rendered_body = render_inner(
                    templates_root,
                    template_rel,
                    body,
                    data,
                    depth + 1,
                    include_stack,
                )?;
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

/// Finds the matching {/rep} for a block starting at `from` (the body start),
/// supporting nested {rep} ... {/rep}.
/// Returns (body_end_index, index_after_close_tag).
fn find_matching_rep_close(hay: &[u8], from: usize) -> Option<(usize, usize)> {
    let mut i = from;
    let mut depth: usize = 1;

    while i < hay.len() {
        if starts_with(hay, i, b"{rep}") {
            depth += 1;
            i += 5;
            continue;
        }
        if starts_with(hay, i, b"{/rep}") {
            depth -= 1;
            if depth == 0 {
                let body_end = i;
                let close_end = i + 6; // after "{/rep}"
                return Some((body_end, close_end));
            }
            i += 6;
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

/// Resolve a dot path into a YAML Value, relative to the current context.
/// Supported in v1:
/// - `foo.bar` mapping traversal (string keys only)
/// - `.` returns the current context value
///
/// Note:
/// - Outside `{rep}`, the current context is the root data document.
/// - Inside `{rep}`, the current context is the current list item.
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

use std::path::{PathBuf, Component};

fn resolve_include_path(
    templates_root: &Path,
    current_template_rel: &str,
    include_str: &str,
) -> Result<(String, PathBuf), String> {
    if include_str.contains('\\') {
        return Err("include paths must use '/' separators".to_string());
    }
    if include_str.starts_with('/') || include_str.contains(':') {
        return Err("include path must be relative".to_string());
    }
    if !include_str.ends_with(".arb") {
        return Err("include target must be a .arb template".to_string());
    }

    let cur_rel = Path::new(current_template_rel);
    let base_dir = cur_rel.parent().unwrap_or_else(|| Path::new(""));

    // Normalize: base_dir + include_str, preventing escape above templates/
    let mut parts: Vec<String> = Vec::new();

    for c in base_dir.components() {
        match c {
            Component::Normal(os) => parts.push(os.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err("include resolution attempted to escape templates/".to_string());
                }
            }
            _ => return Err("invalid template base path".to_string()),
        }
    }

    for seg in include_str.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            if parts.pop().is_none() {
                return Err("include resolution attempted to escape templates/".to_string());
            }
            continue;
        }
        if seg.contains('{') || seg.contains('}') {
            return Err("include path must be literal (no tags)".to_string());
        }
        parts.push(seg.to_string());
    }

    let rel = parts.join("/");
    let abs = templates_root.join(Path::new(&rel));

    // Must remain under templates_root (best-effort without canonicalize)
    if !abs.starts_with(templates_root) {
        return Err("include resolution escaped templates/".to_string());
    }
    if !abs.is_file() {
        return Err(format!("include file not found: {rel}"));
    }

    Ok((rel, abs))
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


