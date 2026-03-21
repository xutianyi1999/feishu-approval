---
name: feishu-approval
description: "OpenClaw: Feishu/Lark approval v4 via feishu-approval-tool (token, approval, instance, task, comment, file upload, subscribe/unsubscribe, api). .env + FEISHU_*; name→approval_code only from skill-root approval-code-map.md; field docs via embedded-docs/INDEX.md. Triggers: 飞书审批, Lark approval, approval_code, instance_code, 抄送, 加签, 订阅."
---

# 飞书审批技能（`feishu-approval-tool`）

## 1. 概述

本技能让助手通过 shell 调用 **`feishu-approval-tool`**，访问飞书开放平台 **审批 v4**（`/open-apis/approval/v4/...`）及通用 **`api`** 逃生口。

| 读什么 | 何时读 |
|--------|--------|
| **本文件 `SKILL.md`** | 安装、全局选项、子命令清单、工作流约定 |
| **`approval-code-map.md`**（技能根目录，**勿改名**） | 用户只说审批**中文名**、需要 **`approval_code`** 时 |
| **`embedded-docs/INDEX.md` → 单页** | JSON body、`form` 控件、错误码、事件字段、未封装的接口形状 |

**禁止**臆造子命令或路径；与实现不一致时以仓库 **`crates/feishu-approval-tool`** 及 **`feishu-approval-tool <子命令> --help`** 为准。

---

## 2. 技能包结构

**本仓库**：`SKILL.md`、`approval-code-map.md`、`embedded-docs/` 在**仓库根目录**（与 `crates/` 并列）。

**装入 OpenClaw**：把上述三项放进**同一目录**（建议目录名 `feishu-approval`），再整目录放到技能路径，例如 **`~/.openclaw/skills/feishu-approval/`** 或项目 **`skills/feishu-approval/`**。

| 路径 | 作用 |
|------|------|
| `SKILL.md` | 本说明（frontmatter 每键须**单行**） |
| `approval-code-map.md` | 租户填写：**审批显示名称 ↔ `approval_code`**（模型解析名称的**唯一**来源） |
| `embedded-docs/` | 内嵌开放平台文档片段，供 Read |

---

## 3. 安装前：映射文件

1. 打开 **`approval-code-map.md`**，删除示例行，按表填入本企业数据。新增审批时**只改此文件**。
2. **`approval_code` 从哪来**：见该文件内「如何填写」；也可对照[什么是 Approval Code](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval/overview-of-approval-resources#8151e0ae)。

**原因**：`instance list`、`subscribe` 等接口**必须**带 `approval_code`；用户口语通常是「请假」「报销」，模型**不能**猜 UUID。

---

## 4. 助手（模型）工作流

1. 用户提到审批**名称**且未给 `approval_code` → **Read `approval-code-map.md`**，用表中 code 再调 CLI。表中没有 → 请用户补充并更新该文件，**禁止**编造 code。
2. 用户已给 `approval_code` / `instance_code` → 直接按子命令拼参。
3. 不确定请求体、控件、`form` 格式、错误含义 → **Read `embedded-docs/INDEX.md`** 定位单页。
4. 工具未封装的方法 → **`api` 子命令** + 单页文档核对路径与 body。

---

## 5. 安装 CLI 与运行方式

在仓库根目录：

```bash
cargo install --path crates/feishu-approval-tool
```

（可选 **`--locked`**。二进制默认在 **`~/.cargo/bin`**，需加入 **`PATH`**。）

执行时**工作目录**建议为含 **`.env`** 的目录；工具会加载 `.env`（**不覆盖**已有环境变量）。

---

## 6. 全局参数（所有子命令前均可加）

| 长选项 / 环境变量 | 含义 |
|-------------------|------|
| `--base` / `FEISHU_OPEN_BASE` | 开放平台根 URL，默认 `https://open.feishu.cn`（国际版常用 `https://open.larksuite.com`） |
| `--approval-upload-base` / `FEISHU_APPROVAL_UPLOAD_BASE` | 审批附件上传域，默认 `https://www.feishu.cn` |
| `--token` / `FEISHU_TENANT_ACCESS_TOKEN` | Bearer；不设则用 `FEISHU_APP_ID` + `FEISHU_APP_SECRET` 换 token |
| `--timeout-secs` | HTTP 超时秒数，默认 `60` |
| `--raw` | JSON 不美化，便于管道 |

勿泄露 **`app_secret`**。

---

## 7. 子命令一览（与实现对齐）

### 7.1 `token`

打印 **`tenant_access_token`**（stdout）。

### 7.2 `approval`

| 子命令 | 说明 |
|--------|------|
| `get` | `GET .../approvals/:approval_code`；参数 **`--approval-code` / `-c`** |

### 7.3 `instance`

| 子命令 | 说明 |
|--------|------|
| `get` | `GET` 单实例；**`--instance` / `-i`**（`instance_code` 或创建时的 `uuid`） |
| `list` | `GET` 批量 instance id；**必填** `--approval-code`、`--start-time`、`--end-time`（毫秒）；可选 `--page-size`、`--page-token` |
| `create` | `POST` 创建实例；**`--approval-code`**；**`--form` 或 `--form-file`**；**`--open-id` 与 `--user-id` 至少其一**；可选 `department-id`、`uuid`、`--extra-json` |
| `query` | `POST` 条件查询实例列表；**`--json-file`** |
| `cc` | 抄送；`approval-code`、`instance-code`、`user-id`、可重复 **`--cc-user-id`** 等 |
| `cancel` | 撤回实例 |
| `preview` | 流程预览；**`--json-file`** |
| `search-cc` | 查抄送列表（文档要求自建应用等）；**`--json-file`** |
| `specified-rollback` | 指定退回；`user-id`、`task-id`、可重复 **`--task-def-key`** 等 |
| `add-sign` | 加签；含 `add-sign-user-id`、`add-sign-type` 等 |

### 7.4 `task`

| 子命令 | 说明 |
|--------|------|
| `act` | 统一入口：**`--action approve|reject|transfer|resubmit`**，与其余核心参数同下 |
| `approve` / `reject` / `transfer` / `resubmit` | 与文档一致的单操作封装 |
| `search` | `POST tasks/search`；**`--json-file`** |
| `query` | `GET tasks/query`（用户任务分组）；`user-id`、`topic` 等 |

写操作里的 **`--user-id-type`** 对应 HTTP **query**，默认 **`open_id`**。

### 7.5 `comment`

| 子命令 | 说明 |
|--------|------|
| `list` | **`--instance` / `-i`**、`user-id` 等 |
| `create` / `remove` | **`--json-file`** |
| `delete` | `comment-id`、`user-id` 等 |

### 7.6 `subscribe` / `unsubscribe`

各需 **`--approval-code`**。开放平台侧仍需订阅相应**事件类型**。

### 7.7 `file`

| 子命令 | 说明 |
|--------|------|
| `upload` | **`--path`**、**`--kind image|attachment`**；可选 **`--name`**。响应 **`data.code`** 写入表单控件 value |

上传走 **`--approval-upload-base`**，与 **`--base`** 无关。

### 7.8 `api`（逃生口）

| 子命令 | 说明 |
|--------|------|
| `get` / `post` / `delete` | 第一参数为 path；可多次 **`--query KEY=VALUE`**；`post` 可用 **`--json`** 或 **`--json-file`** |

Path 可写 **`approval/v4/...`** 或完整 **`/open-apis/...`**。用于未单独封装的 Open API（如部分 `approval/v4/approvals`、`districts`、`external_*`、`approval/v1/message/send` 等），形状以 **`embedded-docs`** 单页为准。

**不在此工具范围内**：`https://www.feishu.cn/approval/openapi/` 下除 **`v2/file/upload`**（已由 `file upload` 封装）外的历史/灰度接口，需自写 HTTP。

---

## 8. 高频场景备忘

- **`instance create` 的 `form`**：值为控件 JSON 数组 **再 JSON 序列化一层** 后的**字符串**；细则 → `embedded-docs/reference/approval-v4/instance/create.md`。
- **图片 / 附件控件**：先 **`file upload`**，把返回 **`code`** 写入表单。
- **文档冲突**：开放平台「总览」表格可能与单页不一致，**以 `embedded-docs` 中单页为准**（例如路径里是 **`approvals`**、指定退回与加签在 **`instances/...`**、抄送事件 type **`approval_cc`** 等）。

---

## 9. 内嵌文档入口

- 场景导航：**`embedded-docs/INDEX.md`**
- 常用单页：同 INDEX 中「常用单页速查」表

---

## 10. 在线文档（与内嵌同源）

- [审批概述](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval-overview)
- [tenant_access_token（自建应用）](https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/auth-v3/auth/tenant_access_token_internal)
- [事件订阅概述](https://open.feishu.cn/document/ukTMukTMukTM/uUTNz4SN1MjL1UzM)
