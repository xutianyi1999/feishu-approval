# Repository guide for coding agents

- **Rust**：`crates/feishu-approval-tool`（CLI `main.rs`；`cli/handlers/`：`mod.rs`（含 `resolve_instance_create_form`）、`approval.rs`（`dump --data-only`）、`instance.rs`、`task.rs`、`comment.rs`、`util.rs`；`cli/json_util.rs`）。
- **AI 文档**：根目录 **`docs/AI.md`** 为面向模型的**唯一**操作手册；改 CLI 行为时同步更新该文件与 **`SKILL.md`**（frontmatter `description`、全局参数表若有变）。**`util`**：`form-string`、`validate-widgets`、`extract-widgets`、`scaffold-widgets`（离线）；**`instance create`**：`--validate-against-json`；**`doctor`**（环境/换票）。**`OpenApiClient::request_json`**：HTTP 2xx 且 JSON 顶层 **`code` ≠ 0** 时返回 `Err`，stderr 可含 **`docs/AI.md` §7** 类提示。
- **勿恢复**已删除的 `recipes/` 路径；示例 JSON 在 **`docs/examples/`**。**`approval-code-map.local.md`** 租户映射不入库（见 **`approval-code-map.md`**）；跟踪 **`approval-code-map.local.md.example`**。
- **验证**：`cargo test -p feishu-approval-tool`、`cargo clippy -p feishu-approval-tool --all-targets`。
