# 审批显示名称 ↔ approval_code（说明与共享补充）

## 映射表放哪里（推荐）

租户真实的 **「中文名 → `approval_code`」** 表请写在：

**`approval-code-map.local.md`**（与 **`SKILL.md` 同目录**）

- 该文件已列入 **`.gitignore`**，**不会进入 Git**，`git pull` 不会覆盖你的本地表，也不会在合并时与上游冲突。
- **首次**：复制 **`approval-code-map.local.md.example`** 为 **`approval-code-map.local.md`**，再删示例行、填入本企业审批。
- **本文件（`approval-code-map.md`）** 留在仓库里：只放**人人相同的说明**与**非机密**的补充（例如下面的「费用报销」参考）；**不要**把仅贵司使用的映射只写在本文件里，否则每次 pull 仍可能产生合并冲突。

### AI / 自动化怎么查

用户只给审批**中文名**、未给 **`approval_code`** 时：

1. **先** Read **`approval-code-map.local.md`**（若工作区存在该文件）；
2. **否则** Read **本文件**（`approval-code-map.md`）——通常只有说明，**无**你司私有表时需请用户补充或创建 local 文件。

**禁止**编造 **`approval_code`**。查不到则请用户给出 code 或维护 **`approval-code-map.local.md`**（可用 **`util init`** 生成模板）。其余流程见 **`docs/AI.md`**（§0 / §1）。

**校验 code 是否仍有效**（需已配置凭证）：`feishu-approval-tool approval get -c <approval_code>`（或 `approval dump -c <code> --data-only`）；若失败先 **`util doctor`**。

## 如何填写 approval_code

1. **审批管理后台**：打开 [飞书审批管理后台](https://www.feishu.cn/approval/admin/approvalList)，进入某一审批定义的编辑页，从浏览器 **地址栏 URL** 复制 `approval_code`。[官方说明](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval/overview-of-approval-resources#8151e0ae)。
2. **Open API**：创建/查看审批定义等接口响应中的 **`approval_code`**。
3. **已有实例**：`instance get` 响应中的 **`approval_code`** 即对应审批定义。

## 费用报销类：控件形状示例（非映射）

本段**不是** `approval_code` 表。仅当团队常做「费用报销」类发起时，可用下面示例理解 **`fieldList` 二维 `value`、日期 RFC3339、`formula` 非空** 等约定。

- **所有 `id`、子列 `type` 必须以本租户为准**：先 **`approval dump -c <code> --data-only`**，再 **`util extract-widgets`**，把示例里的 `REPLACE_FROM_DUMP_*` 全部换成真实值。
- 示例文件：**`docs/examples/expense-reimbursement-widgets.sample.json`**（与 **`docs/AI.md`** §7 对照阅读）。
