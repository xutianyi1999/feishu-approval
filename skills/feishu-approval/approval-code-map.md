# 审批显示名称 ↔ approval_code

本文件路径固定：与 **`SKILL.md` 同目录** 下的 **`approval-code-map.md`**（勿改名）。安装与助手工作流见 **`SKILL.md`** 第 2–4 节。安装前由租户管理员填写；之后如有新审批定义，在本表追加行即可。

大模型在用户使用**业务名称**（如「请假」「报销」）而未直接给出 `approval_code` 时，须 **Read 本文件** 查找对应 `approval_code`，**禁止**编造或猜测 code。表中查不到时，请用户补充名称与 code 或更新本文件。

## 如何填写 approval_code

1. **审批管理后台**：打开 [飞书审批管理后台](https://www.feishu.cn/approval/admin/approvalList)，进入某一审批定义的编辑页，从浏览器 **地址栏 URL** 复制 `approval_code`。[官方说明](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval/overview-of-approval-resources#8151e0ae)。
2. **Open API**：创建/查看审批定义等接口响应中的 **`approval_code`**。
3. **已有实例**：`instance get` 响应中的 **`approval_code`** 即对应审批定义。

## 映射表（请删除示例行后填写）

| 审批显示名称（与飞书内一致） | approval_code |
|------------------------------|---------------|
| _示例：请假（请删除）_ | `00000000-0000-0000-0000-000000000001` |
