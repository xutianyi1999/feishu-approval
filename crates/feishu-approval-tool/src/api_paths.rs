//! Feishu Open Platform approval API v4 (`/open-apis/approval/v4/...`).

macro_rules! v4 {
    ($suffix:literal) => {
        concat!("/open-apis/approval/v4", $suffix)
    };
}

pub const APPROVAL_V4: &str = "/open-apis/approval/v4";

pub const APPROVALS: &str = v4!("/approvals");
pub const INSTANCES: &str = v4!("/instances");
pub const INSTANCES_QUERY: &str = v4!("/instances/query");
pub const INSTANCES_CC: &str = v4!("/instances/cc");
pub const INSTANCES_CANCEL: &str = v4!("/instances/cancel");
pub const INSTANCES_PREVIEW: &str = v4!("/instances/preview");
pub const INSTANCES_SEARCH_CC: &str = v4!("/instances/search_cc");
pub const INSTANCES_SPECIFIED_ROLLBACK: &str = v4!("/instances/specified_rollback");
pub const INSTANCES_ADD_SIGN: &str = v4!("/instances/add_sign");
pub const DISTRICTS: &str = v4!("/districts");
pub const DISTRICTS_SEARCH: &str = v4!("/districts/search");
pub const EXTERNAL_INSTANCES_CHECK: &str = v4!("/external_instances/check");
pub const EXTERNAL_TASKS: &str = v4!("/external_tasks");
pub const TASKS_SEARCH: &str = v4!("/tasks/search");
pub const TASKS_QUERY: &str = v4!("/tasks/query");
pub const TASKS_APPROVE: &str = v4!("/tasks/approve");
pub const TASKS_REJECT: &str = v4!("/tasks/reject");
pub const TASKS_TRANSFER: &str = v4!("/tasks/transfer");
pub const TASKS_RESUBMIT: &str = v4!("/tasks/resubmit");

#[inline]
pub fn approval_definition(approval_code: &str) -> String {
    format!("{APPROVAL_V4}/approvals/{approval_code}")
}

#[inline]
pub fn subscribe(approval_code: &str) -> String {
    format!("{APPROVAL_V4}/approvals/{approval_code}/subscribe")
}

#[inline]
pub fn unsubscribe(approval_code: &str) -> String {
    format!("{APPROVAL_V4}/approvals/{approval_code}/unsubscribe")
}

#[inline]
pub fn instance(instance: &str) -> String {
    format!("{APPROVAL_V4}/instances/{instance}")
}

#[inline]
pub fn instance_comments(instance: &str) -> String {
    format!("{APPROVAL_V4}/instances/{instance}/comments")
}

#[inline]
pub fn instance_comment(instance: &str, comment_id: &str) -> String {
    format!("{APPROVAL_V4}/instances/{instance}/comments/{comment_id}")
}

#[inline]
pub fn instance_comments_remove(instance: &str) -> String {
    format!("{APPROVAL_V4}/instances/{instance}/comments/remove")
}
