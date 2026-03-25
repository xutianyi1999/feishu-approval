# feishu-approval

飞书 / Lark **审批 Open API v4**：CLI **`feishu-approval-tool`**、库 **`feishu_approval_tool`**。

**怎么用（人 / Agent）**：**[`docs/AI.md`](docs/AI.md)**（单一入口）。**安装与环境变量**：根目录 **`SKILL.md`**。**改本仓库**：**[`AGENTS.md`](AGENTS.md)**。

### 开发

```bash
cargo build -p feishu-approval-tool
cargo test -p feishu-approval-tool
cargo clippy -p feishu-approval-tool --all-targets
```

集成测试 `crates/feishu-approval-tool/tests/real_api.rs` 访问真实 API，见文件头注释。

### 许可证

`MIT OR Apache-2.0`（见各 `Cargo.toml`）。
