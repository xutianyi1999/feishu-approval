# embedded-docs 导航

路径相对技能根（与 **`SKILL.md` 同级）。**勿通读本页**：按场景**只打开一行**。流程与何时打开何文件见 **`docs/AI.md` §1**。

## 按场景

| 场景 | 打开 |
|------|------|
| 流程、子命令、JSON、`approval_code` 映射、排错 | **`docs/AI.md`**（飞书 **`msg`** 可再配合 **`util explain --msg`**） |
| 总览、方法列表（易有笔误，以子目录单页为准） | `reference/approval-v4/approval-overview.md` |
| 实例 API（create/get/list/query/cc/cancel/specified_rollback 等为目录下同名 `.md`） | `reference/approval-v4/instance/` |
| 任务 API（approve/reject/transfer/resubmit/search/query 等为目录下同名 `.md`） | `reference/approval-v4/task/` |
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
| 预览流程 / 加签（`POST .../instances/preview` 等） | `ukTM5UjL5ETO14SOxkTN/approval-preview.md`、`approval-task-addsign.md` |
| 三方快捷回调 / 连接器 | `ukjNyYjL5YjM24SO2IjN/quick-approval-callback.md`、`official-approval-connector.md` |
| Webhook 事件（实例/任务/抄送/业务） | `event/common-event/*.md`、`event/*.md` |
