use super::exec::{exec, exec_file_upload};
use super::json_util::{merge_object, read_json_file, read_string_file};
use super::{
    ApiAction, ApprovalAction, Cli, Command, CommentAction, FileAction, InstanceAction, TaskAction,
    TaskActOp,
};
use anyhow::{anyhow, Context, Result};
use feishu_approval_tool::{fetch_tenant_access_token, normalize_open_api_path};
use reqwest::Method;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

pub fn dispatch(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Token => {
            let token = fetch_tenant_access_token(&cli.base)?;
            println!("{token}");
        }
        Command::Approval { action } => match action {
            ApprovalAction::Get { approval_code } => {
                let path = format!("/open-apis/approval/v4/approvals/{approval_code}");
                exec(cli, Method::GET, &path, &[], None)?;
            }
        },
        Command::Instance { action } => dispatch_instance(cli, action)?,
        Command::Task { action } => dispatch_task(cli, action)?,
        Command::Comment { action } => dispatch_comment(cli, action)?,
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
    }
    Ok(())
}

fn dispatch_instance(cli: &Cli, action: &InstanceAction) -> Result<()> {
    match action {
        InstanceAction::Get { instance } => {
            let path = format!("/open-apis/approval/v4/instances/{instance}");
            exec(cli, Method::GET, &path, &[], None)?;
        }
        InstanceAction::List {
            approval_code,
            start_time,
            end_time,
            page_size,
            page_token,
        } => {
            let mut q: Vec<(String, String)> = vec![
                ("approval_code".into(), approval_code.clone()),
                ("start_time".into(), start_time.clone()),
                ("end_time".into(), end_time.clone()),
            ];
            if let Some(p) = page_size {
                q.push(("page_size".into(), p.clone()));
            }
            if let Some(p) = page_token {
                q.push(("page_token".into(), p.clone()));
            }
            let qref: Vec<(&str, &str)> = q.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            exec(
                cli,
                Method::GET,
                "/open-apis/approval/v4/instances",
                &qref,
                None,
            )?;
        }
        InstanceAction::Create {
            approval_code,
            form,
            form_file,
            open_id,
            user_id,
            department_id,
            uuid,
            extra_json,
        } => {
            let form_str = match (form, form_file) {
                (Some(s), None) => s.clone(),
                (None, Some(p)) => read_string_file(p)?,
                (Some(_), Some(_)) => return Err(anyhow!("use only one of --form or --form-file")),
                (None, None) => return Err(anyhow!("--form or --form-file is required")),
            };
            if open_id.is_none() && user_id.is_none() {
                return Err(anyhow!("--open-id or --user-id is required"));
            }
            let mut m = Map::new();
            m.insert("approval_code".into(), json!(approval_code));
            m.insert("form".into(), Value::String(form_str));
            if let Some(v) = open_id {
                m.insert("open_id".into(), json!(v));
            }
            if let Some(v) = user_id {
                m.insert("user_id".into(), json!(v));
            }
            if let Some(v) = department_id {
                m.insert("department_id".into(), json!(v));
            }
            if let Some(v) = uuid {
                m.insert("uuid".into(), json!(v));
            }
            if let Some(p) = extra_json {
                merge_object(&mut m, read_json_file(p)?)?;
            }
            let body = Value::Object(m);
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/instances",
                &[],
                Some(&body),
            )?;
        }
        InstanceAction::Query { json_file } => {
            let body = read_json_file(json_file)?;
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/instances/query",
                &[],
                Some(&body),
            )?;
        }
        InstanceAction::Cc {
            approval_code,
            instance_code,
            user_id,
            cc_user_id,
            comment,
            user_id_type,
        } => {
            if cc_user_id.is_empty() {
                return Err(anyhow!("at least one --cc-user-id is required"));
            }
            let mut m = Map::new();
            m.insert("approval_code".into(), json!(approval_code));
            m.insert("instance_code".into(), json!(instance_code));
            m.insert("user_id".into(), json!(user_id));
            m.insert("cc_user_ids".into(), json!(cc_user_id));
            if let Some(c) = comment {
                m.insert("comment".into(), json!(c));
            }
            let q = [("user_id_type", user_id_type.as_str())];
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/instances/cc",
                &q,
                Some(&Value::Object(m)),
            )?;
        }
        InstanceAction::Cancel {
            approval_code,
            instance_code,
            user_id,
            user_id_type,
        } => {
            let body = json!({
                "approval_code": approval_code,
                "instance_code": instance_code,
                "user_id": user_id,
            });
            let q = [("user_id_type", user_id_type.as_str())];
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/instances/cancel",
                &q,
                Some(&body),
            )?;
        }
        InstanceAction::Preview { json_file } => {
            let body = read_json_file(json_file)?;
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/instances/preview",
                &[],
                Some(&body),
            )?;
        }
        InstanceAction::SearchCc { json_file } => {
            let body = read_json_file(json_file)?;
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/instances/search_cc",
                &[],
                Some(&body),
            )?;
        }
        InstanceAction::SpecifiedRollback {
            user_id,
            task_id,
            task_def_key,
            reason,
            extra,
            user_id_type,
        } => {
            if task_def_key.is_empty() {
                return Err(anyhow!("at least one --task-def-key is required"));
            }
            let mut m = Map::new();
            m.insert("user_id".into(), json!(user_id));
            m.insert("task_id".into(), json!(task_id));
            m.insert("task_def_key_list".into(), json!(task_def_key));
            if let Some(r) = reason {
                m.insert("reason".into(), json!(r));
            }
            if let Some(e) = extra {
                m.insert("extra".into(), json!(e));
            }
            let q = [("user_id_type", user_id_type.as_str())];
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/instances/specified_rollback",
                &q,
                Some(&Value::Object(m)),
            )?;
        }
        InstanceAction::AddSign {
            approval_code,
            instance_code,
            user_id,
            task_id,
            add_sign_user_id,
            add_sign_type,
            comment,
            approval_method,
        } => {
            if add_sign_user_id.is_empty() {
                return Err(anyhow!("at least one --add-sign-user-id is required"));
            }
            let mut m = Map::new();
            m.insert("approval_code".into(), json!(approval_code));
            m.insert("instance_code".into(), json!(instance_code));
            m.insert("user_id".into(), json!(user_id));
            m.insert("task_id".into(), json!(task_id));
            m.insert("add_sign_user_ids".into(), json!(add_sign_user_id));
            m.insert("add_sign_type".into(), json!(add_sign_type));
            if let Some(c) = comment {
                m.insert("comment".into(), json!(c));
            }
            if let Some(am) = approval_method {
                m.insert("approval_method".into(), json!(am));
            }
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/instances/add_sign",
                &[],
                Some(&Value::Object(m)),
            )?;
        }
    }
    Ok(())
}

fn dispatch_task(cli: &Cli, action: &TaskAction) -> Result<()> {
    match action {
        TaskAction::Act {
            action,
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment,
            form,
            form_file,
            transfer_user_id,
            user_id_type,
        } => match action {
            TaskActOp::Approve => {
                task_approve_like(
                    cli,
                    "/open-apis/approval/v4/tasks/approve",
                    approval_code,
                    instance_code,
                    user_id,
                    task_id,
                    comment.as_deref(),
                    form.as_deref(),
                    None,
                    user_id_type,
                )?;
            }
            TaskActOp::Reject => {
                task_approve_like(
                    cli,
                    "/open-apis/approval/v4/tasks/reject",
                    approval_code,
                    instance_code,
                    user_id,
                    task_id,
                    comment.as_deref(),
                    form.as_deref(),
                    None,
                    user_id_type,
                )?;
            }
            TaskActOp::Transfer => {
                let tu = transfer_user_id
                    .as_deref()
                    .ok_or_else(|| anyhow!("task act transfer requires --transfer-user-id"))?;
                task_approve_like(
                    cli,
                    "/open-apis/approval/v4/tasks/transfer",
                    approval_code,
                    instance_code,
                    user_id,
                    task_id,
                    comment.as_deref(),
                    None,
                    Some(tu),
                    user_id_type,
                )?;
            }
            TaskActOp::Resubmit => {
                let form_str = match (form, form_file) {
                    (Some(s), None) => s.clone(),
                    (None, Some(p)) => read_string_file(p)?,
                    (Some(_), Some(_)) => return Err(anyhow!("use only one of --form or --form-file")),
                    (None, None) => {
                        return Err(anyhow!(
                            "task act resubmit requires --form or --form-file"
                        ));
                    }
                };
                task_approve_like(
                    cli,
                    "/open-apis/approval/v4/tasks/resubmit",
                    approval_code,
                    instance_code,
                    user_id,
                    task_id,
                    None,
                    Some(form_str.as_str()),
                    None,
                    user_id_type,
                )?;
            }
        },
        TaskAction::Approve {
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment,
            form,
            user_id_type,
        } => {
            task_approve_like(
                cli,
                "/open-apis/approval/v4/tasks/approve",
                approval_code,
                instance_code,
                user_id,
                task_id,
                comment.as_deref(),
                form.as_deref(),
                None,
                user_id_type,
            )?;
        }
        TaskAction::Reject {
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment,
            form,
            user_id_type,
        } => {
            task_approve_like(
                cli,
                "/open-apis/approval/v4/tasks/reject",
                approval_code,
                instance_code,
                user_id,
                task_id,
                comment.as_deref(),
                form.as_deref(),
                None,
                user_id_type,
            )?;
        }
        TaskAction::Transfer {
            approval_code,
            instance_code,
            user_id,
            task_id,
            transfer_user_id,
            comment,
            user_id_type,
        } => {
            task_approve_like(
                cli,
                "/open-apis/approval/v4/tasks/transfer",
                approval_code,
                instance_code,
                user_id,
                task_id,
                comment.as_deref(),
                None,
                Some(transfer_user_id.as_str()),
                user_id_type,
            )?;
        }
        TaskAction::Resubmit {
            approval_code,
            instance_code,
            user_id,
            task_id,
            form,
            form_file,
            user_id_type,
        } => {
            let form_str = match (form, form_file) {
                (Some(s), None) => s.clone(),
                (None, Some(p)) => read_string_file(p)?,
                (Some(_), Some(_)) => return Err(anyhow!("use only one of --form or --form-file")),
                (None, None) => return Err(anyhow!("--form or --form-file is required for resubmit")),
            };
            task_approve_like(
                cli,
                "/open-apis/approval/v4/tasks/resubmit",
                approval_code,
                instance_code,
                user_id,
                task_id,
                None,
                Some(form_str.as_str()),
                None,
                user_id_type,
            )?;
        }
        TaskAction::Search { json_file } => {
            let body = read_json_file(json_file)?;
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/tasks/search",
                &[],
                Some(&body),
            )?;
        }
        TaskAction::Query {
            user_id,
            topic,
            user_id_type,
            page_size,
            page_token,
        } => {
            let mut q: Vec<(String, String)> = vec![
                ("user_id".into(), user_id.clone()),
                ("topic".into(), topic.clone()),
                ("user_id_type".into(), user_id_type.clone()),
            ];
            if let Some(p) = page_size {
                q.push(("page_size".into(), p.clone()));
            }
            if let Some(p) = page_token {
                q.push(("page_token".into(), p.clone()));
            }
            let qref: Vec<(&str, &str)> = q.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            exec(
                cli,
                Method::GET,
                "/open-apis/approval/v4/tasks/query",
                &qref,
                None,
            )?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn task_approve_like(
    cli: &Cli,
    path: &'static str,
    approval_code: &str,
    instance_code: &str,
    user_id: &str,
    task_id: &str,
    comment: Option<&str>,
    form: Option<&str>,
    transfer_user_id: Option<&str>,
    user_id_type: &str,
) -> Result<()> {
    let mut m = Map::new();
    m.insert("approval_code".into(), json!(approval_code));
    m.insert("instance_code".into(), json!(instance_code));
    m.insert("user_id".into(), json!(user_id));
    m.insert("task_id".into(), json!(task_id));
    if let Some(c) = comment {
        m.insert("comment".into(), json!(c));
    }
    if let Some(f) = form {
        m.insert("form".into(), Value::String(f.to_string()));
    }
    if let Some(t) = transfer_user_id {
        m.insert("transfer_user_id".into(), json!(t));
    }
    let q = [("user_id_type", user_id_type)];
    exec(cli, Method::POST, path, &q, Some(&Value::Object(m)))?;
    Ok(())
}

fn dispatch_comment(cli: &Cli, action: &CommentAction) -> Result<()> {
    match action {
        CommentAction::List {
            instance,
            user_id,
            user_id_type,
            page_token,
            page_size,
        } => {
            let path = format!("/open-apis/approval/v4/instances/{instance}/comments");
            let mut q: Vec<(String, String)> = vec![
                ("user_id".into(), user_id.clone()),
                ("user_id_type".into(), user_id_type.clone()),
            ];
            if let Some(p) = page_token {
                q.push(("page_token".into(), p.clone()));
            }
            if let Some(p) = page_size {
                q.push(("page_size".into(), p.clone()));
            }
            let qref: Vec<(&str, &str)> = q.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            exec(cli, Method::GET, &path, &qref, None)?;
        }
        CommentAction::Create { instance, json_file } => {
            let path = format!("/open-apis/approval/v4/instances/{instance}/comments");
            let body = read_json_file(json_file)?;
            exec(cli, Method::POST, &path, &[], Some(&body))?;
        }
        CommentAction::Delete {
            instance,
            comment_id,
            user_id,
            user_id_type,
        } => {
            let path = format!("/open-apis/approval/v4/instances/{instance}/comments/{comment_id}");
            let q = [
                ("user_id_type", user_id_type.as_str()),
                ("user_id", user_id.as_str()),
            ];
            exec(cli, Method::DELETE, &path, &q, None)?;
        }
        CommentAction::Remove { instance, json_file } => {
            let path = format!("/open-apis/approval/v4/instances/{instance}/comments/remove");
            let body = read_json_file(json_file)?;
            exec(cli, Method::POST, &path, &[], Some(&body))?;
        }
    }
    Ok(())
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
        (None, Some(p)) => Ok(Some(read_json_file(p)?)),
        (Some(_), Some(_)) => unreachable!("clap conflicts"),
    }
}

fn dispatch_api(cli: &Cli, action: &ApiAction) -> Result<()> {
    match action {
        ApiAction::Get { path, query } => {
            let path = normalize_open_api_path(path)?;
            let q = parse_query(query)?;
            let qref: Vec<(&str, &str)> = q.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
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
            let body = body.ok_or_else(|| anyhow!("POST requires --json or --json-file"))?;
            let q = parse_query(query)?;
            let qref: Vec<(&str, &str)> = q.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            exec(cli, Method::POST, &path, &qref, Some(&body))?;
        }
        ApiAction::Delete { path, query } => {
            let path = normalize_open_api_path(path)?;
            let q = parse_query(query)?;
            let qref: Vec<(&str, &str)> = q.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
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
        assert_eq!(v, json!({"from":"file"}));
        let _ = fs::remove_file(&p);
    }
}
