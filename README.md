# feishu-approval

飞书 / Lark **审批 Open API v4**：CLI **`feishu-approval-tool`**、库 **`feishu_approval_tool`**；技能包含 **`SKILL.md`**、**`docs/AI.md`**、**`approval-code-map.md`**（说明）、**`approval-code-map.local.md.example`**、**`approval-code-map.local.md`**（本机映射，不入库）、**`embedded-docs/`**、**`docs/examples/`** 等。

**AI / 自动化**：流程与 JSON 约定见 **`docs/AI.md`**（最小成功路径、**`util scaffold-widgets`** / **`extract-widgets`** / **`validate-widgets`** / **`form-string`** / **`doctor`**、**`instance create --widgets-json-file`** 与 **`--validate-against-json`**、飞书响应 **`code`≠0** 时 CLI **non-zero 退出** 及 stderr 中的 §7 类提示）。**`approval_code`** 仅中文名时：**`approval-code-map.local.md`**（优先）→ **`approval-code-map.md`**。安装、技能包目录与全局环境变量见 **`SKILL.md`**。

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
| `approval-code-map.md` | 映射说明 + 共享补充（费用报销示例等） |
| `approval-code-map.local.md.example` | 复制为 `approval-code-map.local.md` 的表格模板（入库） |
| `approval-code-map.local.md` | 本企业 **显示名 ↔ `approval_code`**（`.gitignore`，从 example 复制后填写） |
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
