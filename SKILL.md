---
name: feishu-approval
description: "AI-first feishu-approval-tool for OpenClaw/agents. Primary doc: docs/AI.md. Triggers: 飞书审批, Lark approval, approval_code, instance_code, 抄送, 加签, 订阅."
---

# 飞书审批技能（`feishu-approval-tool`）

由 **`docs/AI.md`**（流程、子命令、排错、映射）与 **`feishu-approval-tool --help`** 驱动；本文件只补 **安装、环境变量、技能包清单**。

## 技能包应含（与 `SKILL.md` 同级）

- **`SKILL.md`**、**`embedded-docs/`**、**`docs/`**（含 **`docs/AI.md`**、**`docs/examples/`**、**`docs/approval-code-map.local.template.md`**）
- 本机 **`approval-code-map.local.md`**（**`.gitignore`**；路径与 **`util init`** → **`docs/AI.md` §8**）
- 可选 **`AGENTS.md`**（仅改仓库代码时）

目录名建议 `feishu-approval`。

## 安装 CLI

```bash
cargo install --path crates/feishu-approval-tool
```

（在仓库根执行；可选 **`--locked`**。）二进制一般在 **`~/.cargo/bin`**。工作目录建议含 **`.env`**；工具加载 `.env` 且不覆盖已有环境变量。

## 全局参数（可加在任意子命令前）

| 长选项 / 环境变量 | 含义 |
|-------------------|------|
| `--base` / `FEISHU_OPEN_BASE` | 开放平台根 URL，默认 `https://open.feishu.cn` |
| `--approval-upload-base` / `FEISHU_APPROVAL_UPLOAD_BASE` | 审批上传域，默认 `https://www.feishu.cn` |
| `--token` / `FEISHU_TENANT_ACCESS_TOKEN` | Bearer；不设则用 **`FEISHU_APP_ID`** + **`FEISHU_APP_SECRET`** 换票 |
| `FEISHU_APP_ID` / `FEISHU_APP_SECRET` | 仅环境或 `.env`，无对应 CLI 长选项 |
| `--timeout-secs` | HTTP 超时，默认 `60` |
| `--raw` | JSON 不美化 |

勿泄露 **`app_secret`**。

## 在线文档（与 embedded 冲突时以单页为准）

[审批概述](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval-overview)、[tenant_access_token](https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/auth-v3/auth/tenant_access_token_internal)、[事件订阅](https://open.feishu.cn/document/ukTMukTMukTM/uUTNz4SN1MjL1UzM)。

HTTP 字段形状按需从 **`embedded-docs/INDEX.md`** 选**一行**（路由见 **`docs/AI.md` §1**）。
