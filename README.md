# feishu-approval

飞书 / Lark **审批 Open API v4**：CLI **`feishu-approval-tool`**、库 **`feishu_approval_tool`**。**AI-first**：人 / 自动化均以 **`docs/AI.md`** 为入口；安装与环境变量见 **`SKILL.md`**；flags 以 **`feishu-approval-tool -h`** 为准。

### 仓库布局

| 路径 | 说明 |
|------|------|
| `crates/feishu-approval-tool/` | CLI + 库源码 |
| `docs/AI.md` | 操作手册（单一入口） |
| `docs/examples/` | 示例 JSON |
| `docs/approval-code-map.local.template.md` | 本地映射表模板（**`util init`**） |
| `SKILL.md` | 技能元数据、安装、全局参数 |
| `embedded-docs/` | 开放平台文档片段（按需单页） |
| `AGENTS.md` | 改本仓库代码时的约定 |

本机 **`approval-code-map.local.md`**（**`.gitignore`**）见 **`docs/AI.md` §8**。

### 开发

```bash
cargo build -p feishu-approval-tool
cargo test -p feishu-approval-tool
cargo clippy -p feishu-approval-tool --all-targets
```

集成测试 `crates/feishu-approval-tool/tests/real_api.rs` 访问真实 API，见文件头注释。

### 许可证

`MIT OR Apache-2.0`（见各 `Cargo.toml`）。
