# 审批显示名称 ↔ approval_code

与 **`SKILL.md` 同目录**，勿改名。租户管理员维护：有新审批定义时在本表追加行。用户使用**业务名称**而未给 `approval_code` 时须 **Read 本文件** 查表，**禁止**编造；查不到则请用户补充或更新本表。其余流程见 **`docs/AI.md`**。

**校验 code 是否仍有效**（需已配置凭证）：`feishu-approval-tool approval get -c <approval_code>`（或 `approval dump -c <code> --data-only`）；若失败先 **`util doctor`**。

## 如何填写 approval_code

1. **审批管理后台**：打开 [飞书审批管理后台](https://www.feishu.cn/approval/admin/approvalList)，进入某一审批定义的编辑页，从浏览器 **地址栏 URL** 复制 `approval_code`。[官方说明](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval/overview-of-approval-resources#8151e0ae)。
2. **Open API**：创建/查看审批定义等接口响应中的 **`approval_code`**。
3. **已有实例**：`instance get` 响应中的 **`approval_code`** 即对应审批定义。

## 映射表（请删除示例行后填写）

| 审批显示名称（与飞书内一致） | approval_code |
|------------------------------|---------------|
| _示例：请假（请删除）_ | `00000000-0000-0000-0000-000000000001` |

## 费用报销类：控件形状示例（非映射）

本段**不是** `approval_code` 表。仅当团队常做「费用报销」类发起时，可用下面示例理解 **`fieldList` 二维 `value`、日期 RFC3339、`formula` 非空** 等约定。

- **所有 `id`、子列 `type` 必须以本租户为准**：先 **`approval dump -c <code> --data-only`**，再 **`util extract-widgets`**，把示例里的 `REPLACE_FROM_DUMP_*` 全部换成真实值。
- 示例文件：**`docs/examples/expense-reimbursement-widgets.sample.json`**（与 **`docs/AI.md`** §7 对照阅读）。
