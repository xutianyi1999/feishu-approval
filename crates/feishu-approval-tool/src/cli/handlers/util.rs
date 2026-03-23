use crate::cli::json_util::{
    extract_widget_skeletons_value, form_string_from_widgets_json_path, read_json_path_or_stdin,
    validate_widgets_json_value,
};
use crate::cli::{Cli, UtilAction};
use anyhow::Result;
use feishu_approval_tool::resolve_tenant_token;
use std::env;

pub fn dispatch(cli: &Cli, action: &UtilAction) -> Result<()> {
    match action {
        UtilAction::FormString { json_file } => {
            let s = form_string_from_widgets_json_path(json_file)?;
            println!("{s}");
        }
        UtilAction::ValidateWidgets { json_file } => {
            let v = read_json_path_or_stdin(json_file)?;
            validate_widgets_json_value(&v)?;
            let n = v.as_array().map(|a| a.len()).unwrap_or(0);
            println!("OK: {n} widget(s) (offline checks passed)");
        }
        UtilAction::ExtractWidgets { json_file } => {
            let root = read_json_path_or_stdin(json_file)?;
            let sk = extract_widget_skeletons_value(&root)?;
            let pretty = serde_json::to_string_pretty(&sk)?;
            println!("{pretty}");
        }
        UtilAction::Doctor => {
            println!("Open API base: {}", cli.base.trim_end_matches('/'));
            let has_cli_token = cli
                .token
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            let has_env_token = env::var("FEISHU_TENANT_ACCESS_TOKEN")
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            let has_app = env::var("FEISHU_APP_ID")
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false)
                && env::var("FEISHU_APP_SECRET")
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);

            if has_cli_token {
                println!("Auth source: CLI --token (value not printed)");
            } else if has_env_token {
                println!("Auth source: FEISHU_TENANT_ACCESS_TOKEN");
            } else if has_app {
                println!("Auth source: FEISHU_APP_ID + FEISHU_APP_SECRET (tenant token exchange)");
            } else {
                println!("Auth source: none (set CLI --token, or FEISHU_TENANT_ACCESS_TOKEN, or FEISHU_APP_ID+FEISHU_APP_SECRET)");
            }

            println!(
                "FEISHU_APP_ID / FEISHU_APP_SECRET: {}",
                if has_app { "both set" } else { "not both set" }
            );

            match resolve_tenant_token(&cli.base, cli.token.as_deref()) {
                Ok(_) => println!("Token resolve: OK (not printed)"),
                Err(e) => println!("Token resolve: FAIL — {e:#}"),
            }
        }
    }
    Ok(())
}
