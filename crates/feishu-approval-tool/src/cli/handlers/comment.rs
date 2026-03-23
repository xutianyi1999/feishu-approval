use crate::cli::exec::exec;
use super::{post_json_from_file, push_opt_query, query_vec_refs};
use crate::cli::{Cli, CommentAction};
use anyhow::Result;
use feishu_approval_tool::api_paths;
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
            let path = api_paths::instance_comments(instance);
            let mut q: Vec<(String, String)> = vec![
                ("user_id".into(), user_id.clone()),
                ("user_id_type".into(), user_id_type.clone()),
            ];
            push_opt_query(&mut q, "page_token", page_token.as_ref());
            push_opt_query(&mut q, "page_size", page_size.as_ref());
            let qref = query_vec_refs(&q);
            exec(cli, Method::GET, &path, &qref, None)?;
        }
        CommentAction::Create { instance, json_file } => {
            let path = api_paths::instance_comments(instance);
            post_json_from_file(cli, &path, json_file)?;
        }
        CommentAction::Delete {
            instance,
            comment_id,
            user_id,
            user_id_type,
        } => {
            let path = api_paths::instance_comment(instance, comment_id);
            let q = [
                ("user_id_type", user_id_type.as_str()),
                ("user_id", user_id.as_str()),
            ];
            exec(cli, Method::DELETE, &path, &q, None)?;
        }
        CommentAction::Remove { instance, json_file } => {
            let path = api_paths::instance_comments_remove(instance);
            post_json_from_file(cli, &path, json_file)?;
        }
    }
    Ok(())
}
