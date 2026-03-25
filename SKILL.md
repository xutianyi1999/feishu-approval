---
name: feishu-approval
description: "飞书审批 CLI 技能包。工作流与规则只读 docs/AI.md；本文件仅安装、环境、全局参数。触发：飞书审批、Lark approval、approval_code、instance_code、抄送、加签、订阅。"
---

# 技能包：`feishu-approval-tool`

- **工作流、子命令摘要、JSON、映射、排错**：**`docs/AI.md`**（单一入口；阅读地图见其 **§1**）。
- **HTTP 字段细节**：**`embedded-docs/INDEX.md`** 按需一行；与单页冲突以单页为准。
- **子命令 flags**：**`feishu-approval-tool -h`** / **`<子命令> --help`**。

## 目录（与 `SKILL.md` 同级）

**`SKILL.md`**、**`embedded-docs/`**、**`docs/`**（含 **`docs/AI.md`**、**`docs/examples/`**、**`docs/approval-code-map.local.template.md`**）、本机 **`approval-code-map.local.md`**（**`.gitignore`**；**`docs/AI.md` §8**）。可选 **`AGENTS.md`**（仅改仓库代码时）。目录名建议 `feishu-approval`。

## 安装 CLI

```bash
cargo install --path crates/feishu-approval-tool
```

在仓库根执行；可选 **`--locked`**。二进制一般在 **`~/.cargo/bin`**。工作目录建议含 **`.env`**；工具加载 `.env` 且不覆盖已有环境变量。

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
