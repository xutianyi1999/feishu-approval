use crate::cli::output::print_body;
use anyhow::Result;
use feishu_approval_tool::OpenApiClient;
use reqwest::Method;
use serde_json::Value;
use std::path::Path;

use super::Cli;

pub fn exec(
    cli: &Cli,
    method: Method,
    path: &str,
    query: &[(&str, &str)],
    body: Option<&Value>,
) -> Result<()> {
    let client = OpenApiClient::from_env(&cli.base, cli.token.as_deref(), cli.timeout_secs)?;
    let text = client.request_json(method, path, query, body)?;
    print_body(&text, !cli.raw)?;
    Ok(())
}

pub fn exec_file_upload(
    cli: &Cli,
    upload_base: &str,
    file_path: &Path,
    name: Option<&str>,
    kind: &str,
) -> Result<()> {
    let client = OpenApiClient::from_env(&cli.base, cli.token.as_deref(), cli.timeout_secs)?;
    let text = client.upload_approval_file(upload_base, file_path, name, kind)?;
    print_body(&text, !cli.raw)?;
    Ok(())
}
