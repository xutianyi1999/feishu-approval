//! Feishu Open Platform HTTP client for Approval API v4 and auth.
//! The `feishu-approval-tool` binary adds workflow-oriented CLI subcommands (`approval`, `instance`, `task`, `comment`, `file`, etc.) on top of this client.
//! Native approval **image/attachment** widgets need a file `code` from `POST …/approval/openapi/v2/file/upload` ([`OpenApiClient::upload_approval_file`]).

pub mod api_paths;

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::Method;
use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::path::Path;
use std::time::Duration;

/// Normalize CLI path to an `/open-apis/...` path.
/// Accepts `/open-apis/approval/v4/...`, `approval/v4/...`, or `open-apis/...`.
pub fn normalize_open_api_path(path: &str) -> Result<String> {
    let p = path.trim();
    if p.starts_with("http://") || p.starts_with("https://") {
        return Err(anyhow!(
            "pass only the path (e.g. /open-apis/approval/v4/...), not a full URL"
        ));
    }
    if p.starts_with("/open-apis/") {
        return Ok(p.to_string());
    }
    if p.starts_with("open-apis/") {
        return Ok(format!("/{}", p.trim_start_matches('/')));
    }
    let rest = p.trim_start_matches('/');
    Ok(format!("/open-apis/{rest}"))
}

#[derive(Deserialize)]
struct TokenResponse {
    code: Option<i64>,
    msg: Option<String>,
    tenant_access_token: Option<String>,
}

/// Exchange app credentials for `tenant_access_token`.
pub fn fetch_tenant_access_token(base: &str) -> Result<String> {
    let app_id = env::var("FEISHU_APP_ID").context("FEISHU_APP_ID is not set")?;
    let app_secret = env::var("FEISHU_APP_SECRET").context("FEISHU_APP_SECRET is not set")?;

    let base = base.trim_end_matches('/');
    let url = format!("{base}/open-apis/auth/v3/tenant_access_token/internal");

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let body = serde_json::json!({
        "app_id": app_id,
        "app_secret": app_secret,
    });

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json; charset=utf-8")
        .json(&body)
        .send()
        .with_context(|| format!("POST {url}"))?;

    let status = resp.status();
    let text = resp.text().context("read token response body")?;
    let parsed: TokenResponse = serde_json::from_str(&text)
        .with_context(|| format!("token response is not JSON (HTTP {status}): {text}"))?;

    if let Some(c) = parsed.code {
        if c != 0 {
            let msg = parsed.msg.unwrap_or_default();
            return Err(anyhow!("Feishu error code {c}: {msg}"));
        }
    }

    parsed
        .tenant_access_token
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("missing tenant_access_token in response: {text}"))
}

/// Resolved bearer token: explicit env/arg, otherwise fetched with app credentials.
pub fn resolve_tenant_token(base: &str, override_token: Option<&str>) -> Result<String> {
    if let Some(t) = override_token {
        let t = t.trim();
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    if let Ok(t) = env::var("FEISHU_TENANT_ACCESS_TOKEN") {
        let t = t.trim();
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    fetch_tenant_access_token(base)
}

/// Minimal Open API caller: `Authorization: Bearer`, JSON bodies, query pairs.
pub struct OpenApiClient {
    http: Client,
    base: String,
    token: String,
}

impl OpenApiClient {
    pub fn new(base: impl Into<String>, token: impl Into<String>, timeout_secs: u64) -> Result<Self> {
        let base = base.into().trim_end_matches('/').to_string();
        let token = token.into();
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_secs.max(1)))
            .build()?;
        Ok(Self { http, base, token })
    }

    pub fn from_env(base: &str, token_override: Option<&str>, timeout_secs: u64) -> Result<Self> {
        let token = resolve_tenant_token(base, token_override)?;
        Self::new(base, token, timeout_secs)
    }

    /// `path` must be `/open-apis/...` (use [`normalize_open_api_path`]).
    pub fn request_json(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&Value>,
    ) -> Result<String> {
        if !path.starts_with("/open-apis/") {
            return Err(anyhow!("internal: path must start with /open-apis/"));
        }
        let url = format!("{}{}", self.base, path);

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))
                .map_err(|e| anyhow!("invalid token for header: {e}"))?,
        );

        let mut req = self.http.request(method.clone(), &url).headers(headers);

        for (k, v) in query {
            req = req.query(&[(*k, *v)]);
        }

        if body.is_some() {
            req = req.header(CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"));
        }

        let req = if let Some(b) = body {
            req.json(b)
        } else {
            req
        };

        let resp = req.send().with_context(|| format!("{method} {url}"))?;
        let status = resp.status();
        let text = resp.text().context("read response body")?;

        if !status.is_success() {
            return Err(anyhow!("HTTP {}: {}", status, text));
        }

        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(c) = v.get("code").and_then(|x| x.as_i64()) {
                if c != 0 {
                    let msg = v
                        .get("msg")
                        .and_then(|m| m.as_str())
                        .unwrap_or("")
                        .to_string();
                    let augmented = augment_feishu_open_api_error(&msg);
                    return Err(anyhow!(
                        "Feishu API error {c}: {msg}{augmented}\n\nResponse JSON:\n{text}"
                    ));
                }
            }
        }

        Ok(text)
    }

    /// Upload one file for native approval **image** or **attachment** widgets.
    ///
    /// Calls `{upload_base}/approval/openapi/v2/file/upload` with `multipart/form-data`
    /// (`name`, `type`, `content`). This host is separate from the Open Platform base URL.
    pub fn upload_approval_file(
        &self,
        upload_base: &str,
        file_path: &Path,
        name: Option<&str>,
        kind: &str,
    ) -> Result<String> {
        if kind != "image" && kind != "attachment" {
            return Err(anyhow!("file type must be 'image' or 'attachment'"));
        }
        let upload_base = upload_base.trim_end_matches('/');
        let url = format!("{upload_base}/approval/openapi/v2/file/upload");

        let display_name: String = if let Some(n) = name {
            n.trim().to_string()
        } else {
            file_path
                .file_name()
                .and_then(|s| s.to_str())
                .map(String::from)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| anyhow!("cannot derive file name; pass an explicit name"))?
        };
        if display_name.is_empty() {
            return Err(anyhow!("file name must not be empty"));
        }

        let file_part = multipart::Part::file(file_path)
            .with_context(|| format!("read file {:?}", file_path))?
            .file_name(display_name.clone());

        let form = multipart::Form::new()
            .text("name", display_name)
            .text("type", kind.to_string())
            .part("content", file_part);

        let resp = self
            .http
            .post(&url)
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.token))
                    .map_err(|e| anyhow!("invalid token for header: {e}"))?,
            )
            .multipart(form)
            .send()
            .with_context(|| format!("POST {url}"))?;

        let status = resp.status();
        let text = resp.text().context("read response body")?;

        if !status.is_success() {
            return Err(anyhow!("HTTP {}: {}", status, text));
        }

        #[derive(Deserialize)]
        struct UploadTop {
            code: Option<i64>,
            msg: Option<String>,
        }
        if let Ok(top) = serde_json::from_str::<UploadTop>(&text) {
            if let Some(c) = top.code {
                if c != 0 {
                    let msg = top.msg.unwrap_or_default();
                    return Err(anyhow!("Feishu error code {c}: {msg}"));
                }
            }
        }

        Ok(text)
    }
}

/// Offline explanation for a Feishu Open API `msg` about forms/widgets. Used by **`util explain`**
/// and appended after JSON API errors when `code != 0`.
pub fn explain_feishu_widget_msg(msg: &str) -> String {
    let mut out = String::new();
    out.push_str(
        " If this call involved `form` / widgets: **`docs/AI.md` §7** and (for HTTP field shapes) one page under **`embedded-docs/.../approval-instance-form-control-parameters.md`**.",
    );

    let lower = msg.to_lowercase();
    if msg.contains("不是数组") || lower.contains("not an array") {
        out.push_str(
            "\n\n**Expected a JSON array** for that widget `value`, but the payload was another JSON type.",
        );
        if msg.contains("attachmentV2")
            || msg.contains("attachment")
            || msg.contains("image")
            || lower.contains("attachmentv2")
        {
            out.push_str(
                "\n- **image / attachment / attachmentV2**: `value` must be a JSON array. Empty: `[]`. After **`feishu-approval-tool file upload`**, use e.g. `[{{\"code\":\"FILE_CODE_FROM_UPLOAD\"}}]` (exact shape must match your definition; see §7).",
            );
        }
        if msg.contains("fieldList") || msg.contains("明细") {
            out.push_str(
                "\n- **fieldList**: `value` must be **two-dimensional**: `[ [ {{\"id\",…,\"type\",…,\"value\"}}, … ], … ]` (rows × column cells).",
            );
        }
    }

    if (msg.contains("日期") || msg.contains("时间")) && (msg.contains("格式") || msg.contains("无效") || lower.contains("invalid")) {
        out.push_str(
            "\n\n**Date/time**: for `type: \"date\"`, use an **RFC3339 string with `Z` or `±HH:MM`**, e.g. **`2026-03-22T09:00:00+08:00`** (not `YYYY-MM-DD` alone, not a Unix timestamp number).",
        );
    }

    if feishu_msg_sounds_like_form_widget_issue(msg) {
        out.push_str(
            "\n\n**Common fixes**:\n- **`date`** → RFC3339 string as above.\n- **`fieldList`** → 2D array; **each inner cell** needs **`id` + `type` + `value`** (a missing inner `type` often shows as 「控件类型为空」「index=0」 and does **not** always mean the first root widget).\n- **`formula`** → non-empty string `value` is often required at create.\n- Run **`feishu-approval-tool util validate-widgets --json-file widgets.json`** (paths like `widgets[i].value[r][c]`). Add **`--fix`** to apply safe defaults for `null` on file widgets / `fieldList` / some text types.\n- Match widget **`type`** from **`approval dump --data-only`** + **`util extract-widgets`**.",
        );
    } else if msg.contains("formula") || msg.contains("公式") {
        out.push_str(
            "\n\n**formula widget**: the API often requires a **non-empty string** `value` (e.g. same as a related amount); an empty `value` often surfaces as a generic widget error.",
        );
    }

    if msg.contains("index=") || msg.contains("index =") {
        out.push_str(
            "\n\n**About `index` in the message**: it may refer to a **flattened** widget index, not “the first control you see”. If root widgets look fine, inspect **`fieldList` inner cells** (missing `type`, wrong `value` shape).",
        );
    }

    out.push_str(
        "\n\n**Command**: paste the same text with **`feishu-approval-tool util explain --msg \"…\"`** to print this style of guide offline.",
    );
    out
}

fn augment_feishu_open_api_error(msg: &str) -> String {
    format!(
        "\n\n---\n[feishu-approval-tool]{}",
        explain_feishu_widget_msg(msg)
    )
}

fn feishu_msg_sounds_like_form_widget_issue(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    msg.contains("控件")
        || msg.contains("表单")
        || msg.contains("不合法")
        || msg.contains("为空")
        || msg.contains("不是数组")
        || lower.contains("not an array")
        || lower.contains("param is invalid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Method;

    #[test]
    fn normalize_open_api_path_accepts_valid_and_rejects_urls() {
        assert_eq!(
            normalize_open_api_path("/open-apis/approval/v4/approvals/x").unwrap(),
            "/open-apis/approval/v4/approvals/x"
        );
        assert_eq!(
            normalize_open_api_path("approval/v4/approvals/x").unwrap(),
            "/open-apis/approval/v4/approvals/x"
        );
        assert_eq!(
            normalize_open_api_path("open-apis/approval/v4/x").unwrap(),
            "/open-apis/approval/v4/x"
        );
        assert_eq!(
            normalize_open_api_path("  /open-apis/foo  ").unwrap(),
            "/open-apis/foo"
        );
        assert!(normalize_open_api_path("https://x").is_err());
        assert!(normalize_open_api_path("http://open.feishu.cn/open-apis/x").is_err());
    }

    #[test]
    fn upload_approval_file_rejects_invalid_widget_type() {
        let dir = std::env::temp_dir();
        let p = dir.join("feishu_approval_upload_kind_test.txt");
        std::fs::write(&p, b"x").unwrap();
        let c = OpenApiClient::new("https://example.com", "t", 5).unwrap();
        let err = c
            .upload_approval_file("https://example.com", &p, None, "document")
            .unwrap_err();
        assert!(err.to_string().contains("image"), "{err}");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn resolve_tenant_token_override_and_open_api_client() {
        let t = resolve_tenant_token("https://unused.example", Some("  bearer-value  ")).unwrap();
        assert_eq!(t, "bearer-value");
        let c = OpenApiClient::new("https://x.com", "t", 5).unwrap();
        let err = c
            .request_json(Method::GET, "/v1/foo", &[], None)
            .unwrap_err();
        assert!(err.to_string().contains("/open-apis/"));
    }

    #[test]
    fn augment_feishu_error_adds_cheatsheet_for_widget_msgs() {
        let s = super::augment_feishu_open_api_error("控件值不合法或者为空");
        assert!(s.contains("docs/AI.md"));
        assert!(s.contains("RFC3339"), "{s}");
    }

    #[test]
    fn augment_feishu_error_fieldlist_inner_type_hint_for_vague_index_msg() {
        let s = super::augment_feishu_open_api_error("表单控件类型为空。index= 0");
        assert!(s.contains("fieldList"), "{s}");
        assert!(s.contains("inner"), "{s}");
    }

    #[test]
    fn explain_msg_attachmentv2_not_array() {
        let s = super::explain_feishu_widget_msg(
            "控件值不是数组。index= 4, widget type= attachmentV2.",
        );
        assert!(s.contains("attachmentV2") || s.contains("attachment"), "{s}");
        assert!(s.contains("[]"), "{s}");
        assert!(s.contains("validate-widgets"), "{s}");
    }

    #[test]
    fn augment_keeps_heading_for_explain_style() {
        let s = super::augment_feishu_open_api_error("控件值不合法或者为空");
        assert!(s.contains("---"), "{s}");
        assert!(s.contains("docs/AI.md"), "{s}");
    }

    #[test]
    fn augment_feishu_error_short_for_unrelated_msg() {
        let s = super::augment_feishu_open_api_error("rate limit");
        assert!(s.contains("docs/AI.md"));
        assert!(!s.contains("RFC3339"), "{s}");
    }

    #[test]
    fn augment_feishu_error_formula_hint_without_chinese_widget_msg() {
        let s = super::augment_feishu_open_api_error("formula value invalid");
        assert!(s.contains("formula"));
        assert!(s.contains("non-empty"), "{s}");
    }
}
