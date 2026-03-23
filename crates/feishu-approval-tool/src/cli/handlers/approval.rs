use crate::cli::exec::exec;
use crate::cli::{ApprovalAction, Cli};
use anyhow::{anyhow, Context, Result};
use feishu_approval_tool::api_paths;
use feishu_approval_tool::OpenApiClient;
use reqwest::Method;
use serde_json::Value;
use std::fs;

pub fn dispatch(cli: &Cli, action: &ApprovalAction) -> Result<()> {
    match action {
        ApprovalAction::Get { approval_code } => {
            let path = api_paths::approval_definition(approval_code);
            exec(cli, Method::GET, &path, &[], None)?;
        }
        ApprovalAction::Dump {
            approval_code,
            output,
            data_only,
        } => {
            let client =
                OpenApiClient::from_env(&cli.base, cli.token.as_deref(), cli.timeout_secs)?;
            let path = api_paths::approval_definition(approval_code);
            let text = client.request_json(Method::GET, &path, &[], None)?;
            let v: Value = serde_json::from_str(&text).context("approval response is not JSON")?;
            let out_val = if *data_only {
                v.get("data")
                    .cloned()
                    .ok_or_else(|| anyhow!("response has no `data` field; omit --data-only"))?
            } else {
                v
            };
            let pretty = serde_json::to_string_pretty(&out_val)?;
            let out = format!("{pretty}\n");
            match output {
                None => print!("{out}"),
                Some(p) => fs::write(p, &out).with_context(|| format!("write {}", p.display()))?,
            }
        }
    }
    Ok(())
}
