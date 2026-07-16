//! A small, correct YAML block-style emitter for `serde_json::Value`.
//!
//! Every string scalar is double-quoted and escaped, which is always valid
//! YAML — verbose, but it means we never mis-encode a path, version string, or
//! symbol name, and we take on no external YAML dependency.

use serde_json::Value;

/// Render a JSON value as a YAML document.
pub fn to_yaml(value: &Value) -> String {
    let mut out = String::new();
    match value {
        Value::Object(map) if map.is_empty() => out.push_str("{}\n"),
        Value::Array(arr) if arr.is_empty() => out.push_str("[]\n"),
        Value::Object(_) | Value::Array(_) => emit_block(&mut out, value, 0),
        scalar => {
            out.push_str(&scalar_str(scalar));
            out.push('\n');
        }
    }
    out
}

fn emit_block(out: &mut String, value: &Value, indent: usize) {
    let pad = "  ".repeat(indent);
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                out.push_str(&pad);
                out.push_str(&quote(key));
                out.push(':');
                emit_child(out, val, indent);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                out.push_str(&pad);
                out.push('-');
                emit_child(out, item, indent);
            }
        }
        _ => {}
    }
}

/// Emit the value following a `key:` or `-`, choosing inline vs. nested block.
fn emit_child(out: &mut String, val: &Value, indent: usize) {
    match val {
        Value::Object(m) if m.is_empty() => out.push_str(" {}\n"),
        Value::Array(a) if a.is_empty() => out.push_str(" []\n"),
        Value::Object(_) | Value::Array(_) => {
            out.push('\n');
            emit_block(out, val, indent + 1);
        }
        scalar => {
            out.push(' ');
            out.push_str(&scalar_str(scalar));
            out.push('\n');
        }
    }
}

fn scalar_str(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => quote(s),
        // Non-scalars never reach here.
        other => other.to_string(),
    }
}

/// Double-quote and escape a string into an always-valid YAML flow scalar.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
