use crate::cli::exec::exec;
use crate::cli::json_util::{
    form_api_string_from_array_value, merge_object, parse_json_object_from_str, read_json_path_or_stdin,
    validate_widgets_against_approval_data,
};
use super::{post_json_from_file, push_opt_query, query_vec_refs, resolve_instance_create_form};
use crate::cli::{Cli, InstanceAction, InstanceFormTemplate};
use anyhow::{anyhow, Context, Result};
use reqwest::Method;
use serde_json::{json, Map, Value};

pub fn dispatch(cli: &Cli, action: &InstanceAction) -> Result<()> {
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
            push_opt_query(&mut q, "page_size", page_size.as_ref());
            push_opt_query(&mut q, "page_token", page_token.as_ref());
            let qref = query_vec_refs(&q);
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
            widgets_json_file,
            wizard: use_wizard,
            template,
            validate_against_json,
            open_id,
            user_id,
            department_id,
            uuid,
            extra_json,
            extra_json_inline,
        } => {
            let form_str = if *use_wizard {
                super::wizard::interactive_form_string()?
            } else if let Some(tpl) = template {
                form_string_from_template(*tpl)?
            } else {
                resolve_instance_create_form(form, form_file, widgets_json_file)?
            };
            if let Some(p) = validate_against_json {
                let approval_root = read_json_path_or_stdin(p)?;
                let user_widgets: Value = serde_json::from_str(&form_str).context(
                    "resolved `form` must be a JSON array for --validate-against-json (use --widgets-json-file or form that serializes to a widget array)",
                )?;
                validate_widgets_against_approval_data(&user_widgets, &approval_root)?;
            }
            let (open_id, user_id) = if *use_wizard {
                super::wizard::ensure_open_id_or_user_id(open_id.as_ref(), user_id.as_ref())?
            } else {
                (open_id.clone(), user_id.clone())
            };
            if open_id.is_none() && user_id.is_none() {
                return Err(anyhow!(
                    "--open-id or --user-id is required (see docs/AI.md)"
                ));
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
                merge_object(&mut m, read_json_path_or_stdin(p)?)?;
            }
            if let Some(s) = extra_json_inline {
                merge_object(&mut m, parse_json_object_from_str(s)?)?;
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
            post_json_from_file(cli, "/open-apis/approval/v4/instances/query", json_file)?;
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
            post_json_from_file(cli, "/open-apis/approval/v4/instances/preview", json_file)?;
        }
        InstanceAction::SearchCc { json_file } => {
            post_json_from_file(cli, "/open-apis/approval/v4/instances/search_cc", json_file)?;
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

fn form_string_from_template(t: InstanceFormTemplate) -> Result<String> {
    const EXPENSE: &str = include_str!("../../../../../docs/examples/expense-reimbursement-widgets.sample.json");
    let raw = match t {
        InstanceFormTemplate::Expense => EXPENSE,
    };
    let v: Value = serde_json::from_str(raw).context("built-in expense template JSON")?;
    form_api_string_from_array_value(&v)
}
