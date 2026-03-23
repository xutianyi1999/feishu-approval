use crate::cli::exec::exec;
use super::{post_json_from_file, query_vec_refs};
use crate::cli::{Cli, CommentAction};
use anyhow::Result;
use reqwest::Method;

pub fn dispatch(cli: &Cli, action: &CommentAction) -> Result<()> {
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
            let qref = query_vec_refs(&q);
            exec(cli, Method::GET, &path, &qref, None)?;
        }
        CommentAction::Create { instance, json_file } => {
            let path = format!("/open-apis/approval/v4/instances/{instance}/comments");
            post_json_from_file(cli, &path, json_file)?;
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
            post_json_from_file(cli, &path, json_file)?;
        }
    }
    Ok(())
}
