# feishu-approval

飞书 / Lark **审批 Open API v4** 的 Rust 工作区：可安装的 CLI **`feishu-approval-tool`**，以及同名 **`feishu_approval_tool`** 库（`OpenApiClient`、换票、JSON 调用、审批文件 multipart 上传）。

## 安装 CLI

在仓库根目录执行（二进制默认安装到 `~/.cargo/bin`，请确保已在 `PATH` 中）：

```bash
cargo install --path crates/feishu-approval-tool
```

可选 `--locked` 以使用仓库内的 `Cargo.lock`。在 `crates/feishu-approval-tool` 目录下可执行 `cargo install --path .`。

**依赖**：Rust 工具链（edition 2021）。

## 环境变量

程序会尝试加载当前工作目录下的 **`.env`**（不覆盖已存在的环境变量）。

| 变量 / 选项 | 说明 |
|-------------|------|
| `FEISHU_APP_ID` / `FEISHU_APP_SECRET` | 未提供 `FEISHU_TENANT_ACCESS_TOKEN` 时用于换取 `tenant_access_token` |
| `FEISHU_TENANT_ACCESS_TOKEN` | 或命令行 `--token` |
| `FEISHU_OPEN_BASE` | 开放平台 host，默认 `https://open.feishu.cn`；国际版常用 `https://open.larksuite.com` |
| `FEISHU_APPROVAL_UPLOAD_BASE` | 审批文件上传 host，默认 `https://www.feishu.cn`；或 `--approval-upload-base` |

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
| `skills/feishu-approval/` | OpenClaw 技能：`SKILL.md` + `embedded-docs/`（接口字段与示例，供 Agent Read） |

技能说明以 [`skills/feishu-approval/SKILL.md`](skills/feishu-approval/SKILL.md) 为准；字段级文档入口为 [`skills/feishu-approval/embedded-docs/INDEX.md`](skills/feishu-approval/embedded-docs/INDEX.md)。

## 开发

```bash
cargo build -p feishu-approval-tool
cargo test -p feishu-approval-tool
cargo clippy -p feishu-approval-tool --all-targets
```

集成测试 `tests/real_api.rs` 会访问真实 API，需配置相应环境变量（见该文件顶部注释）。

## 官方文档

- [审批概述](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval-overview)
- [tenant_access_token（自建应用）](https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/auth-v3/auth/tenant_access_token_internal)

## 许可证

`MIT OR Apache-2.0`（见各 `Cargo.toml`）。
