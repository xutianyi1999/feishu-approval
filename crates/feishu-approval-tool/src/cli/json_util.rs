use anyhow::{Context, Result, bail};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::sync::OnceLock;

pub fn read_json_file(path: &Path) -> Result<Value> {
    let s = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&s).with_context(|| format!("invalid JSON in {}", path.display()))
}

/// Read a JSON **object** from a file or stdin (`-`). Used when the API expects `{ ... }` (e.g. `tasks/search` body).
pub fn read_json_object_path_or_stdin(path: &Path) -> Result<Map<String, Value>> {
    let v = read_json_path_or_stdin(path)?;
    match v {
        Value::Object(m) => Ok(m),
        _ => bail!("expected JSON object {{ ... }}, not an array or primitive"),
    }
}

/// Read JSON from a file, or from stdin when `path` is `-`.
pub fn read_json_path_or_stdin(path: &Path) -> Result<Value> {
    if path == Path::new("-") {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("read JSON from stdin")?;
        serde_json::from_str(buf.trim()).context("invalid JSON from stdin")
    } else {
        read_json_file(path)
    }
}

/// Parse a JSON object from a string (for CLI `--extra-json-inline`).
pub fn parse_json_object_from_str(s: &str) -> Result<Value> {
    let v: Value =
        serde_json::from_str(s.trim()).context("invalid JSON in --extra-json-inline")?;
    if !v.is_object() {
        anyhow::bail!("--extra-json-inline must be a JSON object {{ ... }}, not an array or primitive");
    }
    Ok(v)
}

pub fn read_string_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .with_context(|| format!("read {}", path.display()))
}

/// Serialize a JSON **array** of widget values to the API `form` field string (compact, one line).
pub fn form_api_string_from_array_value(v: &Value) -> Result<String> {
    if !v.is_array() {
        anyhow::bail!("expected a JSON array of form widgets [...], not an object or primitive");
    }
    serde_json::to_string(v).context("serialize form widget array")
}

/// Read a widget JSON array from a path or stdin (`-`); return the API `form` string.
pub fn form_string_from_widgets_json_path(path: &Path) -> Result<String> {
    let v = read_json_path_or_stdin(path)?;
    form_api_string_from_array_value(&v)
}

/// Offline checks for the JSON array used with `--widgets-json-file` / `util form-string` / `instance create`.
/// Ensures each element is an object with non-empty string `id` and `type`, and a present `value` key.
/// For **`type` = `fieldList`**, `value` must be a JSON array of rows; each row is an array of column
/// widgets, each of which must also have **`id` + `type` + `value`** (Feishu often returns a vague
/// 「控件类型为空」「index=0」when an inner cell omits `type`).
///
/// Extra heuristics (best-effort, Feishu may still reject edge cases): **`date`** → `value` must look
/// like **RFC3339 with offset or `Z`** if non-null; **`amount`** / **`formula`** → non-null `value`
/// must be a JSON number or a **non-empty** numeric string.
pub fn validate_widgets_json_value(v: &Value) -> Result<()> {
    let arr = v
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("expected a JSON array [...] of widgets"))?;
    for (i, item) in arr.iter().enumerate() {
        validate_one_widget_value(item, &format!("widgets[{i}]"))?;
    }
    Ok(())
}

fn validate_one_widget_value(item: &Value, path: &str) -> Result<()> {
    let obj = item
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("{path}: expected object"))?;
    let id = obj
        .get("id")
        .ok_or_else(|| anyhow::anyhow!("{path}: missing \"id\""))?;
    let id_s = id
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("{path}: \"id\" must be a string"))?;
    if id_s.is_empty() {
        bail!("{path}: \"id\" must not be empty");
    }
    let t = obj
        .get("type")
        .ok_or_else(|| anyhow::anyhow!("{path}: missing \"type\""))?;
    let t_s = t
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("{path}: \"type\" must be a string"))?;
    if t_s.is_empty() {
        bail!("{path}: \"type\" must not be empty");
    }
    if !obj.contains_key("value") {
        bail!("{path}: missing \"value\" key (use null only if the widget type allows it)");
    }
    if t_s == "fieldList" || t_s == "fieldListMobile" {
        validate_fieldlist_rows(obj.get("value"), path)?;
    } else {
        validate_widget_value_semantics(path, t_s, obj.get("value"))?;
    }
    Ok(())
}

fn rfc3339_like_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$")
            .expect("rfc3339-like regex")
    })
}

fn validate_widget_value_semantics(path: &str, widget_type: &str, value: Option<&Value>) -> Result<()> {
    let Some(v) = value else {
        return Ok(());
    };
    if v.is_null() {
        return Ok(());
    }
    match widget_type {
        "date" => {
            let s = v.as_str().ok_or_else(|| {
                anyhow::anyhow!("{path}: type \"date\" expects \"value\" as a string (RFC3339 with offset or Z), got {v}")
            })?;
            if !rfc3339_like_regex().is_match(s) {
                bail!(
                    "{path}: type \"date\" \"value\" must look like RFC3339 (e.g. 2026-03-22T09:00:00+08:00), got {s:?}"
                );
            }
        }
        "amount" | "formula" => match v {
            Value::Number(n) => {
                if n.as_f64().map(|x| !x.is_finite()).unwrap_or(true) {
                    bail!("{path}: type {widget_type:?} \"value\" number must be finite");
                }
            }
            Value::String(s) => {
                if s.trim().is_empty() {
                    return Ok(());
                }
                if s.trim().parse::<f64>().is_err() {
                    bail!(
                        "{path}: type {widget_type:?} \"value\" must be a JSON number or a numeric string, got {s:?}"
                    );
                }
            }
            _ => bail!(
                "{path}: type {widget_type:?} \"value\" must be a JSON number or numeric string, got {v}"
            ),
        },
        _ => {}
    }
    Ok(())
}

fn validate_fieldlist_rows(value_field: Option<&Value>, path: &str) -> Result<()> {
    let Some(v) = value_field else {
        bail!("{path}: fieldList missing \"value\"");
    };
    if v.is_null() {
        // e.g. `util scaffold-widgets` / wizard placeholder — no rows to check yet
        return Ok(());
    }
    let rows = v
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("{path}: fieldList \"value\" must be a JSON array of rows (each row is an array of {{id,type,value}} widgets)"))?;
    for (ri, row) in rows.iter().enumerate() {
        let cells = row.as_array().ok_or_else(|| {
            anyhow::anyhow!("{path}: fieldList value[{ri}] must be an array of column widgets, not a single object")
        })?;
        for (ci, cell) in cells.iter().enumerate() {
            let cell_path = format!("{path}.value[{ri}][{ci}]");
            validate_one_widget_value(cell, &cell_path)?;
        }
    }
    Ok(())
}

/// Resolve the approval **`data`** object from **`approval dump --data-only`** output, or from a full GET response (`{ "data": { "form": ... } }`).
pub fn approval_inner_data_object(root: &Value) -> Result<&Map<String, Value>> {
    let as_obj = root
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("expected JSON object"))?;
    if let Some(Value::Object(d)) = as_obj.get("data") {
        return Ok(d);
    }
    if as_obj.contains_key("form") || as_obj.contains_key("form_content") {
        return Ok(as_obj);
    }
    bail!("expected approval `data` (e.g. `approval dump --data-only`) or full response with `.data` containing `form`")
}

fn form_field_value(data: &Map<String, Value>) -> Result<&Value> {
    if let Some(v) = data.get("form") {
        return Ok(v);
    }
    if let Some(v) = data.get("form_content") {
        return Ok(v);
    }
    bail!("approval data has no `form` or `form_content`")
}

/// Parse `data.form` (string or array) into widget definition objects.
pub fn approval_form_widgets_from_data(root: &Value) -> Result<Vec<Value>> {
    let data = approval_inner_data_object(root)?;
    let form = form_field_value(data)?;
    parse_form_field_to_widgets_array(form)
}

fn parse_form_field_to_widgets_array(form: &Value) -> Result<Vec<Value>> {
    match form {
        Value::String(s) => {
            let v: Value =
                serde_json::from_str(s.trim()).context("approval `form` string is not valid JSON")?;
            let arr = v
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("approval `form` JSON must be an array"))?;
            Ok(arr.clone())
        }
        Value::Array(a) => Ok(a.clone()),
        _ => bail!("approval `form` must be a JSON string or array"),
    }
}

/// Compact tree for AI: `id`, `type`, `name`, `required`, optional `options` (from `option` array), recursive `children`.
pub fn widget_definition_skeleton(w: &Value) -> Value {
    let Some(obj) = w.as_object() else {
        return Value::Null;
    };
    let mut m = Map::new();
    if let Some(v) = obj.get("id") {
        m.insert("id".into(), v.clone());
    }
    if let Some(v) = obj.get("type") {
        m.insert("type".into(), v.clone());
    }
    if let Some(v) = obj.get("name") {
        m.insert("name".into(), v.clone());
    }
    if let Some(v) = obj.get("required") {
        m.insert("required".into(), v.clone());
    }
    if let Some(Value::Array(opts)) = obj.get("option") {
        let simplified: Vec<Value> = opts
            .iter()
            .filter_map(|x| {
                x.as_object().map(|o| {
                    json!({
                        "value": o.get("value"),
                        "text": o.get("text"),
                    })
                })
            })
            .collect();
        if !simplified.is_empty() {
            m.insert("options".into(), Value::Array(simplified));
        }
    }
    if let Some(Value::Array(children)) = obj.get("children") {
        let sk: Vec<Value> = children.iter().map(widget_definition_skeleton).collect();
        m.insert("children".into(), Value::Array(sk));
    }
    Value::Object(m)
}

/// Build skeleton JSON array from approval dump JSON (see [`approval_inner_data_object`]).
pub fn extract_widget_skeletons_value(root: &Value) -> Result<Value> {
    let widgets = approval_form_widgets_from_data(root)?;
    let out: Vec<Value> = widgets.iter().map(widget_definition_skeleton).collect();
    Ok(Value::Array(out))
}

/// Top-level form rows for `widgets.json`: one `{ "id", "type", "value": … }` per **root** definition widget (offline).
/// For **`fieldList` / `fieldListMobile`**, `value` is **`[[ { id, type, value }, … ]]`** — one template row built from
/// definition **`children`** (`value`: `null` per column, or nested `fieldList` shape if a column is also fieldList).
pub fn scaffold_root_widgets_from_approval_data(root: &Value) -> Result<Value> {
    let widgets = approval_form_widgets_from_data(root)?;
    let mut out = Vec::new();
    for (i, w) in widgets.iter().enumerate() {
        let o = w
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("definition form[{i}]: expected object"))?;
        let id = o
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("definition form[{i}]: missing string id"))?;
        let t = o
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("definition form[{i}]: missing string type"))?;
        let value = scaffold_value_from_definition_object(o)
            .with_context(|| format!("definition form[{i}] id={id:?} type={t:?}"))?;
        out.push(json!({
            "id": id,
            "type": t,
            "value": value,
        }));
    }
    Ok(Value::Array(out))
}

fn scaffold_value_from_definition_object(o: &Map<String, Value>) -> Result<Value> {
    let t = o.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match t {
        "fieldList" | "fieldListMobile" => {
            let inner = build_one_fieldlist_row_from_children(o)?;
            if inner.is_empty() {
                Ok(json!([]))
            } else {
                Ok(Value::Array(vec![Value::Array(inner)]))
            }
        }
        _ => Ok(Value::Null),
    }
}

/// One row: one JSON object per column from `children`, matching API instance shape.
fn build_one_fieldlist_row_from_children(field_list_def: &Map<String, Value>) -> Result<Vec<Value>> {
    let Some(Value::Array(children)) = field_list_def.get("children") else {
        return Ok(Vec::new());
    };
    let mut cols = Vec::new();
    for (j, ch) in children.iter().enumerate() {
        let co = ch
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("fieldList children[{j}]: expected object"))?;
        let cid = co
            .get("id")
            .and_then(|x| x.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow::anyhow!("fieldList children[{j}]: missing non-empty string id"))?;
        let ct = co
            .get("type")
            .and_then(|x| x.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow::anyhow!("fieldList children[{j}]: missing non-empty string type"))?;
        let cell_value = match ct {
            "fieldList" | "fieldListMobile" => scaffold_value_from_definition_object(co)?,
            _ => Value::Null,
        };
        cols.push(json!({
            "id": cid,
            "type": ct,
            "value": cell_value,
        }));
    }
    Ok(cols)
}

fn collect_definition_id_types(widgets: &[Value], map: &mut HashMap<String, String>) {
    for w in widgets {
        let Some(o) = w.as_object() else { continue };
        if let (Some(Value::String(id)), Some(Value::String(t))) = (o.get("id"), o.get("type")) {
            map.insert(id.clone(), t.clone());
        }
        if let Some(Value::Array(ch)) = o.get("children") {
            collect_definition_id_types(ch, map);
        }
    }
}

/// Check each top-level widget in `user_widgets` (array) exists in the approval definition `form` tree with the same `type`.
pub fn validate_widgets_against_approval_data(user_widgets: &Value, approval_root: &Value) -> Result<()> {
    let def_widgets = approval_form_widgets_from_data(approval_root)?;
    let mut def_map: HashMap<String, String> = HashMap::new();
    collect_definition_id_types(&def_widgets, &mut def_map);
    let arr = user_widgets
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("widgets must be a JSON array"))?;
    for (i, item) in arr.iter().enumerate() {
        let o = item
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("widgets[{i}]: expected object"))?;
        let id = o
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("widgets[{i}]: missing string id"))?;
        let utype = o
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("widgets[{i}]: missing string type"))?;
        let def_type = def_map.get(id).ok_or_else(|| {
            anyhow::anyhow!(
                "widgets[{i}]: id {id:?} not in approval definition form (check approval dump --data-only)"
            )
        })?;
        if def_type != utype {
            bail!(
                "widgets[{i}]: id {id:?} type mismatch: definition has {def_type:?}, widgets have {utype:?}"
            );
        }
    }
    Ok(())
}

/// Top-level keys on `POST .../instances` whose values are arrays of `{ "key", "value" }` per Feishu docs (not `node_id` / `approver_*_list` inside each element).
const INSTANCE_NODE_LIST_KEYS: &[&str] = &[
    "node_approver_open_id_list",
    "node_approver_user_id_list",
    "node_cc_open_id_list",
    "node_cc_user_id_list",
];

/// Reject common mistakes when building `--extra-json` for `instance create` (Feishu uses **`key`** + **`value`** per element; parameter tables are easy to misread).
pub fn validate_instance_create_extra_patch(patch: &Value) -> Result<()> {
    let Some(root) = patch.as_object() else {
        return Ok(());
    };
    for list_key in INSTANCE_NODE_LIST_KEYS {
        let Some(Value::Array(items)) = root.get(*list_key) else {
            continue;
        };
        for (i, item) in items.iter().enumerate() {
            let Some(o) = item.as_object() else {
                continue;
            };
            if o.contains_key("node_id") {
                bail!(
                    "extra JSON field `{list_key}[{i}]`: Feishu expects `key` (node id from `approval get` → `node_list`), not `node_id`. Each item is `{{ \"key\": \"…\", \"value\": [\"ou_…\"] }}`. See `docs/AI.md` (instance create / extra-json) and `embedded-docs/reference/approval-v4/instance/create.md` request body example."
                );
            }
            if o.contains_key("approver_open_id_list") || o.contains_key("approver_user_id_list") {
                bail!(
                    "extra JSON field `{list_key}[{i}]`: use `value` (JSON array of open_id or user_id strings), not `approver_open_id_list` / `approver_user_id_list` as inner keys. See `docs/examples/instance-create-extra.sample.json`."
                );
            }
        }
    }
    Ok(())
}

/// Shallow-merge: top-level keys from `patch` overwrite `base`. If `patch` is not an object, returns error.
pub fn merge_object(base: &mut Map<String, Value>, patch: Value) -> Result<()> {
    let obj = patch
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("merge JSON must be a JSON object"))?;
    for (k, v) in obj {
        base.insert(k.clone(), v.clone());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_instance_create_extra_rejects_guessed_inner_field_names() {
        let ok = json!({
            "node_approver_open_id_list": [{"key": "nid", "value": ["ou_x"]}]
        });
        validate_instance_create_extra_patch(&ok).unwrap();

        let bad_node_id = json!({
            "node_approver_open_id_list": [{"node_id": "nid", "value": ["ou_x"]}]
        });
        assert!(validate_instance_create_extra_patch(&bad_node_id).is_err());

        let bad_inner_list = json!({
            "node_approver_open_id_list": [{"key": "k", "approver_open_id_list": ["ou_x"]}]
        });
        assert!(validate_instance_create_extra_patch(&bad_inner_list).is_err());
    }

    #[test]
    fn merge_object_overwrites_and_rejects_non_object_patch() {
        let mut m = Map::new();
        m.insert("a".into(), json!(1));
        merge_object(&mut m, json!({"a": 2, "b": 3})).unwrap();
        assert_eq!(m.get("a").unwrap(), &json!(2));
        assert_eq!(m.get("b").unwrap(), &json!(3));
        let err = merge_object(&mut Map::new(), json!([1])).unwrap_err();
        assert!(err.to_string().contains("JSON object"));
    }

    #[test]
    fn parse_json_object_from_str_accepts_object_rejects_array() {
        assert_eq!(
            parse_json_object_from_str(r#"  {"x": true} "#).unwrap(),
            json!({"x": true})
        );
        let err = parse_json_object_from_str("[1]").unwrap_err();
        assert!(err.to_string().contains("object"));
    }

    #[test]
    fn form_api_string_serializes_array() {
        let v = json!([{"id": "w1", "type": "input", "value": "hi"}]);
        let s = form_api_string_from_array_value(&v).unwrap();
        assert!(s.starts_with('[') && s.contains("w1"));
        assert!(form_api_string_from_array_value(&json!({"a": 1})).is_err());
    }

    #[test]
    fn form_string_from_widgets_json_path_reads_file() {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "feishu_approval_tool_widgets_form_{}.json",
            std::process::id()
        ));
        let w = json!([{"id": "w1", "type": "input", "value": "hi"}]);
        std::fs::write(&p, serde_json::to_string(&w).unwrap()).unwrap();
        let s = form_string_from_widgets_json_path(&p).unwrap();
        assert!(s.starts_with('[') && s.contains("w1"));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn validate_widgets_accepts_sample_shape() {
        let v = json!([{"id": "a", "type": "input", "value": "x"}]);
        validate_widgets_json_value(&v).unwrap();
        assert!(validate_widgets_json_value(&json!({"a": 1})).is_err());
        assert!(validate_widgets_json_value(&json!([{"type": "input", "value": 1}])).is_err());
        assert!(validate_widgets_json_value(&json!([{"id": "", "type": "input", "value": 1}])).is_err());
        let ok_null = json!([{"id": "w", "type": "input", "value": null}]);
        validate_widgets_json_value(&ok_null).unwrap();
    }

    #[test]
    fn validate_widgets_fieldlist_inner_requires_type() {
        let ok = json!([{
            "id": "fl",
            "type": "fieldList",
            "value": [[
                {"id": "c1", "type": "input", "value": "x"},
                {"id": "c2", "type": "date", "value": "2026-03-22T09:00:00+08:00"}
            ]]
        }]);
        validate_widgets_json_value(&ok).unwrap();

        let bad = json!([{
            "id": "fl",
            "type": "fieldList",
            "value": [[{"id": "c1", "value": "x"}]]
        }]);
        let e = validate_widgets_json_value(&bad).unwrap_err();
        let s = e.to_string();
        assert!(s.contains("value[0][0]"), "{s}");
        assert!(s.contains("type"), "{s}");

        let null_rows = json!([{"id": "fl", "type": "fieldList", "value": null}]);
        validate_widgets_json_value(&null_rows).unwrap();
    }

    #[test]
    fn validate_widgets_date_value_rfc3339() {
        let ok = json!([{"id": "d", "type": "date", "value": "2026-03-22T09:00:00Z"}]);
        validate_widgets_json_value(&ok).unwrap();
        let ok2 = json!([{"id": "d", "type": "date", "value": "2026-03-22T09:00:00+08:00"}]);
        validate_widgets_json_value(&ok2).unwrap();
        let bad = json!([{"id": "d", "type": "date", "value": "2026-03-22"}]);
        assert!(validate_widgets_json_value(&bad).unwrap_err().to_string().contains("RFC3339"));
        let bad2 = json!([{"id": "d", "type": "date", "value": 1}]);
        assert!(validate_widgets_json_value(&bad2).is_err());
    }

    #[test]
    fn validate_widgets_amount_formula_numeric() {
        let ok = json!([
            {"id": "a", "type": "amount", "value": "12.5"},
            {"id": "f", "type": "formula", "value": 0},
        ]);
        validate_widgets_json_value(&ok).unwrap();
        let bad = json!([{"id": "a", "type": "amount", "value": "x"}]);
        assert!(validate_widgets_json_value(&bad).unwrap_err().to_string().contains("numeric"));
    }

    #[test]
    fn read_json_object_path_or_stdin_accepts_object_rejects_array() {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "feishu_approval_tool_obj_test_{}.json",
            std::process::id()
        ));
        std::fs::write(&p, r#"{"user_id":"x"}"#).unwrap();
        let m = read_json_object_path_or_stdin(&p).unwrap();
        assert_eq!(m.get("user_id").unwrap(), &json!("x"));
        std::fs::write(&p, r#"[1]"#).unwrap();
        assert!(read_json_object_path_or_stdin(&p).is_err());
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn read_json_path_or_stdin_reads_file() {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "feishu_approval_tool_extra_test_{}.json",
            std::process::id()
        ));
        std::fs::write(&p, r#"{"k":"v"}"#).unwrap();
        let v = read_json_path_or_stdin(&p).unwrap();
        assert_eq!(v, json!({"k": "v"}));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn extract_skeleton_from_string_or_array_form() {
        let string_form = json!({
            "form": r#"[{"id":"w1","type":"input","name":"Label","required":true,"option":[{"value":"a","text":"A"}]}]"#
        });
        let v = extract_widget_skeletons_value(&string_form).unwrap();
        let a = v.as_array().unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].get("id"), Some(&json!("w1")));
        assert!(a[0].get("options").is_some());

        let array_form = json!({"data": {"form": [{"id":"x","type":"textarea","name":"T"}]}});
        let v2 = extract_widget_skeletons_value(&array_form).unwrap();
        assert_eq!(v2.as_array().unwrap()[0]["type"], json!("textarea"));
    }

    #[test]
    fn validate_widgets_against_approval_data_ok_and_errors() {
        let approval = json!({
            "form": "[{\"id\":\"w1\",\"type\":\"input\",\"name\":\"x\",\"required\":true}]"
        });
        let ok = json!([{"id":"w1","type":"input","value":"hi"}]);
        validate_widgets_against_approval_data(&ok, &approval).unwrap();
        let bad_type = json!([{"id":"w1","type":"textarea","value":"hi"}]);
        assert!(validate_widgets_against_approval_data(&bad_type, &approval).is_err());
        let bad_id = json!([{"id":"nope","type":"input","value":"hi"}]);
        assert!(validate_widgets_against_approval_data(&bad_id, &approval).is_err());
    }

    #[test]
    fn skeleton_nested_children() {
        let approval = json!({
            "form": [{
                "id": "p",
                "type": "fieldList",
                "name": "T",
                "option": [],
                "children": [{"id":"c","type":"date","name":"D","required":true}]
            }]
        });
        let v = extract_widget_skeletons_value(&approval).unwrap();
        let ch = v[0]["children"].as_array().unwrap();
        assert_eq!(ch.len(), 1);
        assert_eq!(ch[0]["id"], json!("c"));
    }

    #[test]
    fn validate_widgets_matches_nested_definition_ids() {
        let approval = json!({"form": [{"id":"p","type":"fieldList","children":[{"id":"c","type":"date","name":"D"}]}]});
        let user = json!([{"id":"c","type":"date","value":"2024-01-01"}]);
        validate_widgets_against_approval_data(&user, &approval).unwrap();
    }

    #[test]
    fn scaffold_root_widgets_one_row_per_root() {
        let approval = json!({"form": [
            {"id":"a","type":"input","name":"A"},
            {"id":"b","type":"fieldList","name":"B","children":[{"id":"c","type":"date","name":"C"}]}
        ]});
        let v = scaffold_root_widgets_from_approval_data(&approval).unwrap();
        let a = v.as_array().unwrap();
        assert_eq!(a.len(), 2);
        assert_eq!(a[0], json!({"id":"a","type":"input","value": null}));
        assert_eq!(a[1]["id"], json!("b"));
        assert_eq!(a[1]["type"], json!("fieldList"));
        let rows = a[1]["value"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        let row0 = rows[0].as_array().unwrap();
        assert_eq!(row0.len(), 1);
        assert_eq!(
            row0[0],
            json!({"id":"c","type":"date","value": null})
        );
    }
}
