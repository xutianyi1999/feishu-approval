---
name: feishu-approval
description: "OpenClaw: Feishu/Lark approval v4 via feishu-approval-tool. docs/AI.md; approval-code-map.local.md first (gitignored), else approval-code-map.md; §7 fieldList/date/formula; util scaffold-widgets, extract-widgets, validate-widgets, doctor; Feishu code≠0 → stderr hints; instance create --widgets-json-file --validate-against-json; approval dump --data-only. Triggers: 飞书审批, Lark approval, approval_code, instance_code, 抄送, 加签, 订阅."
---

# 飞书审批技能（`feishu-approval-tool`）

Shell 调用 CLI，访问 **`/open-apis/approval/v4/...`** 及 **`api`** 逃生口。

## 文档怎么读

| 优先级 | 文件 |
|--------|------|
| 执行任务、JSON/文件约定、子命令表 | **`docs/AI.md`** |
| 仅中文审批名、未给 code；费用报销类形状示例 | **`approval-code-map.local.md`**（优先，不入库）或 **`approval-code-map.md`**；首次可复制 **`approval-code-map.local.md.example`** |
| 仅缺 HTTP 参数/错误码/事件体 | **`embedded-docs/INDEX.md`** 中**一行**链到单页（勿通读 INDEX） |
| 与实现一致的 flags | **`feishu-approval-tool -h`** / **`<子命令> --help`** |

## 技能包目录（与 `SKILL.md` 同级）

装入 OpenClaw 时建议包含：**`SKILL.md`**、**`approval-code-map.md`**、**`approval-code-map.local.md.example`**、**`approval-code-map.local.md`**（由你在本机从 example 复制并填写，**勿**提交）、**`embedded-docs/`**、**`docs/`**（至少 **`docs/AI.md`** 与 **`docs/examples/`**）。可选 **`AGENTS.md`**（仅仓库开发）。目录名建议 `feishu-approval`。

## 安装 CLI

```bash
cargo install --path crates/feishu-approval-tool
```

（仓库根执行；可选 **`--locked`**。）二进制一般在 **`~/.cargo/bin`**。工作目录建议含 **`.env`**；工具加载 `.env` 且不覆盖已有环境变量。

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

## 子命令与在线文档

子命令摘要、**`--json-file` / `-` / `form`** 等约定见 **`docs/AI.md`** §2–§3；完整列表以 **`feishu-approval-tool --help`** 为准。内嵌单页从 **`embedded-docs/INDEX.md`** 按场景选一行（见上文「文档怎么读」）。

在线参考：[审批概述](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval-overview)、[tenant_access_token](https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/auth-v3/auth/tenant_access_token_internal)、[事件订阅](https://open.feishu.cn/document/ukTMukTMukTM/uUTNz4SN1MjL1UzM)。与在线冲突时**以 `embedded-docs` 单页为准**。
