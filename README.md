# feishu-approval

飞书 / Lark **审批 Open API v4**：CLI **`feishu-approval-tool`**、库 **`feishu_approval_tool`**。设计上是 **AI-first**（技能与自动化按 `docs/AI.md` 驱动 CLI），人类同样以该手册为入口即可。

- **使用说明（人 / AI）**：**`docs/AI.md`**
- **安装 CLI、全局参数、技能包清单**：**`SKILL.md`**；与实现一致的 flags：**`feishu-approval-tool -h`**

### 仓库布局

| 路径 | 说明 |
|------|------|
| `crates/feishu-approval-tool/` | CLI + 库源码 |
| `docs/AI.md` | AI 操作手册（单一入口） |
| `docs/examples/` | 示例 JSON |
| `docs/approval-code-map.local.template.md` | 本地「中文名 ↔ approval_code」空表模板；**`util init`** 写入根目录 `approval-code-map.local.md` |
| `SKILL.md` | 技能 frontmatter、安装、环境变量 |
| `approval-code-map.local.md` | 本企业映射（**`.gitignore`**，勿提交） |
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
