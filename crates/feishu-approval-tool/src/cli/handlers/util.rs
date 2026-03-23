use crate::cli::json_util::{
    apply_safe_widget_value_defaults, extract_widget_skeletons_value, form_string_from_widgets_json_path,
    read_json_path_or_stdin, scaffold_root_widgets_from_approval_data, validate_widgets_json_value,
};
use crate::cli::{Cli, UtilAction};
use anyhow::{Context, Result};
use feishu_approval_tool::{explain_feishu_widget_msg, resolve_tenant_token};
use std::fs;
use std::env;

pub fn dispatch(cli: &Cli, action: &UtilAction) -> Result<()> {
    match action {
        UtilAction::FormString { json_file } => {
            let s = form_string_from_widgets_json_path(json_file)?;
            println!("{s}");
        }
        UtilAction::ValidateWidgets { json_file, fix } => {
            let mut v = read_json_path_or_stdin(json_file)?;
            if *fix {
                apply_safe_widget_value_defaults(&mut v)?;
            }
            validate_widgets_json_value(&v)?;
            if *fix {
                let pretty = serde_json::to_string_pretty(&v)?;
                println!("{pretty}");
            } else {
                let n = v.as_array().map(|a| a.len()).unwrap_or(0);
                println!("OK: {n} widget(s) (offline checks passed)");
            }
        }
        UtilAction::Explain { msg } => {
            println!("{}", explain_feishu_widget_msg(msg));
        }
        UtilAction::ExtractWidgets { json_file } => {
            let root = read_json_path_or_stdin(json_file)?;
            let sk = extract_widget_skeletons_value(&root)?;
            let pretty = serde_json::to_string_pretty(&sk)?;
            println!("{pretty}");
        }
        UtilAction::ScaffoldWidgets { json_file } => {
            let root = read_json_path_or_stdin(json_file)?;
            let tpl = scaffold_root_widgets_from_approval_data(&root)?;
            let pretty = serde_json::to_string_pretty(&tpl)?;
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
        UtilAction::Init { output_dir } => {
            const MAP_EXAMPLE: &str = include_str!("../../../../../docs/approval-code-map.local.template.md");
            let path = output_dir.join("approval-code-map.local.md");
            if path.exists() {
                println!("{} already exists; not overwriting.", path.display());
            } else {
                fs::write(&path, MAP_EXAMPLE)
                    .with_context(|| format!("write {}", path.display()))?;
                println!(
                    "Wrote {} — edit the table; keep this file gitignored locally.",
                    path.display()
                );
            }
            println!("Next: set FEISHU_APP_ID + FEISHU_APP_SECRET (or FEISHU_TENANT_ACCESS_TOKEN), then `feishu-approval-tool util doctor`.");
        }
    }
    Ok(())
}
