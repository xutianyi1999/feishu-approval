---
name: feishu-approval
description: "AI-first: OpenClaw/agents drive feishu-approval-tool (not a casual human CLI). No approval-code-map.local.md + user names approval only → must ask user for approval_code then write map (util init + table row); docs/AI.md §8; §7 fieldList/date/formula; util scaffold/extract/validate/form-string/doctor/init; task reject batch; task search filters; instance --wizard|--template expense; Feishu code≠0 → stderr hints. Triggers: 飞书审批, Lark approval, approval_code, instance_code, 抄送, 加签, 订阅."
---

# 飞书审批技能（`feishu-approval-tool`）

**AI-first**：本技能假定由 **模型 / 自动化** 按 **`docs/AI.md`** 与 **`--help`** 调用 CLI；人类操作员同样以 **`docs/AI.md`** 为入口即可，不要求记住所有子命令。

Shell 调用 CLI，访问 **`/open-apis/approval/v4/...`** 及 **`api`** 逃生口。

## 文档怎么读

**完整「哪份文档何时打开」决策表**见 **`docs/AI.md` §1**（单一维护点）。技能包**必须**包含 **`docs/AI.md`**，否则仅靠本文件无法覆盖子命令、JSON 约定与 §7 排错。

## 技能包目录（与 `SKILL.md` 同级）

装入 OpenClaw 时建议包含：**`SKILL.md`**、**`embedded-docs/`**、**`docs/`**（**`docs/AI.md`**、**`docs/examples/`**、**`docs/approval-code-map.local.template.md`**）、本机 **`approval-code-map.local.md`**（**勿**提交；无文件时用户只给审批名须**先问 code** 再 **`util init` + 写表**，见 **`docs/AI.md` §8**）。可选 **`AGENTS.md`**（仅仓库开发）。目录名建议 `feishu-approval`。

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

子命令摘要、**`--json-file` / `-` / `form`** 等约定见 **`docs/AI.md`** §2–§3；完整列表以 **`feishu-approval-tool --help`** 为准。HTTP 单页从 **`embedded-docs/INDEX.md`** 选一行（路由逻辑见 **`docs/AI.md` §1**）。

在线参考：[审批概述](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval-overview)、[tenant_access_token](https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/auth-v3/auth/tenant_access_token_internal)、[事件订阅](https://open.feishu.cn/document/ukTMukTMukTM/uUTNz4SN1MjL1UzM)。与在线冲突时**以 `embedded-docs` 单页为准**。
