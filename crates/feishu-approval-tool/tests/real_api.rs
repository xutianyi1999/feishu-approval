//! Live Feishu Approval API v4 integration test (single `#[test]`, no mocks, no skips).
//! Requires `FEISHU_APP_ID`/`FEISHU_APP_SECRET`, `FEISHU_TEST_USER_ID`, optional `FEISHU_OPEN_BASE`,
//! `FEISHU_TEST_USER_ID_TYPE` (default `open_id`), `FEISHU_TEST_TIMEOUT_SECS` (default `120`).
//! Creates one API-defined approval and two instances (test tenant only).

use feishu_approval_tool::{fetch_tenant_access_token, OpenApiClient};
use reqwest::Method;
use serde_json::{json, Map, Value};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

fn load_dotenv() {
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_filename(format!("{}/../../.env", env!("CARGO_MANIFEST_DIR")));
}

fn base_url() -> String {
    std::env::var("FEISHU_OPEN_BASE").unwrap_or_else(|_| "https://open.feishu.cn".into())
}

fn timeout_secs() -> u64 {
    std::env::var("FEISHU_TEST_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(120)
        .max(10)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_millis() as i64
}

fn make_client() -> OpenApiClient {
    let base = base_url();
    let token = fetch_tenant_access_token(&base).expect("tenant token");
    OpenApiClient::new(&base, token, timeout_secs()).expect("OpenApiClient")
}

fn parse_ok(text: &str, ctx: &str) -> Value {
    let v: Value = serde_json::from_str(text).unwrap_or_else(|e| panic!("{ctx}: invalid JSON: {e}\n{text}"));
    assert_eq!(v.get("code").and_then(|x| x.as_i64()).unwrap_or(-1), 0, "{ctx}: {v}");
    v
}

fn api(
    c: &OpenApiClient,
    method: Method,
    path: &str,
    query: &[(&str, &str)],
    body: Option<&Value>,
    ctx: &str,
) -> Value {
    let text = c
        .request_json(method, path, query, body)
        .unwrap_or_else(|e| panic!("{ctx}: {e}"));
    parse_ok(&text, ctx)
}

fn require_test_user() -> (String, String) {
    let uid = std::env::var("FEISHU_TEST_USER_ID")
        .expect("FEISHU_TEST_USER_ID must be set")
        .trim()
        .to_string();
    assert!(!uid.is_empty());
    let ty = std::env::var("FEISHU_TEST_USER_ID_TYPE").unwrap_or_else(|_| "open_id".into());
    (uid, ty.trim().to_string())
}

fn json_form_content(widgets: &Value) -> String {
    serde_json::to_string(widgets).expect("form JSON")
}

fn build_create_approval_body(tag: u64, widget_id: &str) -> Value {
    let node_mid = format!("e2e{:016x}", tag);
    let widgets = json!([{
        "id": widget_id,
        "type": "input",
        "required": false,
        "name": "@i18n@wf_label_e2e"
    }]);
    json!({
        "approval_name": "@i18n@appr_e2e_title",
        "viewers": [{ "viewer_type": "TENANT" }],
        "form": { "form_content": json_form_content(&widgets) },
        "node_list": [
            { "id": "START" },
            {
                "id": node_mid,
                "name": "@i18n@node_mid_e2e",
                "node_type": "OR",
                "approver": [{ "type": "Free" }],
                "privilege_field": {
                    "writable": [widget_id],
                    "readable": [widget_id]
                }
            },
            { "id": "END" }
        ],
        "config": {
            "can_update_viewer": false,
            "can_update_form": false,
            "can_update_process": false,
            "can_update_revert": false
        },
        "i18n_resources": [{
            "locale": "zh-CN",
            "is_default": true,
            "texts": [
                { "key": "@i18n@appr_e2e_title", "value": format!("feishu-approval-tool e2e {}", tag) },
                { "key": "@i18n@wf_label_e2e", "value": "Note" },
                { "key": "@i18n@node_mid_e2e", "value": "Approve" }
            ]
        }]
    })
}

fn build_create_instance_body(
    approval_code: &str,
    user_id: &str,
    user_id_type: &str,
    widget_id: &str,
    value: &str,
) -> Value {
    let row = json!([{ "id": widget_id, "type": "input", "value": value }]);
    let mut m = Map::new();
    m.insert("approval_code".into(), json!(approval_code));
    if user_id_type == "user_id" {
        m.insert("user_id".into(), json!(user_id));
    } else {
        m.insert("open_id".into(), json!(user_id));
    }
    m.insert("form".into(), Value::String(json_form_content(&row)));
    m.insert("uuid".into(), json!(Uuid::new_v4().to_string()));
    Value::Object(m)
}

fn post_create_instance(c: &OpenApiClient, user_id_type: &str, body: &Value) -> String {
    let q = [("user_id_type", user_id_type)];
    let v = api(
        c,
        Method::POST,
        "/open-apis/approval/v4/instances",
        &q,
        Some(body),
        "POST instances (create)",
    );
    v["data"]["instance_code"]
        .as_str()
        .expect("instance_code")
        .to_string()
}

fn cancel_instance(
    c: &OpenApiClient,
    approval_code: &str,
    instance_code: &str,
    user_id: &str,
    user_id_type: &str,
    ctx: &str,
) {
    let body = json!({
        "approval_code": approval_code,
        "instance_code": instance_code,
        "user_id": user_id
    });
    let q = [("user_id_type", user_id_type)];
    let _ = api(
        c,
        Method::POST,
        "/open-apis/approval/v4/instances/cancel",
        &q,
        Some(&body),
        ctx,
    );
}

fn task_id_from_detail(v: &Value) -> String {
    let list = v["data"]["task_list"].as_array().expect("task_list");
    let first = list.first().expect("non-empty task_list");
    first["id"]
        .as_str()
        .or_else(|| first["task_id"].as_str())
        .expect("task id")
        .to_string()
}

fn poll_instance_with_tasks(c: &OpenApiClient, instance_code: &str, ctx: &str) -> Value {
    let path = format!("/open-apis/approval/v4/instances/{instance_code}");
    for _attempt in 1..=60u32 {
        let v = api(c, Method::GET, &path, &[], None, ctx);
        if v["data"]["task_list"].as_array().map(|a| !a.is_empty()).unwrap_or(false) {
            return v;
        }
        sleep(Duration::from_millis(500));
    }
    panic!("{ctx}: task_list empty after ~30s");
}

fn comments_roundtrip(
    c: &OpenApiClient,
    instance_code: &str,
    uid: &str,
    uid_type: &str,
    label: &str,
    text: &str,
) {
    let q = [("user_id", uid), ("user_id_type", uid_type)];
    let path = format!("/open-apis/approval/v4/instances/{instance_code}/comments");
    let _ = api(c, Method::GET, &path, &q, None, &format!("GET comments {label}"));
    let content = serde_json::to_string(&json!({ "text": text })).unwrap();
    let _ = api(
        c,
        Method::POST,
        &path,
        &q,
        Some(&json!({ "content": content, "disable_bot": true })),
        &format!("POST comment {label}"),
    );
}

#[test]
fn approval_v4_live_end_to_end() {
    load_dotenv();
    let (user_id, user_id_type) = require_test_user();
    let c = make_client();

    let _ = api(
        &c,
        Method::GET,
        "/open-apis/approval/v4/districts",
        &[],
        None,
        "GET districts",
    );
    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/districts/search",
        &[("locale", "zh-CN")],
        Some(&json!({ "keyword": "北京" })),
        "POST districts/search",
    );

    let ts = format!("{}", now_ms());
    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/external_instances/check",
        &[],
        Some(&json!({
            "instances": [{
                "instance_id": "feishu_approval_tool_smoke_instance",
                "update_time": ts,
                "tasks": [{ "task_id": "feishu_approval_tool_smoke_task", "update_time": ts }]
            }]
        })),
        "POST external_instances/check",
    );

    let tag = now_ms().max(0) as u64;
    let widget = format!("wf_e2e_{}", tag);
    let q_ut = [("user_id_type", user_id_type.as_str())];

    let created = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/approvals",
        &q_ut,
        Some(&build_create_approval_body(tag, &widget)),
        "POST approvals",
    );
    let approval_code = created["data"]["approval_code"]
        .as_str()
        .expect("approval_code")
        .to_string();

    let path_ap = format!("/open-apis/approval/v4/approvals/{approval_code}");
    let got = api(&c, Method::GET, &path_ap, &[], None, "GET approval");
    assert!(got.get("data").is_some());

    let instance_1 = post_create_instance(
        &c,
        &user_id_type,
        &build_create_instance_body(
            &approval_code,
            &user_id,
            &user_id_type,
            &widget,
            "instance 1",
        ),
    );

    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/instances/query",
        &q_ut,
        Some(&json!({ "instance_code": instance_1.clone(), "page_size": 10 })),
        "POST instances/query (1)",
    );

    let detail_1 = poll_instance_with_tasks(&c, &instance_1, "GET instance (1)");

    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/instances/preview",
        &q_ut,
        Some(&json!({
            "approval_code": approval_code.clone(),
            "user_id": user_id.clone(),
            "form": json_form_content(&json!([{ "id": widget.clone(), "type": "input", "value": "preview" }]))
        })),
        "POST preview before",
    );

    let tid1 = task_id_from_detail(&detail_1);
    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/instances/preview",
        &q_ut,
        Some(&json!({
            "instance_code": instance_1.clone(),
            "user_id": user_id.clone(),
            "task_id": tid1
        })),
        "POST preview after (1)",
    );

    comments_roundtrip(&c, &instance_1, &user_id, &user_id_type, "(1)", "e2e (1)");

    let _ = api(
        &c,
        Method::GET,
        "/open-apis/approval/v4/external_tasks",
        &[("page_size", "20")],
        Some(&json!({ "approval_codes": [approval_code.clone()], "status": "PENDING" })),
        "GET external_tasks",
    );

    let end = now_ms();
    let start_ms = detail_1["data"]["start_time"]
        .as_str()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(end - 8 * 3_600_000);
    let start = (start_ms - 3_600_000).max(0).to_string();
    let end_s = end.to_string();
    let _ = api(
        &c,
        Method::GET,
        "/open-apis/approval/v4/instances",
        &[
            ("approval_code", approval_code.as_str()),
            ("start_time", start.as_str()),
            ("end_time", end_s.as_str()),
            ("page_size", "50"),
        ],
        None,
        "GET instances batch",
    );

    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/instances/query",
        &q_ut,
        Some(&json!({ "approval_code": approval_code.clone(), "page_size": 10 })),
        "POST instances/query by approval",
    );

    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/instances/search_cc",
        &q_ut,
        Some(&json!({
            "user_id": user_id.clone(),
            "instance_code": instance_1.clone(),
            "page_size": 10
        })),
        "POST search_cc",
    );

    let tq = [
        ("user_id", user_id.as_str()),
        ("user_id_type", user_id_type.as_str()),
        ("topic", "1"),
        ("page_size", "20"),
    ];
    let _ = api(
        &c,
        Method::GET,
        "/open-apis/approval/v4/tasks/query",
        &tq,
        None,
        "GET tasks/query",
    );
    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/tasks/search",
        &q_ut,
        Some(&json!({ "user_id": user_id.clone(), "page_size": 20 })),
        "POST tasks/search",
    );

    let sub = format!("/open-apis/approval/v4/approvals/{}/subscribe", approval_code);
    let unsub = format!("/open-apis/approval/v4/approvals/{}/unsubscribe", approval_code);
    let _ = api(&c, Method::POST, &sub, &[], None, "POST subscribe");
    let _ = api(&c, Method::POST, &unsub, &[], None, "POST unsubscribe");

    let instance_2 = post_create_instance(
        &c,
        &user_id_type,
        &build_create_instance_body(
            &approval_code,
            &user_id,
            &user_id_type,
            &widget,
            "instance 2",
        ),
    );

    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/instances/query",
        &q_ut,
        Some(&json!({ "instance_code": instance_2.clone(), "page_size": 10 })),
        "POST instances/query (2)",
    );

    let detail_2 = poll_instance_with_tasks(&c, &instance_2, "GET instance (2)");
    let tid2 = task_id_from_detail(&detail_2);
    let _ = api(
        &c,
        Method::POST,
        "/open-apis/approval/v4/instances/preview",
        &q_ut,
        Some(&json!({
            "instance_code": instance_2.clone(),
            "user_id": user_id.clone(),
            "task_id": tid2
        })),
        "POST preview after (2)",
    );

    comments_roundtrip(&c, &instance_2, &user_id, &user_id_type, "(2)", "e2e (2)");

    cancel_instance(
        &c,
        &approval_code,
        &instance_2,
        &user_id,
        &user_id_type,
        "POST cancel (2)",
    );
    cancel_instance(
        &c,
        &approval_code,
        &instance_1,
        &user_id,
        &user_id_type,
        "POST cancel (1)",
    );
}
