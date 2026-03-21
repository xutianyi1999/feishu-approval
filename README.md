# feishu-approval

飞书 / Lark **审批 Open API v4** 的 Rust 工作区：可安装的 CLI **`feishu-approval-tool`**，以及同名 **`feishu_approval_tool`** 库（`OpenApiClient`、换票、JSON 调用、审批文件 multipart 上传）。仓库根目录另有 OpenClaw 技能文件（`SKILL.md`、`approval-code-map.md`、`embedded-docs/`），与 [`SKILL.md`](SKILL.md) 描述一致。

## 安装 CLI

在仓库根目录执行（二进制默认安装到 `~/.cargo/bin`，请确保已在 `PATH` 中）：

```bash
cargo install --path crates/feishu-approval-tool
```

可选 `--locked` 以使用仓库内的 `Cargo.lock`。在 `crates/feishu-approval-tool` 目录下可执行 `cargo install --path .`。

**依赖**：Rust 工具链（edition 2021）。

执行时**工作目录**建议为含 **`.env`** 的目录；工具会加载 `.env`（**不覆盖**已有环境变量）。更完整的全局选项与子命令说明见 [`SKILL.md`](SKILL.md)「全局参数」「子命令一览」两节。

## 环境变量与全局选项

与 [`SKILL.md`](SKILL.md)「全局参数」一节一致（所有子命令前均可加对应长选项）：

| 长选项 / 环境变量 | 说明 |
|-------------------|------|
| `--base` / `FEISHU_OPEN_BASE` | 开放平台根 URL，默认 `https://open.feishu.cn`；国际版常用 `https://open.larksuite.com` |
| `--approval-upload-base` / `FEISHU_APPROVAL_UPLOAD_BASE` | 审批附件上传域，默认 `https://www.feishu.cn` |
| `--token` / `FEISHU_TENANT_ACCESS_TOKEN` | Bearer；不设则用 `FEISHU_APP_ID` + `FEISHU_APP_SECRET` 换 `tenant_access_token` |
| `--timeout-secs` | HTTP 超时秒数，默认 `60` |
| `--raw` | JSON 不美化，便于管道 |

勿将 `app_secret` 提交到仓库或泄露到日志。

## 用法摘要

```bash
feishu-approval-tool token
feishu-approval-tool approval get -c <APPROVAL_CODE>
feishu-approval-tool instance get -i <INSTANCE_CODE_OR_UUID>
feishu-approval-tool file upload --path ./a.pdf --kind attachment
feishu-approval-tool api get approval/v4/districts
```

完整子命令与参数：

```bash
feishu-approval-tool --help
feishu-approval-tool instance create --help
```

## 仓库布局

| 路径 | 说明 |
|------|------|
| `crates/feishu-approval-tool/` | CLI + `feishu_approval_tool` 库源码 |
| `SKILL.md` | OpenClaw 技能说明（安装、子命令、工作流约定） |
| `approval-code-map.md` | 审批显示名称 ↔ `approval_code` 映射（租户填写，勿改名） |
| `embedded-docs/` | 内嵌开放平台文档片段（字段与示例，供 Agent Read） |

技能说明、工作流与字段导航以 [`SKILL.md`](SKILL.md) 为准；字段级文档入口为 [`embedded-docs/INDEX.md`](embedded-docs/INDEX.md)。装入 OpenClaw 时把三项放进同一目录（建议名 `feishu-approval`），再放到 `~/.openclaw/skills/feishu-approval/` 或项目 `skills/feishu-approval/`（见 `SKILL.md`「技能包结构」）。

## 开发

```bash
cargo build -p feishu-approval-tool
cargo test -p feishu-approval-tool
cargo clippy -p feishu-approval-tool --all-targets
```

集成测试 `crates/feishu-approval-tool/tests/real_api.rs` 会访问真实 API，需配置相应环境变量（见该文件顶部注释）。

## 官方文档

与 [`SKILL.md`](SKILL.md)「在线文档」一节相同：

- [审批概述](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval-overview)
- [tenant_access_token（自建应用）](https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/auth-v3/auth/tenant_access_token_internal)
- [事件订阅概述](https://open.feishu.cn/document/ukTMukTMukTM/uUTNz4SN1MjL1UzM)

## 许可证

`MIT OR Apache-2.0`（见各 `Cargo.toml`）。
