//! Interactive `instance create --wizard` (stdin prompts).

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::io::{self, Write};
use std::path::Path;

use crate::cli::json_util::{
    form_api_string_from_array_value, read_json_path_or_stdin, scaffold_root_widgets_from_approval_data,
};

/// Prompt for `open_id` when neither `--open-id` nor `--user-id` was passed.
pub fn ensure_open_id_or_user_id(
    open_id: Option<&String>,
    user_id: Option<&String>,
) -> Result<(Option<String>, Option<String>)> {
    if open_id.is_some() || user_id.is_some() {
        return Ok((open_id.cloned(), user_id.cloned()));
    }
    let line = prompt_line(
        "open_id (required for wizard if you did not pass --open-id / --user-id): ",
    )?;
    let line = line.trim().to_string();
    if line.is_empty() {
        bail!("wizard needs an actor: pass --open-id or --user-id, or type open_id when prompted");
    }
    Ok((Some(line), None))
}

fn prompt_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush().context("stdout")?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .context("read line from stdin")?;
    Ok(line)
}

/// Read a widgets template path, prompt for simple widget values, return API `form` string.
pub fn interactive_form_string() -> Result<String> {
    let path_line = prompt_line(
        "Path to widgets JSON array OR approval dump JSON (file or `-` for stdin): ",
    )?;
    let path = Path::new(path_line.trim());
    let root = read_json_path_or_stdin(path)?;

    let mut widgets: Vec<Value> = if root.is_array() {
        root.as_array()
            .expect("array")
            .clone()
    } else {
        let tpl = scaffold_root_widgets_from_approval_data(&root)?;
        tpl.as_array()
            .ok_or_else(|| anyhow!("scaffold-widgets produced non-array"))?
            .clone()
    };

    eprintln!("--- Enter values (empty line keeps the current value). JSON literals accepted. Ctrl+Z then Enter (Windows) or Ctrl+D (Unix) may be needed if stdin is a pipe. ---");

    for (i, w) in widgets.iter_mut().enumerate() {
        let Some(obj) = w.as_object_mut() else {
            continue;
        };
        let id = obj
            .get("id")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_string();
        let t = obj
            .get("type")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_string();
        let name = obj
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();

        match t.as_str() {
            "fieldList" | "fieldListMobile" => {
                eprintln!(
                    "[{i}] id={id} type={t} name={name} — nested rows: edit the JSON file or paste a JSON array for value after this session; skipping prompt."
                );
                continue;
            }
            "image" | "attachment" => {
                eprintln!(
                    "[{i}] id={id} type={t} — use `file upload` for a file code, then paste JSON like {{\"code\":\"...\"}} as value; skipping prompt."
                );
                continue;
            }
            _ => {}
        }

        let cur = obj.get("value").cloned().unwrap_or(Value::Null);
        let hint = match t.as_str() {
            "date" => " (RFC3339 e.g. 2026-03-22T00:00:00+08:00)",
            "formula" => " (often must be non-empty, e.g. same as amount)",
            _ => "",
        };
        eprintln!("[{i}] id={id} type={t} name={name}{hint}");
        eprintln!("    current value: {cur}");
        let line = prompt_line("    > ")?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let new_val: Value = serde_json::from_str(line).unwrap_or_else(|_| Value::String(line.to_string()));
        obj.insert("value".into(), new_val);
    }

    let arr = Value::Array(widgets);
    form_api_string_from_array_value(&arr)
}
