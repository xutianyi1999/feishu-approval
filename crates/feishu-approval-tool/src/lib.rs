//! Feishu Open Platform HTTP client for Approval API v4 and auth.
//! The `feishu-approval-tool` binary adds workflow-oriented CLI subcommands (`approval`, `instance`, `task`, `comment`, `file`, etc.) on top of this client.
//! Native approval **image/attachment** widgets need a file `code` from `POST …/approval/openapi/v2/file/upload` ([`OpenApiClient::upload_approval_file`]).

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
    #[allow(dead_code)]
    expire: Option<i64>,
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
        let _ = OpenApiClient::new(" https://x.com/ ", "t", 0).unwrap();
        let c = OpenApiClient::new("https://x.com", "t", 5).unwrap();
        let err = c
            .request_json(Method::GET, "/v1/foo", &[], None)
            .unwrap_err();
        assert!(err.to_string().contains("/open-apis/"));
    }
}
