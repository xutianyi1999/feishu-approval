# AI 使用指南（单一入口）

**定位**：CLI 与文档优先服务 **Agent 工作流**；人类同样以本文件为入口。

**只维护一处**：安装、`.env`、技能包目录、**全局参数表** → 根目录 **`SKILL.md`**。子命令 **flags** → **`feishu-approval-tool -h`** / **`<子命令> --help`**（根 `-h` 只有摘要，**以具体子命令的 `--help` 为准**）。改本仓库 Rust → **`AGENTS.md`**。

- **`approval_code`**：只认 **§8** + 本机 **`approval-code-map.local.md`**；仓库无单独映射说明文件。

**文档和 CLI 对不上时**：以 **`feishu-approval-tool <子命令> --help`** 为准。**`--validate-against-json`**（`approval dump --data-only` 或完整 GET JSON）在 **`instance create`** 与 **`util validate-widgets`** 上**同名同义**（先跑形状校验，再核对顶层 **`id`/`type`**）；若帮助里找不到，多半是本地二进制过旧，请在本仓库根 **`cargo install --path crates/feishu-approval-tool`**（可加 **`--locked`**）重装。

## 硬规则（4 条）

1. **`approval_code`**：**禁止**编造。用户只给**审批显示名称** → 按 **§8** 读/写映射文件；无文件或无对应行 → **必须**索要 code；用户给出 code → **必须**落盘（**`util init`** + 表格一行），建议随即 **`approval get -c <code>`**。
2. **控件**：填 **`widgets.json`** 须对照 **`approval dump` / `get`** 的 **`form`**、**`node_list`**（**`id` / `type` / 选项 value**），勿凭印象。
3. **`embedded-docs/INDEX.md`**：只打开与当前接口相关的**一行**，**勿通读**全页。
4. **凭证**：怀疑 token / `.env` → **`util doctor`**（不打印密钥）。

## 0. 最小成功路径

1. **`util doctor`**（可选）。需要本地「中文名 → code」表 → **§8**（**`util init`** 在技能包根执行，或 **`--output-dir <技能包根>`**）。
2. **`approval dump -c <code> --data-only -o approval-data.json`**（或 `get`），从 **`data.form`** 看控件。
3. （推荐）**`util extract-widgets --json-file approval-data.json`**。可选 **`util scaffold-widgets`** 得 **`widgets.json` 模板**。
4. 编辑 **`widgets.json`** → **`util validate-widgets --json-file widgets.json`**，可选 **`--validate-against-json approval-data.json`**（与 **`instance create`** 上该参数**同一检查**；附件数组、`fieldList` 内层、`date` RFC3339 等见 **§7**）。
5. **`instance create --widgets-json-file widgets.json --approval-code … --open-id …`**；若第 4 步未带对照文件，这里仍可加 **`--validate-against-json approval-data.json`**（提交前再拦一层）。或 **`util form-string`** → **`--form-file`**。首次可 **`--dry-run`**。**`code` ≠ 0** 时看 stderr 与 **§7**；飞书原始 **`msg`** 可 **`util explain --msg "…"`**。

**多行命令、`extra.json`、stdin `-`、批量 task**：见 **§3–§4**（勿与下面重复背命令）。

### 0.1 想少几步时

| 情况 | 做法 |
|------|------|
| 控件很少、能接受问答 | **`instance create --wizard`**（仍要 **`approval_code`** / open_id） |
| 费用报销类、只换 id | **`instance create --template expense`**（id 必须与本租户 **dump** 一致） |
| 不想跑 `extract-widgets` | 从 **`docs/examples/`** 拷贝改 **`id`**，或直接照着 **dump 的 `form`** 手写数组 |
| 一次 shell 串起来 | **`dump --data-only`** → 编辑 → **`util validate-widgets --validate-against-json approval-data.json … && instance create …`**（create 上可省略已做过的对照） |

本 CLI 面向 **Open API + JSON**；「谁审批、多少钱」自然语言填表适合在**外层 Agent / 产品**里生成 **`widgets.json`**，再调用本工具（本仓库不内置对话式封装）。

## 1. 阅读顺序

| 场景 | 打开 |
|------|------|
| 默认 | 本文件 |
| 安装 CLI、环境变量、技能包清单、全局参数 | **`SKILL.md`** |
| 仅中文名、未给 `approval_code` | **§8** |
| 控件 value / body 形状 | **`approval dump --data-only`** 的 **`form`** / **`node_list`**；明细等 → **`embedded-docs/.../approval-instance-form-control-parameters.md`** |
| 仍缺 HTTP 请求体字段说明 | **`embedded-docs/INDEX.md`** 选**一行** |
| flags / 长帮助 | **`feishu-approval-tool -h`**、**`<子命令> --help`** |

## 2. 子命令一览（细节以 `--help` 为准）

| 命令组 | 作用 |
|--------|------|
| `token` | 打印 `tenant_access_token` |
| `approval get` | 拉审批定义（终端输出；受 `--raw` 影响） |
| `approval dump` | 同上 GET，**总是美化**；**`-o`** 落盘；**`--data-only`** 只写 **`data`** |
| `util form-string` | 离线：控件 JSON 数组 → 单行 **`form`** 字符串 |
| `util validate-widgets` | 离线：校验数组及 **`fieldList` / `fieldListMobile` 内层**；**`image` / `attachment` / `attachmentV2`** 的 **`value` 须为数组**（勿 `null`，空 **`[]`**）。可选 **`--validate-against-json`**（与 **`instance create`** 同源）。**`date` / `amount` / `formula`** 启发式与 **`--fix`** 见 **`--help`** 与 **§7** |
| `util explain` | 离线：**`--msg`** 粘贴飞书 **`msg`**，打印与 CLI **`code≠0`** 类似的排错提示 |
| `util extract-widgets` | 离线：dump JSON → 控件骨架（`id` / `type` / `name` / `options` / `children`） |
| `util scaffold-widgets` | 离线：dump JSON → 顶层 **`widgets.json` 模板**（**`fieldList`** 从 **`children`** 占位一行） |
| `util doctor` | 凭证摘要 + 尝试换票（不打印密钥） |
| `util init` | 离线：**`--output-dir`**（默认 **`.`**）写入 **`approval-code-map.local.md`**（模板见 **`docs/approval-code-map.local.template.md`**），已存在不覆盖；目录应对准 **§8** 的技能根 |
| `instance` | get / list / **create**（**`--widgets-json-file`** 或 **`--form` / `--form-file`** 或 **`--wizard`** 或 **`--template expense`**；可选 **`--validate-against-json`**（与 **`util validate-widgets`** 同源）、**`--dry-run`**）/ query / … |
| `task` | act 与子命令；**`task reject`**：**`--task-ids`** 或 **`--batch-json-file`**；**`task search`**：**`--json-file`** 为对象，可与 **`--pending-only`** / **`--task-status`** / **`--search-user-id`** 浅合并 |
| `comment` | list / create / delete / remove |
| `subscribe` / `unsubscribe` | 按 `approval_code` |
| `file upload` | 附件/图片 → 表单 `code` |
| `api get\|post\|delete` | 未封装接口；`post` 须 **`--json` 或 `--json-file`** |

## 3. JSON 与文件（全局）

| 参数 | 含义 |
|------|------|
| **`--json-file`**、**`--extra-json`** | **文件路径**，内容为 JSON；**勿**把 `{...}` 当路径 |
| **`-`** | 从 **stdin** 读 JSON |
| **`api post --json`** | 内联 JSON（与 `--json-file` 二选一） |
| **`instance create --extra-json-inline`** | 内联一个 JSON 对象（与 `--extra-json` 二选一） |

**`--form`** 是 API 的字符串；过长用 **`--form-file`**。手里已是控件 **JSON 数组** 时可直接 **`instance create --widgets-json-file`**，不必先 **`util form-string`**。

### 建议文件名

| 文件 | 内容 |
|------|------|
| `approval-data.json` | **`dump --data-only`** 输出 |
| `widgets.json` | 控件数组 |
| `form.txt` | **`util form-string`** 输出（方式 B） |
| `extra.json` | 与 body 浅合并（如自选审批人） |

示例：**`docs/examples/form-widgets.sample.json`**、**`docs/examples/approval-data.sample.json`**。**费用报销形状**（id 须换成本租户）：**`docs/examples/expense-reimbursement-widgets.sample.json`**。

### 对照定义（离线）

- **`util extract-widgets`**：接受 **`form`** 为字符串或数组。
- **`util validate-widgets --validate-against-json approval-data.json`** 与 **`instance create --validate-against-json approval-data.json`**：同一实现，只核对**顶层**控件 **`id`/`type`** 与 **`approval dump --data-only`** 里 **`form`** 树一致；**不**保证 **value** 业务合法（见 **§7**）。习惯上可在 **validate-widgets** 一步做完，**create** 上省略；若只信单次提交前检查，可仅在 **create** 上带该参数。

## 4. `instance create` 与相关

### 4.1 命令模板

```bash
# A：控件数组文件（推荐）
feishu-approval-tool instance create \
  --approval-code YOUR_CODE \
  --widgets-json-file ./widgets.json \
  --validate-against-json ./approval-data.json \
  --open-id ou_xxx \
  --extra-json ./extra.json

# B：已有 form 字符串（如 form.txt）
feishu-approval-tool instance create \
  --approval-code YOUR_CODE \
  --form-file ./form.txt \
  --open-id ou_xxx \
  --extra-json ./extra.json

# C：交互（简单控件）
feishu-approval-tool instance create --approval-code YOUR_CODE --wizard --open-id ou_xxx

# D：内置费用报销形状（id 须换成 dump 里真实 id）
feishu-approval-tool instance create --approval-code YOUR_CODE --template expense --open-id ou_xxx
```

**从定义到创建（管道）**：`approval dump --data-only` → `util extract-widgets` / `scaffold-widgets` → 编辑 **`widgets.json`** → **`util validate-widgets --validate-against-json approval-data.json`** → **`instance create`**（可先 **`--dry-run`**；若上一步已对照，create 可不再传 **`--validate-against-json`**）。**stdin 读 widgets**：**`--json-file -`** 与 **§3** 的 **`-`** 约定。

### 4.2 `extra.json`（`node_approver_*` / `node_cc_*`）

从 **`approval dump`** 取 **`node_list[].node_id`**。外层键须为 **`node_approver_open_id_list`**、**`node_approver_user_id_list`** 或 **`node_cc_*_list`** 等**完整参数名**，**勿**把节点 ID 拼进参数名。值为 **`{ "key": "<node_id>", "value": ["ou_..."] }`** 的数组。

```json
{
  "node_approver_open_id_list": [
    {
      "key": "0cfd07f250e0d105fa5ed9c12fb5a625",
      "value": ["ou_d303844764a301beb7d2828cacb15fec"]
    }
  ]
}
```

完整可复制示例：**`docs/examples/instance-create-extra.sample.json`**（内层结构校验与 CLI 的 **`validate_instance_create_extra_patch`** 同源）。

### `task reject` 批量

```bash
feishu-approval-tool task reject \
  --approval-code CODE --instance-code INST --user-id UID \
  --task-ids id1,id2,id3 \
  --comment "批量拒绝"
```

多实例：**`--batch-json-file`**，数组元素含 `approval_code` / `instance_code` / `user_id` / `task_id`（可选 `comment`）。

### `task search` 待办

Body 须为**对象**（如 **`echo {} > empty.json`** 或 stdin **`{}`**）：

```bash
feishu-approval-tool task search --json-file empty.json --search-user-id ou_xxx --pending-only
feishu-approval-tool task search --json-file body.json --task-status PENDING
```

CLI 会把 **`--pending-only` / `--task-status` / `--search-user-id`** 浅合并进 body。

## 5. `user_id_type`

**`--user-id-type`** 默认 **`open_id`**：`open_id` | `union_id` | `user_id`。

## 6. 并行与渐进调试

- 有 **`approval_code`** 时：**解析 open_id** 与 **`approval dump`** 可并行；**`widgets.json`** 必须以定义为准。
- **复杂表单**：先最少必填 → 逐个加控件 → 单独测 **`fieldList`** → 再叠 **`--extra-json`**。跑通后可建本地 **`templates/`** 存脱敏 JSON（**勿**提交隐私）；仓库 **`docs/examples/`** 仅通用形状。

## 7. 常见错误 → 下一步

### 明细表 `fieldList` / `fieldListMobile`（怎么记）

飞书把**表格**做成一个控件，其 **`value`** 是**二维数组**：**外层 = 行**，**内层 = 列**，每个格子又是一个完整子控件对象（**`id` + `type` + `value`**），和 API 文档一致但第一次写容易漏层。建议：**`util scaffold-widgets`** 或 **dump 里已有空表**先占好格子，再只改叶子 **`value`**；报错路径里的 **`[i][j]`** 即第几行、第几列。

**`instance create`** 与 **`util validate-widgets`** 对 **`form` 解析后的数组**做同源离线检查（**`fieldList` 内层**须 **`id`+`type`+`value`** 等）。**`--validate-against-json`**（两子命令共用）不覆盖 **value** 业务规则。飞书 **「index=0」** 类文案：对照 stderr 路径（如 **`widgets[2].value[0][1]`**），勿默认第一个根控件有错。

**原始飞书报错**：HTTP 层有时只打印平台 JSON；把响应里的 **`msg`**（或整段）粘到 **`util explain --msg`**，可与 CLI 侧 **`[feishu-approval-tool]`** 提示对齐理解。

| 现象 | 下一步 |
|------|--------|
| token / `.env` | **`util doctor`** |
| **`extra.json` 自选审批人无效** | **§4.2**；对照 **`embedded-docs/.../instance/create.md`** |
| 参数像 JSON 却报找不到文件 | 真实路径、**`-`**、**`--extra-json-inline`**、**`api post --json`** |
| JSON 解析失败 | BOM、尾逗号、stdin 是否空；**`util validate-widgets`** |
| id/type 与定义不符 | **`util extract-widgets`**；创建时 **`--validate-against-json`** |
| **明细 / `fieldList`** | **`value`** = 二维数组；每格对象须含 **`id`+`type`+`value`**；**`util validate-widgets`** 或 **`instance create`** 报错路径 |
| **日期** | RFC3339 字符串；勿 Unix 数字、勿仅 `YYYY-MM-DD` |
| **`formula` 空** | API 常要求非空 |
| **`code` ≠ 0**（附件数组、`attachmentV2` 等） | stderr 若有 **`[feishu-approval-tool]`** 优先读；否则 **`util explain --msg`** 粘贴飞书 **`msg`**；**`util validate-widgets`**（**`--fix`**） |
| 其他 value 业务 | **`approval dump --data-only`** 的 **`form`** + **`embedded-docs`** 对应 **type** 单页 |
| 仍不懂请求体字段 | **`embedded-docs/INDEX.md`** **一行** |

## 8. 审批中文名 ↔ `approval_code`

仓库**不**维护 `approval-code-map.md`；映射只在本节 + 本机文件。

- **技能包根**：OpenClaw / 装入方式决定的目录（与 **`SKILL.md` 同级）。下文「根」均指此目录。
- **唯一路径**：**`{根}/approval-code-map.local.md`**（勿写入 `docs/`、`crates/` 等子目录）。**`.gitignore`**，勿提交进 Git。
- **生成空表**：**`docs/approval-code-map.local.template.md`**；或 **`feishu-approval-tool util init`**（**`--output-dir`** 指向根，默认 **`.`** = 当前 shell 目录）。
- **用户只给显示名、未给 code**：Read 上路径 → 无文件 / 无行则**索要 code**（禁猜）→ 用户给 code 后：若无文件先 **`util init`**，再在表中**追加一行**（显示名与用户一致 \| **`approval_code`**），删模板占位行；已有文件则只追加。
- **校验 code**：**`approval get -c <code>`** 或 **`dump --data-only`**；失败 **`util doctor`**。
- **从哪拿 code**：[审批管理后台](https://www.feishu.cn/approval/admin/approvalList) 定义页 URL（[官方说明](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval/overview-of-approval-resources#8151e0ae)）；或 **`instance get`** 等响应。

**费用报销示例 JSON**（非映射表）：**`docs/examples/expense-reimbursement-widgets.sample.json`**（对照 **§7**）。

## 9. 改本仓库代码时

见根目录 **`AGENTS.md`**。

## 10. 开放平台链接（与 embedded 冲突时以单页为准）

- [审批概述](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval-overview)
- [tenant_access_token](https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/auth-v3/auth/tenant_access_token_internal)
- [事件订阅](https://open.feishu.cn/document/ukTMukTMukTM/uUTNz4SN1MjL1UzM)
