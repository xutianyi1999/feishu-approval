use crate::cli::exec::exec;
use super::{post_json_from_file, query_vec_refs, resolve_form_str};
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
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment,
            form,
            user_id_type,
        } => task_act_dispatch(
            cli,
            TaskActOp::Reject,
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
        TaskAction::Search { json_file } => {
            post_json_from_file(cli, "/open-apis/approval/v4/tasks/search", json_file)?;
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
        TaskActOp::Approve => approve_like(
            cli,
            "/open-apis/approval/v4/tasks/approve",
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment,
            form.as_deref(),
            None,
            user_id_type,
        ),
        TaskActOp::Reject => approve_like(
            cli,
            "/open-apis/approval/v4/tasks/reject",
            approval_code,
            instance_code,
            user_id,
            task_id,
            comment,
            form.as_deref(),
            None,
            user_id_type,
        ),
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
