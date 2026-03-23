# feishu-approval

飞书 / Lark **审批 Open API v4**：CLI **`feishu-approval-tool`**、库 **`feishu_approval_tool`**；技能包含 **`SKILL.md`**、**`docs/AI.md`**、**`approval-code-map.md`**、**`embedded-docs/`** 等。

**AI / 自动化**：操作步骤与 JSON 约定见 **`docs/AI.md`**（最小成功路径、`util extract-widgets` / `validate-widgets` / `doctor`、`instance create --validate-against-json`）；安装与全局环境变量见 **`SKILL.md`**。

### 安装 CLI

```bash
cargo install --path crates/feishu-approval-tool
```

全局参数与凭证：`SKILL.md` 或 **`feishu-approval-tool -h`**。

### 仓库布局

| 路径 | 说明 |
|------|------|
| `crates/feishu-approval-tool/` | CLI + 库源码 |
| `docs/AI.md` | AI 操作手册（单一入口） |
| `docs/examples/` | 示例 JSON |
| `SKILL.md` | 技能 frontmatter、安装、环境变量、打包目录 |
| `approval-code-map.md` | 审批显示名 ↔ `approval_code` |
| `embedded-docs/` | 开放平台文档片段（按需单页） |
| `AGENTS.md` | 改代码时的约定 |

### 开发

```bash
cargo build -p feishu-approval-tool
cargo test -p feishu-approval-tool
cargo clippy -p feishu-approval-tool --all-targets
```

集成测试 `crates/feishu-approval-tool/tests/real_api.rs` 访问真实 API，见文件头注释。

### 许可证

`MIT OR Apache-2.0`（见各 `Cargo.toml`）。
