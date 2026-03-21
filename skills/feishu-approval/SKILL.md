---
name: feishu-approval
description: "PRIORITY: Feishu/Lark workflow approval for agents. Execute via feishu-approval-tool (token, approval|instance|task|comment, subscribe|unsubscribe, api, task act). Env: FEISHU_APP_ID, FEISHU_APP_SECRET, FEISHU_TENANT_ACCESS_TOKEN, FEISHU_OPEN_BASE; .env loaded. Field-level API: embedded-docs/ + INDEX.md. Triggers (zh/en): 飞书审批, 流程审批, 审批单, 审批接口, Lark approval, approval_code, instance_code, tenant_access_token, approve, reject, transfer, resubmit, add_sign, specified_rollback, approval event, 抄送, 加签, 订阅审批."
---

# 飞书审批（技能 + `feishu-approval-tool`）

本技能以 **`feishu-approval-tool` CLI** 为 Agent **执行飞书审批 Open API 的首选方式**；`embedded-docs/` 提供与官方一致的 **请求/响应字段与示例**。回答或集成时：**先按本文件选命令与参数形状，字段细节再 Read 内嵌单页**。

## 0. 何时必须优先用本技能

- 用户问题涉及飞书 / Lark **流程审批、审批 v4 接口、实例与任务、抄送/加签/指定退回、评论、按定义订阅事件、三方同步、行政区** 等 → **必须先遵循本文件 + 按需 Read `embedded-docs/`**，勿仅凭记忆或与 v4 不一致的旧路径作答。
- **能跑命令时**：用 **`feishu-approval-tool`** 发真实请求；勿编造未实现的子命令（以 **第2节** 为准）。
- **阅读顺序**：本文件 **第2节（工具）** → **第5～7节（业务规则与陷阱）** → `embedded-docs/INDEX.md` → 具体 `.md`。

## 1. 技能包与仓库

| 路径 | 用途 |
|------|------|
| `skills/feishu-approval/SKILL.md` | 本文件（工具映射 + API 要点） |
| `skills/feishu-approval/embedded-docs/INDEX.md` | 按场景定位到具体 `.md` |
| `skills/feishu-approval/embedded-docs/` | 官方文档摘录（字段、示例、FAQ） |
| 仓库 `crates/feishu-approval-tool/` | Rust 源码；在**仓库根**执行 `cargo build -p feishu-approval-tool --release`，得到可执行文件 **`feishu-approval-tool`** |

**Cursor**：将 `skills/feishu-approval/` 复制到 `~/.cursor/skills/feishu-approval/`（目录内须有 `SKILL.md` 与 `embedded-docs/`）。

**OpenClaw**：放入 `~/.openclaw/skills/feishu-approval/` 或工作区 `skills/feishu-approval/`；**frontmatter 每键单行**（勿用多行 `description`）。

**路径约定**：细文档只读 **`embedded-docs/`** 下路径（以 `INDEX.md` 为准）。

---

## 2. `feishu-approval-tool` 使用说明（与实现对齐）

### 2.1 全局选项（所有子命令前均可加）

| 长选项 | 环境变量 | 默认 | 含义 |
|--------|-----------|------|------|
| `--base` | `FEISHU_OPEN_BASE` | `https://open.feishu.cn` | 开放平台 host（国际版常用 `https://open.larksuite.com`） |
| `--token` | `FEISHU_TENANT_ACCESS_TOKEN` | 无 | Bearer；若设则**不再**用应用密钥换 token |
| `--timeout-secs` | 无 | `60` | HTTP 超时（秒） |
| `--raw` | 无 | 关 | 响应 JSON 不美化（便于管道） |

程序启动时会加载当前工作目录 **`.env`**（**不覆盖**已存在的环境变量）。

**鉴权顺序**：`--token` → `FEISHU_TENANT_ACCESS_TOKEN` → 用 **`FEISHU_APP_ID`** + **`FEISHU_APP_SECRET`** 调用 `tenant_access_token/internal`（`token` 子命令仅打印该 token）。

### 2.2 一级子命令一览

| 子命令 | 作用 |
|--------|------|
| `token` | 仅输出 `tenant_access_token`（stdout） |
| `approval` | 审批定义 |
| `instance` | 审批实例 |
| `task` | 审批任务 |
| `comment` | 实例评论 |
| `subscribe` / `unsubscribe` | 按 `approval_code` 订阅 / 取消订阅审批事件（仍需在开放平台订阅事件类型） |
| `api` | 任意 `open-apis` 路径的 GET/POST/DELETE（逃生舱） |

### 2.3 `approval`

| 子命令 | HTTP |
|--------|------|
| `approval get -c <approval_code>` | `GET .../approval/v4/approvals/:approval_code` |

### 2.4 `instance`

| 子命令 | HTTP |
|--------|------|
| `instance get -i <instance>` | `GET .../instances/:instance_id`（参数为 `instance_code` 或 uuid） |
| `instance list` | `GET .../instances`（必填 `--approval-code`、`--start-time`、`--end-time`；可选 `--page-size`、`--page-token`） |
| `instance create` | `POST .../instances`（`--form` 或 `--form-file` 为 **字符串化**的表单 JSON 数组；`--open-id` / `--user-id` 至少其一；可选 `--department-id`、`--uuid`、`--extra-json` 合并进 body） |
| `instance query --json-file` | `POST .../instances/query` |
| `instance cc` | `POST .../instances/cc`（`--cc-user-id` 可重复；`--user-id-type` 默认 `open_id`） |
| `instance cancel` | `POST .../instances/cancel` |
| `instance preview --json-file` | `POST .../instances/preview` |
| `instance search-cc --json-file` | `POST .../instances/search_cc`（文档限制见单页） |
| `instance specified-rollback` | `POST .../instances/specified_rollback`（`--task-def-key` 可重复） |
| `instance add-sign` | `POST .../instances/add_sign`（`--add-sign-user-id` 可重复） |

### 2.5 `task`

| 子命令 | HTTP |
|--------|------|
| **`task act --action` `approve` / `reject` / `transfer` / `resubmit`** | 与下行四者等价语义；**transfer** 须 `--transfer-user-id`；**resubmit** 须 `--form` 或 `--form-file` |
| `task approve` / `reject` / `transfer` / `resubmit` | `POST .../tasks/approve` / `reject` / `transfer` / `resubmit`（body 含 `approval_code`、`instance_code`、`user_id`、`task_id`；默认 `--user-id-type open_id`） |
| `task search --json-file` | `POST .../tasks/search` |
| `task query` | `GET .../tasks/query`（`--user-id`、`--topic` 必填；可选分页） |

任务类写接口的 `user_id_type` 在 OpenAPI 上为 **query**；工具会把 **`--user-id-type`** 放到 query。

### 2.6 `comment`

| 子命令 | HTTP |
|--------|------|
| `comment list` | `GET .../instances/:instance/comments` |
| `comment create -i ... --json-file` | `POST .../instances/:instance/comments` |
| `comment delete` | `DELETE .../comments/:comment_id`（query 含 `user_id`、`user_id_type`） |
| `comment remove -i ... --json-file` | `POST .../comments/remove` |

### 2.7 `subscribe` / `unsubscribe`

- `subscribe --approval-code <CODE>` → `POST .../approvals/:approval_code/subscribe`
- `unsubscribe --approval-code <CODE>` → `POST .../approvals/:approval_code/unsubscribe`

### 2.8 `api`（逃生舱）

- `api get <path> [--query KEY=VALUE ...]`：`path` 可写 `approval/v4/...` 或完整 `/open-apis/...`
- `api post <path> [--query ...] (--json \| --json-file)`
- `api delete <path> [--query ...]`

用于 **工具未单独封装的 path**（例如 `approval/v1/message/send`、`POST .../approvals` 创建定义、**三方审批/实例/任务**、**行政区** `districts` 等）：用 **`api get` / `api post` / `api delete`** 并 **Read** `embedded-docs/` 拼 query 与 body。

### 2.9 模型调用策略（建议）

1. **常见读写**：优先 **2.3～2.7 节** 的细命令（参数自解释、易拼）。
2. **三方 / 行政区 / 其它未封装接口**：用 **`api`**（见 **2.8**）+ `embedded-docs/`。
3. **工具未列 path**：**`api`** 或自行 HTTP。
4. **字段含义、条件必填、示例 JSON**：**Read** `embedded-docs/` 对应 `.md`。

---

## 3. 平台连接（与工具一致）

- **鉴权头**：`Authorization: Bearer <tenant_access_token>`（多数接口）；`GET .../tasks/query` 另可用 `user_access_token`（工具当前按 tenant 使用）。
- **Content-Type**：`application/json; charset=utf-8`（POST body）。
- **勿在仓库或对话中硬编码** `app_secret`。

---

## 4. 前置与核心对象

1. 应用开通文档要求 scope；**「开启其中任意一项即可」** = 所列权限 **满足任一**。
2. **`approval_code`**：后台开发者模式 URL 的 `definitionCode`，或接口返回；后台列表：`https://www.feishu.cn/approval/admin/approvalList?devMode=on`。不知 code 时：实例详情 / `instance query` / `task search` 等响应常带 `approval_code`。
3. 建议后台配置控件 **custom_id**、节点 **custom_node_id**，便于 `form` 与自选 `key`。
4. **慎用 `POST .../approvals` 创建原生定义**：创建后难以在后台停用/删除；优先后台建单。见 `embedded-docs/reference/approval-v4/approval/create.md`。

| 对象 | 要点 |
|------|------|
| 审批定义 | `approval_code` |
| 审批实例 | `instance_code`；`GET .../instances/:id` 的 id 为 **instance_code 或 uuid** |
| 任务 | `task_id` ← `instance get` → `task_list[].id` |
| 实例状态 | PENDING / APPROVED / REJECTED / CANCELED / DELETED |
| 任务状态 | PENDING / APPROVED / REJECTED / TRANSFERRED / DONE |

---

## 5. 推荐业务流程与命令对照

1. **看定义**：`approval get -c <CODE>` → 内嵌 `reference/approval-v4/approval/get.md`
2. **（可选）上传附件**：非 `open-apis`，见 **第8节**；内嵌 `extra/uUDOyUjL1gjM14SN4ITN.md`
3. **建单**：`instance create ...` → `instance/create.md`（**`form` 须为字符串化的 JSON 数组**）
4. **预览**：`instance preview --json-file` → `ukTM5UjL5ETO14SOxkTN/approval-preview.md`
5. **详情**：`instance get -i` → `instance/get.md`
6. **抄送 / 撤回**：`instance cc` / `instance cancel`
7. **同意/拒绝/转交/重提**：`task act` 或 `task approve|reject|transfer|resubmit` → `task/*.md`
8. **加签**：`instance add-sign` → `ukTM5UjL5ETO14SOxkTN/approval-task-addsign.md`
9. **指定退回**：`instance specified-rollback` → `instance/specified_rollback.md`（body **无** `approval_code`/`instance_code`）
10. **列表类**：`instance query`、`instance search-cc`、`task search`、`task query`（延迟见单页）

**同意/拒绝 body 摘要**（query 含 `user_id_type`）：`approval_code`、`instance_code`、`user_id`、`task_id` 必填；`comment`、`form`（字符串化数组）按场景可选/必填 → 详见 `task/approve.md`、`reject.md`。

---

## 6. `form` 规则

- API 要求 **字符串**：控件对象数组的 JSON **序列化后再作为字符串字段**传入。
- 控件取值：内嵌 `instance/approval-instance-form-control-parameters.md`。
- 条件分支下任务操作常需额外 `form`。

---

## 7. 事件与订阅

- **审批定义级**：`subscribe` / `unsubscribe` 子命令对应 `POST .../approvals/:approval_code/subscribe|unsubscribe`（总览表常漏写 **`approvals`** 路径段）。应用仍需在开放平台订阅具体 **事件类型**。内嵌 `approval/subscribe.md`。
- **常见事件 type**：`approval_instance`、`approval_task`、**`approval_cc`**（抄送；总览勿与 `approval_task` 混淆）、业务类见 `embedded-docs/event/`。

---

## 8. 工具未直接封装的能力

| 能力 | 说明 |
|------|------|
| 审批文件上传 | `POST https://www.feishu.cn/approval/openapi/v2/file/upload`（multipart，非本 CLI 内置） |
| 灰度 ID 互转 | `POST https://www.feishu.cn/approval/openapi/v1/id/get` |
| `approval/v1/message/send`、`update` | 使用 **`api post`** 或自写 HTTP |
| 创建原生定义 `POST .../approvals` | 建议 **`api post`**；慎用以避免不可删定义 |

---

## 9. 《审批概述》汇总表常见笔误

以 **内嵌单页** 为准：

| 总览写法 | 实际 |
|----------|------|
| `POST .../v4/approval` | `POST .../v4/approvals` |
| `POST .../tasks/specified_rollback` | `POST .../instances/specified_rollback` |
| `POST .../tasks/add_sign` | `POST .../instances/add_sign` |
| `POST .../v4/:approval_code/subscribe` | `POST .../v4/approvals/:approval_code/subscribe` |
| 抄送事件 type `approval_task` | **`approval_cc`** |
| 「专用 ID」与 `tasks/query` 同列 | 实为 **第8节** 灰度 ID 接口（非 `tasks/query`） |

---

## 10. HTTP 路由参考（相对 `https://open.feishu.cn`）

| 方法 | 路径 |
|------|------|
| POST | `/open-apis/approval/v4/approvals` |
| GET | `/open-apis/approval/v4/approvals/:approval_code` |
| POST | `/open-apis/approval/v4/instances` |
| GET | `/open-apis/approval/v4/instances/:instance_id` |
| GET | `/open-apis/approval/v4/instances` |
| POST | `/open-apis/approval/v4/instances/preview` |
| POST | `/open-apis/approval/v4/instances/cc` |
| POST | `/open-apis/approval/v4/instances/cancel` |
| POST | `/open-apis/approval/v4/instances/query` |
| POST | `/open-apis/approval/v4/instances/search_cc` |
| POST | `/open-apis/approval/v4/instances/specified_rollback` |
| POST | `/open-apis/approval/v4/instances/add_sign` |
| POST | `/open-apis/approval/v4/tasks/approve` |
| POST | `/open-apis/approval/v4/tasks/reject` |
| POST | `/open-apis/approval/v4/tasks/transfer` |
| POST | `/open-apis/approval/v4/tasks/resubmit` |
| POST | `/open-apis/approval/v4/tasks/search` |
| GET | `/open-apis/approval/v4/tasks/query` |
| POST | `/open-apis/approval/v4/external_approvals` |
| GET | `/open-apis/approval/v4/external_approvals/:approval_code` |
| POST | `/open-apis/approval/v4/external_instances` |
| POST | `/open-apis/approval/v4/external_instances/check` |
| GET | `/open-apis/approval/v4/external_tasks` |
| GET | `/open-apis/approval/v4/districts` |
| POST | `/open-apis/approval/v4/districts/search` |
| POST/GET/DELETE | `/open-apis/approval/v4/instances/:instance_id/comments`（及 remove/delete） |
| POST | `/open-apis/approval/v1/message/send` |
| POST | `/open-apis/approval/v1/message/update` |
| POST | `/open-apis/approval/v4/approvals/:approval_code/subscribe` |
| POST | `/open-apis/approval/v4/approvals/:approval_code/unsubscribe` |

---

## 11. 错误码与限流（摘要）

| code | 方向 |
|------|------|
| 1390001 | 参数非法 |
| 1390002 / 1390003 | approval / instance 不存在 |
| 1390010 | task |
| 1390015 | 定义未启用 |
| 1390018 | 需客户端能力 |
| 60012 | 创建实例 uuid 冲突 |

限流以内嵌单页与 [频率说明](https://open.feishu.cn/document/ukTMukTMukTM/uUzN04SN3QjL1cDN) 为准。

---

## 12. 在线文档（与内嵌同源）

- [审批概述](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval-overview)
- [创建审批实例](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/instance/create)
- [同意审批任务](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/task/approve)
- [tenant_access_token（自建应用）](https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/auth-v3/auth/tenant_access_token_internal)
- [事件订阅概述](https://open.feishu.cn/document/ukTMukTMukTM/uUTNz4SN1MjL1UzM)
- [三方快捷审批回调](https://open.feishu.cn/document/ukTMukTMukTM/ukjNyYjL5YjM24SO2IjN/quick-approval-callback)
