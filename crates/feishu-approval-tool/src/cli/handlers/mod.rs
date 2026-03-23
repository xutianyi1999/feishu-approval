//! HTTP dispatch for CLI subcommands.

mod approval;
mod comment;
mod instance;
mod task;
mod util;
mod wizard;

use super::exec::{exec, exec_file_upload};
use super::json_util::{form_string_from_widgets_json_path, read_json_path_or_stdin, read_string_file};
use super::{ApiAction, Cli, Command, FileAction};
use anyhow::{anyhow, Context, Result};
use feishu_approval_tool::{fetch_tenant_access_token, normalize_open_api_path};
use reqwest::Method;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn dispatch(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Token => {
            let token = fetch_tenant_access_token(&cli.base)?;
            println!("{token}");
        }
        Command::Approval { action } => approval::dispatch(cli, action)?,
        Command::Instance { action } => instance::dispatch(cli, action)?,
        Command::Task { action } => task::dispatch(cli, action)?,
        Command::Comment { action } => comment::dispatch(cli, action)?,
        Command::Subscribe { approval_code } => {
            let path = format!("/open-apis/approval/v4/approvals/{approval_code}/subscribe");
            exec(cli, Method::POST, &path, &[], None)?;
        }
        Command::Unsubscribe { approval_code } => {
            let path = format!("/open-apis/approval/v4/approvals/{approval_code}/unsubscribe");
            exec(cli, Method::POST, &path, &[], None)?;
        }
        Command::Api { action } => dispatch_api(cli, action)?,
        Command::File { action } => match action {
            FileAction::Upload { path, kind, name } => {
                exec_file_upload(
                    cli,
                    &cli.approval_upload_base,
                    path,
                    name.as_deref(),
                    kind.as_api_str(),
                )?;
            }
        },
        Command::Util { action } => util::dispatch(cli, action)?,
    }
    Ok(())
}

pub(super) fn post_json_from_file(cli: &Cli, path: &str, json_file: &Path) -> Result<()> {
    let body = read_json_path_or_stdin(json_file)?;
    exec(cli, Method::POST, path, &[], Some(&body))
}

pub(super) fn resolve_form_str(
    form: Option<&str>,
    form_file: Option<&Path>,
    err_if_none: &'static str,
) -> Result<String> {
    match (form, form_file) {
        (Some(s), None) => Ok(s.to_string()),
        (None, Some(p)) => read_string_file(p),
        (Some(_), Some(_)) => Err(anyhow!(
            "use only one of --form or --form-file (see docs/AI.md)"
        )),
        (None, None) => Err(anyhow!(err_if_none)),
    }
}

/// Exactly one of form / form_file / widgets_json_file (create instance).
pub(super) fn resolve_instance_create_form(
    form: &Option<String>,
    form_file: &Option<PathBuf>,
    widgets_json_file: &Option<PathBuf>,
) -> Result<String> {
    match widgets_json_file.as_ref() {
        Some(p) => {
            if form.is_some() || form_file.is_some() {
                return Err(anyhow!(
                    "--widgets-json-file cannot be used with --form or --form-file (see docs/AI.md)"
                ));
            }
            form_string_from_widgets_json_path(p)
        }
        None => resolve_form_str(
            form.as_deref(),
            form_file.as_ref().map(|x| x.as_path()),
            "provide exactly one of --form, --form-file, or --widgets-json-file (see docs/AI.md)",
        ),
    }
}

/// Borrow `Vec<(String,String)>` query pairs as `&str` slices for `exec`.
pub(super) fn query_vec_refs(q: &[(String, String)]) -> Vec<(&str, &str)> {
    q.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
}

pub(super) fn push_opt_query(q: &mut Vec<(String, String)>, key: &str, val: Option<&String>) {
    if let Some(v) = val {
        q.push((key.to_string(), v.clone()));
    }
}

fn parse_query(items: &[String]) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for s in items {
        let (k, v) = s
            .split_once('=')
            .with_context(|| format!("query must be KEY=value, got: {s}"))?;
        out.push((k.to_string(), v.to_string()));
    }
    Ok(out)
}

fn load_json_body(json: Option<&str>, json_file: Option<&PathBuf>) -> Result<Option<Value>> {
    match (json, json_file) {
        (None, None) => Ok(None),
        (Some(s), None) => {
            let v: Value = serde_json::from_str(s).context("invalid --json")?;
            Ok(Some(v))
        }
        (None, Some(p)) => Ok(Some(read_json_path_or_stdin(p)?)),
        (Some(_), Some(_)) => unreachable!("clap conflicts"),
    }
}

fn dispatch_api(cli: &Cli, action: &ApiAction) -> Result<()> {
    match action {
        ApiAction::Get { path, query } => {
            let path = normalize_open_api_path(path)?;
            let q = parse_query(query)?;
            let qref = query_vec_refs(&q);
            exec(cli, Method::GET, &path, &qref, None)?;
        }
        ApiAction::Post {
            path,
            query,
            json,
            json_file,
        } => {
            let path = normalize_open_api_path(path)?;
            let body = load_json_body(json.as_deref(), json_file.as_ref())?;
            let body = body.ok_or_else(|| {
                anyhow!("POST requires --json or --json-file (see docs/AI.md)")
            })?;
            let q = parse_query(query)?;
            let qref = query_vec_refs(&q);
            exec(cli, Method::POST, &path, &qref, Some(&body))?;
        }
        ApiAction::Delete { path, query } => {
            let path = normalize_open_api_path(path)?;
            let q = parse_query(query)?;
            let qref = query_vec_refs(&q);
            exec(cli, Method::DELETE, &path, &qref, None)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn parse_query_accepts_pairs_and_rejects_invalid() {
        let q = parse_query(&["a=b".into(), "c=d=e".into()]).unwrap();
        assert_eq!(
            q,
            vec![("a".into(), "b".into()), ("c".into(), "d=e".into())]
        );
        assert!(parse_query(&["noequals".into()]).is_err());
    }

    #[test]
    fn load_json_body_inline_file_and_invalid() {
        assert!(load_json_body(None, None).unwrap().is_none());
        let v = load_json_body(Some(r#"{"ok":true}"#), None).unwrap().unwrap();
        assert_eq!(v, json!({"ok": true}));
        assert!(load_json_body(Some("not json"), None).is_err());
        let p: PathBuf = std::env::temp_dir().join("feishu_approval_tool_load_json_test.json");
        fs::write(&p, r#"{"from":"file"}"#).unwrap();
        let v = load_json_body(None, Some(&p)).unwrap().unwrap();
        assert_eq!(v, json!({"from": "file"}));
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn resolve_instance_create_form_widgets_or_form() {
        let w = json!([{"id": "a", "type": "input", "value": "x"}]);
        let mut p = std::env::temp_dir();
        p.push(format!("feishu_widgets_{}.json", std::process::id()));
        fs::write(&p, serde_json::to_string(&w).unwrap()).unwrap();
        let s = resolve_instance_create_form(&None, &None, &Some(p.clone())).unwrap();
        assert!(s.starts_with('[') && s.contains("a"));
        let _ = fs::remove_file(&p);

        assert!(resolve_instance_create_form(&None, &None, &None).is_err());
        assert!(resolve_instance_create_form(
            &Some("x".into()),
            &None,
            &Some(PathBuf::from("nope"))
        )
        .is_err());
    }
}
