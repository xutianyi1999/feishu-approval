---
name: feishu-approval
description: "OpenClaw: Feishu/Lark approval v4 via feishu-approval-tool (token, approval|instance|task|comment, file upload, subscribe|unsubscribe, api). Env FEISHU_APP_ID, FEISHU_APP_SECRET, FEISHU_TENANT_ACCESS_TOKEN, FEISHU_OPEN_BASE, FEISHU_APPROVAL_UPLOAD_BASE; .env. Field-level Read embedded-docs/INDEX.md. Triggers: 飞书审批, Lark approval, approval_code, instance_code, 抄送, 加签, 订阅."
---

# `feishu-approval-tool`（飞书审批 · OpenClaw）

供 **OpenClaw** 在 shell 中调用 **`feishu-approval-tool`**。需要拼 JSON body、`form`、控件 value、错误码、事件字段时 → **Read 技能目录内** `embedded-docs/`（从 `embedded-docs/INDEX.md` 定位）。**勿编造子命令**：下表与仓库 `crates/feishu-approval-tool` 一致。

## 安装（OpenClaw）

1. 将 **`feishu-approval/`** 整包放到 OpenClaw 技能目录，例如 **`~/.openclaw/skills/feishu-approval/`**（若你使用工作区技能，则为项目下 `skills/feishu-approval/`）。
2. 包内须含 **`SKILL.md`** 与 **`embedded-docs/`**（字段级说明依赖后者）。
3. **`SKILL.md` frontmatter**：每个键 **单行**（勿多行 `description`），避免解析失败。
4. **安装 CLI**：在仓库根执行 **`cargo install --path crates/feishu-approval-tool`**（默认安装到 **`~/.cargo/bin`**，请把该目录加入 **`PATH`**）。可选 **`--locked`** 使用仓库 `Cargo.lock`。若已在 `crates/feishu-approval-tool` 目录下，可用 **`cargo install --path .`**。
5. 执行命令时 **工作目录** 建议为含 **`.env`**（`FEISHU_APP_ID` 等）的目录，或事先导出环境变量。

| 技能包内路径 | 用途 |
|--------------|------|
| `SKILL.md` | 本说明 |
| `embedded-docs/` | 参数表、JSON 示例、FAQ（OpenClaw Read） |

## 环境变量与鉴权

启动时加载当前目录 **`.env`**（不覆盖已有环境变量）。

| 用途 | 变量 / 选项 |
|------|----------------|
| 开放平台 host | `--base` / `FEISHU_OPEN_BASE`（默认 `https://open.feishu.cn`；国际版常用 `https://open.larksuite.com`） |
| Bearer | `--token` / `FEISHU_TENANT_ACCESS_TOKEN`；不设则用 **`FEISHU_APP_ID`** + **`FEISHU_APP_SECRET`** 换 token |
| 仅打印 token | `token` 子命令 → stdout |
| 审批**文件上传**域名 | `--approval-upload-base` / `FEISHU_APPROVAL_UPLOAD_BASE`（默认 `https://www.feishu.cn`） |
| 其它 | `--timeout-secs`（默认 60）、`--raw`（JSON 不美化） |

`Authorization: Bearer <tenant_access_token>`。勿泄露 `app_secret`。

## 子命令总览

| 命令 | 作用 |
|------|------|
| `token` | 输出 `tenant_access_token` |
| `approval` | 审批定义（当前仅 `get`） |
| `instance` | 实例 CRUD 类、query、cc、cancel、preview、search-cc、specified-rollback、add-sign |
| `task` | approve / reject / transfer / resubmit（含 `task act`）、search、query |
| `comment` | list / create / delete / remove |
| `subscribe` / `unsubscribe` | 按 `approval_code` 订阅或取消（开放平台仍需订阅事件类型） |
| `file` | **`file upload`**：multipart 上传，拿图片/附件控件的 `code` |
| `api` | 任意 **`/open-apis/`** 的 GET / POST(JSON) / DELETE |

路径前缀可省略：写 `approval/v4/...` 或完整 `/open-apis/approval/v4/...`。各子命令完整参数以 **`feishu-approval-tool <子命令> --help`** 为准。

## 常用调用要点

- **`instance create`**：`--form` 或 `--form-file` 为 **整段字符串**（控件 JSON 数组 **再 JSON 序列化一次** 后的字符串）；`--open-id` / `--user-id` 至少其一；可选 `--extra-json` 合并 body。细则 → `embedded-docs/reference/approval-v4/instance/create.md`。
- **图片/附件控件**：先 **`file upload --path <文件> --kind image|attachment`**（可选 `--name`），响应里 `data.code` 写入表单 value。上传走 **`www.feishu.cn`** 域，与 `--base` 无关；国际环境可改 `FEISHU_APPROVAL_UPLOAD_BASE`。
- **任务写操作**：`--user-id-type` 在 HTTP 里是 **query**（默认 `open_id`）。
- **`api`**：用于工具未单独封装的 Open API，例如 `POST approval/v4/approvals`（创建定义）、`districts`、`external_*`、`POST approval/v1/message/send` 等；body/query 形状 Read 对应 `embedded-docs` 单页。
- **`api` 达不到的**：`https://www.feishu.cn/approval/openapi/` 下除 **`v2/file/upload`（已由 `file upload` 封装）** 以外的接口（如灰度 `v1/id/get`、历史 v2/v3）需自写 HTTP 或其它客户端。

## 文档与易错点

- **按场景找页**：`embedded-docs/INDEX.md`。
- **《审批概述》汇总表**可能与单页不一致，**以单页为准**，例如：`POST .../v4/approvals`（非 `/v4/approval`）；指定退回 / 加签为 **`.../instances/specified_rollback`**、**`.../instances/add_sign`**（非 `tasks/...`）；订阅路径含 **`approvals`** 段；抄送事件 type 为 **`approval_cc`**。

## 在线入口（与内嵌同源）

- [审批概述](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval-overview)
- [tenant_access_token（自建应用）](https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/auth-v3/auth/tenant_access_token_internal)
- [事件订阅概述](https://open.feishu.cn/document/ukTMukTMukTM/uUTNz4SN1MjL1UzM)
