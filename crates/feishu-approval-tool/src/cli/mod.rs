//! High-level CLI for Feishu Approval Open API v4 (`/open-apis/approval/v4/...`).

mod exec;
mod handlers;
mod json_util;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "feishu-approval-tool")]
#[command(about = "Feishu workflow approval (v4): structured subcommands + `api` escape hatch")]
pub struct Cli {
    /// Open Platform host (no path suffix). Env: `FEISHU_OPEN_BASE`.
    #[arg(long, env = "FEISHU_OPEN_BASE", default_value = "https://open.feishu.cn")]
    pub base: String,

    /// Base URL for approval file upload (`…/approval/openapi/v2/file/upload`). Env: `FEISHU_APPROVAL_UPLOAD_BASE`.
    #[arg(long, env = "FEISHU_APPROVAL_UPLOAD_BASE", default_value = "https://www.feishu.cn", global = true)]
    pub approval_upload_base: String,

    /// Bearer token; overrides `FEISHU_TENANT_ACCESS_TOKEN` and app credential exchange.
    #[arg(long, env = "FEISHU_TENANT_ACCESS_TOKEN")]
    pub token: Option<String>,

    /// HTTP client timeout in seconds.
    #[arg(long, default_value_t = 60)]
    pub timeout_secs: u64,

    /// Print JSON response without pretty-printing (for piping).
    #[arg(long)]
    pub raw: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Print tenant_access_token (FEISHU_APP_ID + FEISHU_APP_SECRET).
    Token,
    /// Approval definition: GET /approvals/:code
    Approval {
        #[command(subcommand)]
        action: ApprovalAction,
    },
    /// Approval instances
    Instance {
        #[command(subcommand)]
        action: InstanceAction,
    },
    /// Approval tasks: approve, reject, transfer, resubmit, search, user task list
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Instance comments
    Comment {
        #[command(subcommand)]
        action: CommentAction,
    },
    /// POST .../approvals/:code/subscribe
    Subscribe {
        #[arg(long)]
        approval_code: String,
    },
    /// POST .../approvals/:code/unsubscribe
    Unsubscribe {
        #[arg(long)]
        approval_code: String,
    },
    /// Low-level HTTP escape hatch: any /open-apis/ path
    Api {
        #[command(subcommand)]
        action: ApiAction,
    },
    /// Upload file for image/attachment form widgets (returns `code` for `instance create`)
    File {
        #[command(subcommand)]
        action: FileAction,
    },
}

/// `image` / `attachment` must match the widget type in the approval definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum FileWidgetKind {
    Image,
    Attachment,
}

impl FileWidgetKind {
    pub fn as_api_str(self) -> &'static str {
        match self {
            FileWidgetKind::Image => "image",
            FileWidgetKind::Attachment => "attachment",
        }
    }
}

#[derive(Subcommand)]
pub enum FileAction {
    /// Multipart upload (one file per request)
    Upload {
        /// Local file path
        #[arg(long)]
        path: PathBuf,
        #[arg(long, value_enum)]
        kind: FileWidgetKind,
        /// `name` field (defaults to file basename)
        #[arg(long)]
        name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ApprovalAction {
    /// GET approval definition by code
    Get {
        #[arg(short = 'c', long)]
        approval_code: String,
    },
}

#[derive(Subcommand)]
pub enum InstanceAction {
    /// GET single instance (path param: instance_code or uuid)
    Get {
        #[arg(short = 'i', long)]
        instance: String,
    },
    /// GET instance_code batch (time range + paging)
    List {
        #[arg(long)]
        approval_code: String,
        #[arg(long)]
        start_time: String,
        #[arg(long)]
        end_time: String,
        #[arg(long)]
        page_size: Option<String>,
        #[arg(long)]
        page_token: Option<String>,
    },
    /// POST create instance (`form` is the API stringified JSON array; or put that string in a file)
    Create {
        #[arg(long)]
        approval_code: String,
        /// Form value: stringified JSON array (single line / escaped, as in API docs)
        #[arg(long)]
        form: Option<String>,
        /// Read form string from file (trimmed; entire file becomes `form`)
        #[arg(long)]
        form_file: Option<PathBuf>,
        #[arg(long)]
        open_id: Option<String>,
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long)]
        department_id: Option<String>,
        #[arg(long)]
        uuid: Option<String>,
        /// Merge JSON object into body (optional approvers, CC, i18n, etc.)
        #[arg(long)]
        extra_json: Option<PathBuf>,
    },
    /// POST query instance list (body: see embedded-docs instance/query)
    Query {
        #[arg(long)]
        json_file: PathBuf,
    },
    /// POST CC instance to users
    Cc {
        #[arg(long)]
        approval_code: String,
        #[arg(long)]
        instance_code: String,
        #[arg(long)]
        user_id: String,
        #[arg(long, action = clap::ArgAction::Append)]
        cc_user_id: Vec<String>,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
    },
    /// POST cancel (revoke) instance
    Cancel {
        #[arg(long)]
        approval_code: String,
        #[arg(long)]
        instance_code: String,
        #[arg(long)]
        user_id: String,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
    },
    /// POST preview flow (body: see approval-preview doc)
    Preview {
        #[arg(long)]
        json_file: PathBuf,
    },
    /// POST search CC list (Custom App only per docs)
    SearchCc {
        #[arg(long)]
        json_file: PathBuf,
    },
    /// POST specified rollback
    SpecifiedRollback {
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        task_id: String,
        #[arg(long = "task-def-key", action = clap::ArgAction::Append)]
        task_def_key: Vec<String>,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        extra: Option<String>,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
    },
    /// POST add sign
    AddSign {
        #[arg(long)]
        approval_code: String,
        #[arg(long)]
        instance_code: String,
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        task_id: String,
        #[arg(long, action = clap::ArgAction::Append)]
        add_sign_user_id: Vec<String>,
        #[arg(long)]
        add_sign_type: i32,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long)]
        approval_method: Option<i32>,
    },
}

/// Used by `task act --action ...` (shared flags).
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum TaskActOp {
    Approve,
    Reject,
    Transfer,
    Resubmit,
}

#[derive(Subcommand)]
pub enum TaskAction {
    /// Approve, reject, transfer, or resubmit with the same core flags
    Act {
        #[arg(value_enum)]
        action: TaskActOp,
        #[arg(long)]
        approval_code: String,
        #[arg(long)]
        instance_code: String,
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        task_id: String,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long)]
        form: Option<String>,
        #[arg(long)]
        form_file: Option<PathBuf>,
        #[arg(long)]
        transfer_user_id: Option<String>,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
    },
    /// POST approve task
    Approve {
        #[arg(long)]
        approval_code: String,
        #[arg(long)]
        instance_code: String,
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        task_id: String,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long)]
        form: Option<String>,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
    },
    /// POST reject task
    Reject {
        #[arg(long)]
        approval_code: String,
        #[arg(long)]
        instance_code: String,
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        task_id: String,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long)]
        form: Option<String>,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
    },
    /// POST transfer task
    Transfer {
        #[arg(long)]
        approval_code: String,
        #[arg(long)]
        instance_code: String,
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        task_id: String,
        #[arg(long)]
        transfer_user_id: String,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
    },
    /// POST resubmit task
    Resubmit {
        #[arg(long)]
        approval_code: String,
        #[arg(long)]
        instance_code: String,
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        task_id: String,
        #[arg(long)]
        form: Option<String>,
        #[arg(long)]
        form_file: Option<PathBuf>,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
    },
    /// POST tasks/search
    Search {
        #[arg(long)]
        json_file: PathBuf,
    },
    /// GET tasks/query (user task groups / tabs)
    Query {
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        topic: String,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
        #[arg(long)]
        page_size: Option<String>,
        #[arg(long)]
        page_token: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CommentAction {
    /// GET list comments
    List {
        #[arg(short = 'i', long)]
        instance: String,
        #[arg(long)]
        user_id: String,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
        #[arg(long)]
        page_token: Option<String>,
        #[arg(long)]
        page_size: Option<String>,
    },
    /// POST create comment
    Create {
        #[arg(short = 'i', long)]
        instance: String,
        #[arg(long)]
        json_file: PathBuf,
    },
    /// DELETE comment
    Delete {
        #[arg(short = 'i', long)]
        instance: String,
        #[arg(long)]
        comment_id: String,
        #[arg(long)]
        user_id: String,
        #[arg(long, default_value = "open_id")]
        user_id_type: String,
    },
    /// POST comments/remove
    Remove {
        #[arg(short = 'i', long)]
        instance: String,
        #[arg(long)]
        json_file: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ApiAction {
    Get {
        path: String,
        #[arg(long = "query", value_name = "KEY=VALUE")]
        query: Vec<String>,
    },
    Post {
        path: String,
        #[arg(long = "query", value_name = "KEY=VALUE")]
        query: Vec<String>,
        #[arg(long, conflicts_with = "json_file")]
        json: Option<String>,
        #[arg(long)]
        json_file: Option<PathBuf>,
    },
    Delete {
        path: String,
        #[arg(long = "query", value_name = "KEY=VALUE")]
        query: Vec<String>,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    handlers::dispatch(&cli)
}
