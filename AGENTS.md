# Repository guide for coding agents

- **Rust**：`crates/feishu-approval-tool` — `lib.rs`（**`api_paths`**、`explain_feishu_widget_msg`、`OpenApiClient::request_json`：HTTP 2xx 且 JSON **`code` ≠ 0** → `Err`，stderr 可含 **`docs/AI.md` §7** 类提示）；`main.rs`；`cli/handlers/`（`mod.rs`：`resolve_instance_create_form`、`exec_normalized`/`dispatch_api`；`approval.rs`、`instance.rs`、`task.rs`、`comment.rs`、`util.rs`、`wizard.rs`）；`cli/json_util.rs`（**`validate_instance_create_extra_patch`** 等）。
- **文档**：**`docs/AI.md`** 为面向模型的**唯一操作手册**（含子命令摘要、JSON 约定、排错、§8 映射）。改 CLI 行为时同步 **`docs/AI.md`**、**`SKILL.md`**（frontmatter `description`、全局参数表若有变），并核对 **`README.md`** 与本文件指向。
- **勿恢复**已删除的 `recipes/`、仓库级 **`approval-code-map.md`**、根目录 **`approval-code-map.local.md.example`**。映射与 **`util init`** → **`docs/AI.md` §8**；模板 **`docs/approval-code-map.local.template.md`**（**`include_str!`**）。**`approval-code-map.local.md`** 不入库。
- **验证**：`cargo test -p feishu-approval-tool`、`cargo clippy -p feishu-approval-tool --all-targets`。
