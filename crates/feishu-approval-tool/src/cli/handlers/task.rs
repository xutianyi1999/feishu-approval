use crate::cli::exec::exec;
use crate::cli::json_util::read_json_object_path_or_stdin;
use crate::cli::json_util::read_json_path_or_stdin;
use super::{push_opt_query, query_vec_refs, resolve_form_str};
use crate::cli::{Cli, TaskAction, TaskActOp};
use anyhow::{anyhow, Result};
use reqwest::Method;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

pub fn dispatch(cli: &Cli, action: &TaskAction) -> Result<()> {
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
        } => task_act_dispatch(
            cli,
            *action,
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment.as_deref(),
            form,
            form_file,
            transfer_user_id.as_deref(),
            user_id_type,
            "task act resubmit requires --form or --form-file",
        )?,
        TaskAction::Approve {
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment,
            form,
            user_id_type,
        } => task_act_dispatch(
            cli,
            TaskActOp::Approve,
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment.as_deref(),
            form,
            &None,
            None,
            user_id_type,
            "",
        )?,
        TaskAction::Reject {
            task_id,
            task_ids,
            batch_json_file,
            approval_code,
            instance_code,
            user_id,
            comment,
            form,
            user_id_type,
        } => dispatch_task_reject(
            cli,
            task_id.as_deref(),
            task_ids.as_deref(),
            batch_json_file.as_ref(),
            approval_code.as_deref(),
            instance_code.as_deref(),
            user_id.as_deref(),
            comment.as_deref(),
            form.as_deref(),
            user_id_type,
        )?,
        TaskAction::Transfer {
            approval_code,
            instance_code,
            user_id,
            task_id,
            transfer_user_id,
            comment,
            user_id_type,
        } => task_act_dispatch(
            cli,
            TaskActOp::Transfer,
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment.as_deref(),
            &None,
            &None,
            Some(transfer_user_id.as_str()),
            user_id_type,
            "",
        )?,
        TaskAction::Resubmit {
            approval_code,
            instance_code,
            user_id,
            task_id,
            form,
            form_file,
            user_id_type,
        } => task_act_dispatch(
            cli,
            TaskActOp::Resubmit,
            approval_code,
            instance_code,
            user_id,
            task_id,
            None,
            form,
            form_file,
            None,
            user_id_type,
            "--form or --form-file is required for resubmit",
        )?,
        TaskAction::Search {
            json_file,
            task_status,
            pending_only,
            search_user_id,
        } => {
            let mut body = read_json_object_path_or_stdin(json_file)?;
            if *pending_only {
                body.insert("task_status".into(), json!("PENDING"));
            } else if let Some(s) = task_status {
                body.insert("task_status".into(), json!(s));
            }
            if let Some(u) = search_user_id {
                body.insert("user_id".into(), json!(u));
            }
            let v = Value::Object(body);
            exec(
                cli,
                Method::POST,
                "/open-apis/approval/v4/tasks/search",
                &[],
                Some(&v),
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
            push_opt_query(&mut q, "page_size", page_size.as_ref());
            push_opt_query(&mut q, "page_token", page_token.as_ref());
            let qref = query_vec_refs(&q);
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
fn dispatch_task_reject(
    cli: &Cli,
    task_id: Option<&str>,
    task_ids: Option<&str>,
    batch_path: Option<&PathBuf>,
    approval_code: Option<&str>,
    instance_code: Option<&str>,
    user_id: Option<&str>,
    comment: Option<&str>,
    form: Option<&str>,
    user_id_type: &str,
) -> Result<()> {
    let path = "/open-apis/approval/v4/tasks/reject";
    if let Some(p) = batch_path {
        let v = read_json_path_or_stdin(p)?;
        let arr = v
            .as_array()
            .ok_or_else(|| anyhow!("--batch-json-file must be a JSON array [...]"))?;
        for (i, item) in arr.iter().enumerate() {
            let o = item
                .as_object()
                .ok_or_else(|| anyhow!("batch[{i}]: expected object"))?;
            let ac = o
                .get("approval_code")
                .and_then(|x| x.as_str())
                .ok_or_else(|| anyhow!("batch[{i}]: missing string approval_code"))?;
            let ic = o
                .get("instance_code")
                .and_then(|x| x.as_str())
                .ok_or_else(|| anyhow!("batch[{i}]: missing string instance_code"))?;
            let uid = o
                .get("user_id")
                .and_then(|x| x.as_str())
                .ok_or_else(|| anyhow!("batch[{i}]: missing string user_id"))?;
            let tid = o
                .get("task_id")
                .and_then(|x| x.as_str())
                .ok_or_else(|| anyhow!("batch[{i}]: missing string task_id"))?;
            let row_comment = o
                .get("comment")
                .and_then(|x| x.as_str())
                .or(comment);
            let row_form = o
                .get("form")
                .and_then(|x| x.as_str())
                .or(form);
            approve_like(
                cli,
                path,
                ac,
                ic,
                uid,
                tid,
                row_comment,
                row_form,
                None,
                user_id_type,
            )?;
        }
        return Ok(());
    }

    let ids: Vec<String> = if let Some(s) = task_ids {
        split_task_ids(s)
    } else if let Some(t) = task_id {
        vec![t.to_string()]
    } else {
        return Err(anyhow!("provide --task-id, --task-ids, or --batch-json-file"));
    };
    if ids.is_empty() {
        return Err(anyhow!("--task-ids produced no task ids"));
    }
    let ac = approval_code
        .ok_or_else(|| anyhow!("--approval-code is required unless using --batch-json-file"))?;
    let ic = instance_code
        .ok_or_else(|| anyhow!("--instance-code is required unless using --batch-json-file"))?;
    let uid =
        user_id.ok_or_else(|| anyhow!("--user-id is required unless using --batch-json-file"))?;
    for tid in ids {
        approve_like(
            cli,
            path,
            ac,
            ic,
            uid,
            &tid,
            comment,
            form,
            None,
            user_id_type,
        )?;
    }
    Ok(())
}

fn split_task_ids(s: &str) -> Vec<String> {
    s.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn task_act_dispatch(
    cli: &Cli,
    op: TaskActOp,
    approval_code: &str,
    instance_code: &str,
    user_id: &str,
    task_id: &str,
    comment: Option<&str>,
    form: &Option<String>,
    form_file: &Option<PathBuf>,
    transfer_user_id: Option<&str>,
    user_id_type: &str,
    resubmit_form_err: &'static str,
) -> Result<()> {
    match op {
        TaskActOp::Approve | TaskActOp::Reject => {
            let path = match op {
                TaskActOp::Approve => "/open-apis/approval/v4/tasks/approve",
                TaskActOp::Reject => "/open-apis/approval/v4/tasks/reject",
                _ => unreachable!(),
            };
            approve_like(
                cli,
                path,
                approval_code,
                instance_code,
                user_id,
                task_id,
                comment,
                form.as_deref(),
                None,
                user_id_type,
            )
        }
        TaskActOp::Transfer => {
            let tu = transfer_user_id
                .ok_or_else(|| anyhow!("task act transfer requires --transfer-user-id"))?;
            approve_like(
                cli,
                "/open-apis/approval/v4/tasks/transfer",
                approval_code,
                instance_code,
                user_id,
                task_id,
                comment,
                None,
                Some(tu),
                user_id_type,
            )
        }
        TaskActOp::Resubmit => {
            let form_str = resolve_form_str(
                form.as_deref(),
                form_file.as_ref().map(|p| p.as_path()),
                resubmit_form_err,
            )?;
            approve_like(
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
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn approve_like(
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

#[cfg(test)]
mod tests {
    use super::split_task_ids;

    #[test]
    fn split_task_ids_trims_and_skips_empty() {
        assert_eq!(
            split_task_ids(" a , b , "),
            vec!["a".to_string(), "b".to_string()]
        );
        assert!(split_task_ids("  ,  ").is_empty());
    }
}
