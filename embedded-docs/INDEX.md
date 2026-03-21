# embedded-docs 导航

路径均相对于**技能根目录**（与 `SKILL.md` 同级）。需要参数表、示例 JSON、错误码、事件体字段时，用 Read 打开下列文件。

## 按场景

| 场景 | 打开 |
|------|------|
| 总览、方法列表（易有笔误，以子目录单页为准） | `reference/approval-v4/approval-overview.md` |
| 创建/查/列/撤实例、预览、抄送、指定退回（单页） | `reference/approval-v4/instance/*.md` |
| 同意/拒绝/转交/重提、任务查询 | `reference/approval-v4/task/*.md` |
| 评论 | `reference/approval-v4/instance-comment/*.md` |
| 三方定义/实例/任务 | `reference/approval-v4/external_approval/`、`external_instance/`、`external_task/` |
| 行政区 | `reference/approval-v4/district/` |
| 订阅/取消订阅 | `reference/approval-v4/approval/subscribe.md`、`unsubscribe.md` |
| 创建/查看审批定义 | `reference/approval-v4/approval/create.md`、`get.md`、控件说明 `approval-definition-form-control-parameters.md` |
| 实例表单控件取值 | `reference/approval-v4/instance/approval-instance-form-control-parameters.md` |
| 审批文件（图片/附件控件与 code 说明） | `reference/approval-v4/file/overview.md`（上传接口仍见 `extra/uUDOyUjL1gjM14SN4ITN.md`） |
| 原生审批定义 / 实例 **概念与字段总览** | `reference/approval-v4/approval/overview-of-approval-resources.md`、`instance/overview-approval-instance.md` |
| FAQ / 事件 FAQ | `reference/approval-v4/approval-related-faqs.md`、`approval-common-problem/approval-events-faq.md` |
| 上传附件/图片 | `extra/uUDOyUjL1gjM14SN4ITN.md` |
| 灰度 user_id ↔ lark id | `extra/uEDN5UjLxQTO14SM0kTN.md` |
| 审批 Bot 消息 | `extra/ugDNyYjL4QjM24CO0IjN.md`、`extra/uAjNyYjLwYjM24CM2IjN.md` |
| 预览流程 / 加签（`ukTM5UjL5ETO14SOxkTN`；**指定退回**见 `reference/approval-v4/instance/specified_rollback.md`） | `ukTM5UjL5ETO14SOxkTN/approval-preview.md`、`approval-task-addsign.md` |
| 三方快捷回调 / 连接器 | `ukjNyYjL5YjM24SO2IjN/quick-approval-callback.md`、`official-approval-connector.md` |
| Webhook 事件（实例/任务/抄送/业务） | `event/common-event/*.md`、`event/*.md` |

## 常用单页速查

| 能力 | 文件 |
|------|------|
| 创建实例 | `reference/approval-v4/instance/create.md` |
| 实例详情 | `reference/approval-v4/instance/get.md` |
| 批量实例 ID | `reference/approval-v4/instance/list.md` |
| 查询实例列表 | `reference/approval-v4/instance/query.md` |
| 同意任务 | `reference/approval-v4/task/approve.md` |
| 拒绝 | `reference/approval-v4/task/reject.md` |
| 转交 | `reference/approval-v4/task/transfer.md` |
| 重新提交 | `reference/approval-v4/task/resubmit.md` |
| 指定退回 | `reference/approval-v4/instance/specified_rollback.md` |
| 加签 | `ukTM5UjL5ETO14SOxkTN/approval-task-addsign.md` |
| 预览流程 | `ukTM5UjL5ETO14SOxkTN/approval-preview.md`（与 `POST .../instances/preview` 对应） |
| 实例抄送 / 撤回 | `reference/approval-v4/instance/cc.md`、`reference/approval-v4/instance/cancel.md` |
| 任务概述 | `reference/approval-v4/task/introduction.md` |
| 搜任务 / 用户任务列表 | `reference/approval-v4/task/search.md`、`reference/approval-v4/task/query.md` |
| 查抄送列表 | `reference/approval-v4/instance/search_cc.md` |
