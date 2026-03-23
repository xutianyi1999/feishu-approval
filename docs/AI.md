# AI 使用指南（单一入口）

**定位（AI-first）**：本仓库 CLI 与文档优先服务 **被技能/Agent 调用的工作流**（结构化命令、贴近 API）；人类偶尔用时同样读本文件即可，不必按「消费级产品」预期去猜交互。

执行任务以本文件为准。**「哪份文档何时打开」的决策表以 §1 为准**（`SKILL.md` 仅作交叉引用）。**安装 CLI、全局参数、技能包应包含哪些文件**见根目录 **`SKILL.md`**（「技能包目录」与参数表；不必通读全文）。**审批中文名 ↔ `approval_code`** 只维护本节 **§8** + 本机 **`approval-code-map.local.md`**，**无**单独的仓库级映射说明文件。

### 必须 / 禁止（先扫一眼）

- **禁止**编造或猜测 **`approval_code`**。用户只给**审批显示名称**、未给 code 时：**先 Read 根目录 `approval-code-map.local.md`**（若工作区有）。**若该文件不存在**，**必须**向用户索要 **`approval_code`**（不得凭名称继续调 API）。用户给出 code 后：**必须**在本工作区**创建或更新** **`approval-code-map.local.md`**（无文件时先 **`feishu-approval-tool util init`** 再编辑表格新增一行；已有文件则追加一行），写入「显示名称（与用户表述一致）↔ code」。**建议**随即 **`approval get -c <code>`** 校验 code 有效。约定与细节见 **§8**。
- **必须**用 **`approval dump` / `get`** 里的 **`form`** / **`node_list`** 对照控件 **`id`**、**`type`**、选项 **value**；不要凭印象填 `widgets.json`。
- **勿通读** **`embedded-docs/INDEX.md`**：只打开与当前接口相关的**一行**链到单页。
- 怀疑环境或 token：先跑 **`feishu-approval-tool util doctor`**（不打印密钥；会尝试换票）。

### 第一次用（少在文档间跳转）

- **本文件**：执行流程、子命令、§7 排错；**`SKILL.md`**：安装 CLI、全局参数、技能包应含路径。
- 仅**中文名**无 **`approval_code`**：先 Read **`approval-code-map.local.md`**；无此文件则**必须问用户要 code**，用户提供后**写入该文件**（**§8**）；缺 HTTP 字段形状：**`embedded-docs/INDEX.md`** 选**一行**。
- 主路径 **§0**；捷径：**`util init`**、**`instance create --wizard`**、**`--template expense`**。

## 0. 最小成功路径（推荐顺序）

1. **`feishu-approval-tool util doctor`** — 确认凭证与换票是否成功（可选但省时间）。无本地映射文件时可先 **`util init`** 再编辑 **`approval-code-map.local.md`**。
2. 有 **`approval_code`** 则 **`approval dump -c <code> --data-only -o approval-data.json`**（或 `get`），从 **`data.form`** 看清控件。
3. （推荐）**`feishu-approval-tool util extract-widgets --json-file approval-data.json`** — 打印紧凑 **控件骨架**（`id` / `type` / `name` / `options` / `children`），比整份 dump 更短、更好扫。
3b. （可选）**`feishu-approval-tool util scaffold-widgets --json-file approval-data.json`** — 生成仅含顶层控件的 **`widgets.json` 模板**（`value`: `null`），再按骨架与 §7 补全。
4. 编写 **`widgets.json`**（数组），可先 **`util validate-widgets --json-file widgets.json`** 做形状检查；创建时可选 **`--validate-against-json approval-data.json`** 核对 **id/type 与定义一致**（仍不调定义接口，只读本地 JSON）。**`value` 内部结构**（明细二维数组、日期字符串格式等）工具**不校验**，见 **§7**。
5. **`instance create --widgets-json-file widgets.json ...`**（可加 **`--validate-against-json`**；或 `util form-string` → `--form-file`）。
6. 若 API 报错：CLI 在飞书 **`code` ≠ 0** 时会 **non-zero 退出**，stderr 中除官方 **`msg`** 外可能附带 **§7 类提示**（仍须按控件 **type** 自查）。对照步骤 2–4 与 **§7**；仍缺 HTTP 字段形状 → **`embedded-docs/INDEX.md`** 一行进单页。

## 1. 阅读顺序（能省则省）

**本表为「文档怎么读」唯一详表**（根目录 **`SKILL.md`** 仅指向此处，避免重复维护）。

| 场景 | 打开 |
|------|------|
| **任何操作** | 本文件 |
| 安装 CLI、`.env`、全局 flags 表、技能包应含哪些路径 | 根目录 **`SKILL.md`** |
| 用户只说审批中文名、未给 code | **Read `approval-code-map.local.md`**；**若无此文件 → 必须向用户索要 `approval_code` → 用户提供后创建/更新该文件**（可先 **`util init`**）；约定 **§8** |
| 控件 value、fieldList、body | **`approval dump -c <code> --data-only -o approval-data.json`**（或 `dump` / `get`）看 `form` / `node_list`；复杂类型再点开 **`embedded-docs/.../approval-instance-form-control-parameters.md`** |
| 仍缺 HTTP 形状 | **`embedded-docs/INDEX.md`** 中**一行**（勿通读 INDEX） |
| flags / `after_long_help` | **`feishu-approval-tool -h`**、**`<子命令> --help`** |

## 2. 子命令一览（细节以 `--help` 为准）

| 命令组 | 作用 |
|--------|------|
| `token` | 打印 `tenant_access_token` |
| `approval get` | 拉审批定义（终端输出；受 `--raw` 影响） |
| `approval dump` | 同上 GET，**总是美化 JSON**；**`-o path`** 落盘；**`--data-only`** 只写响应里的 **`data`**（更小，适合 Read） |
| `util form-string` | **不调飞书**：读入控件 **JSON 数组**（文件或 `-`），打印 **`form` 单行字符串** → 配合 `instance create --form-file` |
| `util validate-widgets` | **不调飞书**：检查控件数组是否每项含 **`id` / `type` / `value` 键**（形状检查，不保证飞书业务侧通过） |
| `util extract-widgets` | **不调飞书**：读 **`approval dump --data-only`**（或带 **`data`** 的完整 GET JSON），打印 **控件骨架** 美化 JSON |
| `util scaffold-widgets` | **不调飞书**：读同上 dump JSON，打印顶层 **`widgets.json` 模板**（`id` / `type` / `value`: `null`）；**`fieldList` 子列**须手写进二维 **`value`** |
| `util doctor` | 打印凭证是否配置（**不打印密钥**），并尝试 **`resolve_tenant_token`**（会访问换票接口） |
| `util init` | **不调飞书**：在指定目录（默认当前目录）写入 **`approval-code-map.local.md`**（内容与 **`docs/approval-code-map.local.template.md`** 相同），已存在则**不覆盖** |
| `instance` | get / list / **create**（**`--widgets-json-file`** 或 `--form` / `--form-file` 或 **`--wizard`** 或 **`--template expense`** 四选一；可选 **`--validate-against-json`**）/ query / … |
| `task` | act（approve\|reject\|transfer\|resubmit）/ 同名子命令；**`task reject`** 支持 **`--task-ids`**（逗号分隔，同一审批/实例/用户）或 **`--batch-json-file`**（JSON 数组，每行含 `approval_code` / `instance_code` / `user_id` / `task_id`）；**`task search`** 的 **`--json-file`** 须为 **JSON 对象**，可与 **`--pending-only`** 或 **`--task-status`**、**`--search-user-id`** 浅合并进 body |
| `comment` | list / create / delete / remove |
| `subscribe` / `unsubscribe` | 按 `approval_code` |
| `file upload` | 附件/图片 → 表单 `code` |
| `api get\|post\|delete` | 未封装接口；`post` 须 **`--json` 或 `--json-file`** |

## 3. JSON 与文件（全局）

| 参数 | 含义 |
|------|------|
| **`--json-file`**、**`--extra-json`** | **文件路径**，内容为 JSON；**不要**把 `{...}` 当路径传 |
| **`-`** | 从 **stdin** 读 JSON |
| **`api post --json`** | 内联 JSON 字符串（与 `--json-file` 二选一） |
| **`instance create --extra-json-inline`** | 内联 **一个 JSON 对象**（与 `--extra-json` 二选一） |

`--form` 是 API 要求的**字符串**（控件数组再序列化一层）；过长用 **`--form-file`**。若手里是 **JSON 数组文件**（同 `util form-string` 输入），创建实例时可 **`instance create --widgets-json-file widgets.json`**，**不必**先跑 `util form-string`。

### 建议文件名（团队可对齐用语义）

| 文件 | 内容 |
|------|------|
| `approval-data.json` | `approval dump -c CODE --data-only -o approval-data.json`（仅 `data`，省 token） |
| `approval-full.json` | `approval dump -c CODE -o approval-full.json`（完整响应，需看 `code`/`msg` 时） |
| `widgets.json` | 控件 **数组** `[{ "id", "type", "value" ...}]`（人类/模型好写） |
| `form.txt` | 由 `widgets.json` 经 `util form-string` 生成（见 §4 方式 B） |
| `extra.json` | 与 body 浅合并的对象（如 `node_approver_open_id_list`） |

示例控件数组：**`docs/examples/form-widgets.sample.json`**。脱敏的 dump **`data`** 形状示例（勿用于真实租户）：**`docs/examples/approval-data.sample.json`**。**费用报销类**常见 **fieldList + date + formula** 形状（**`id`/`type` 须换成本租户 dump**，见 **§8** 末段）：**`docs/examples/expense-reimbursement-widgets.sample.json`**。由 `widgets.json` 生成 `form.txt`：`util form-string --json-file widgets.json > form.txt`（或 **`--json-file -`** 从 stdin 读）。

### 对照定义与校验（离线）

- **`util extract-widgets --json-file approval-data.json`**：`form` 可为 **字符串**（API 常见）或 **数组**；骨架里 **`options`** 仅当定义里 `option` 为 `{value,text}[]`。
- **`instance create ... --validate-against-json approval-data.json`**：在 POST 前检查 **`widgets.json`（或序列化后的 form 数组）** 里每个控件的 **`id`/`type`** 是否在定义的 **`form` 树**（含 **`children`**）中出现且 **type 一致**。不检查 **`value` 嵌套形状或格式**（明细数组维度、日期 RFC3339 等），见 **§7**。

## 4. `instance create` 模板

```bash
# 方式 A：控件数组文件（推荐，少一步；等同 util form-string 再 --form-file）
feishu-approval-tool instance create \
  --approval-code YOUR_CODE \
  --widgets-json-file ./widgets.json \
  --validate-against-json ./approval-data.json \
  --open-id ou_xxx \
  --extra-json ./extra.json

# 方式 B：已是 API form 字符串（如由 §3 的 util form-string 得到 form.txt）
feishu-approval-tool instance create \
  --approval-code YOUR_CODE \
  --form-file ./form.txt \
  --open-id ou_xxx \
  --extra-json ./extra.json

# 方式 C：交互问答（简单控件；fieldList/图片/附件见提示）
feishu-approval-tool instance create \
  --approval-code YOUR_CODE \
  --wizard \
  --open-id ou_xxx

# 方式 D：内置费用报销**形状**示例（须把控件 id 换成你租户 dump 里的真实 id）
feishu-approval-tool instance create \
  --approval-code YOUR_CODE \
  --template expense \
  --open-id ou_xxx
```

自选审批人：在 `extra.json` 里浅合并，例如 **`docs/examples/instance-create-extra.sample.json`**。

### `node_approver_*_list` / `node_cc_*_list`（易错：不要凭参数名猜内部字段）

Open API 文档里，**`node_approver_open_id_list`** 等字段的值是**对象数组**；每个元素必须是 **`key` + `value`**：

- **`key`**：流程节点的 **`node_id`**（或文档所称 custom 节点 id），来自 **`approval get` / `approval dump`** 的 **`node_list`**。
- **`value`**：**该节点上的审批人**的 **`open_id`（或 user_id）字符串数组**。

文档参数表把「key」「value」写成子行，容易直觉写成 **`node_id`** + **`approver_open_id_list`** 这类**不存在的**内层字段名。请以 **`embedded-docs/reference/approval-v4/instance/create.md`** 文末 **请求体示例** 为准；本地示例 **`docs/examples/instance-create-extra.sample.json`**。

使用 **`instance create --extra-json` / `--extra-json-inline`** 时，CLI 会在发送前对上述列表做**常见误写字段**检查（如内层出现 `node_id`）；仍建议对照官方 JSON 示例。

### `task reject` 批量（飞书每次请求仍要带齐四个字段）

同一 **`approval_code` / `instance_code` / `user_id`** 下多个任务：

```bash
feishu-approval-tool task reject \
  --approval-code CODE --instance-code INST --user-id UID \
  --task-ids id1,id2,id3 \
  --comment "批量拒绝"
```

多实例混排用 **`--batch-json-file`**，内容为数组，元素形如：
`{ "approval_code": "…", "instance_code": "…", "user_id": "…", "task_id": "…", "comment": "可选" }`。

### `task search` 只看待办

请求体须为**对象**（可用 **`echo {} > empty.json`** 或 **`--json-file -`** 与 stdin 的 `{}`）：

```bash
feishu-approval-tool task search --json-file empty.json --search-user-id ou_xxx --pending-only
# 或显式状态（与飞书文档一致，如 PENDING）
feishu-approval-tool task search --json-file body.json --task-status PENDING
```

CLI 会把你传的 **`--pending-only` / `--task-status` / `--search-user-id`** 写进 body（后写覆盖 JSON 文件里同名字段）。

## 5. `user_id_type`

**`--user-id-type`** 默认 **`open_id`**。取值：`open_id` | `union_id` | `user_id`（与飞书文档一致）；命令行上的 `user_id` 须与该类型一致。

## 6. 并行建议

已有 `approval_code` 时：**解析用户 open_id** 与 **`approval dump` / `get`** 可并行；组 `widgets.json` 前以 **定义里的控件 id / 选项**为准。

## 7. 常见错误 → 先跑什么

以下为 **value 形状/格式** 的典型坑（`--validate-against-json` 只保证 **id/type**，**不**保证 **value** 合法）：

| 现象 | 下一步（一条命令或动作） |
|------|---------------------------|
| 不确定 token / `.env` 是否生效 | **`feishu-approval-tool util doctor`** |
| **`instance create` + `extra.json` 自选审批人报错 / 参数无效** | 核对 **`node_approver_*` / `node_cc_*`** 数组元素是否为 **`key` + `value`**（见上文 **§4** 易错段与 **`instance/create.md`** 请求体示例），勿写成内层 **`node_id`** / **`approver_open_id_list`** |
| `No such file or directory` 且参数像 JSON | 改用**真实文件路径**、**`-`**（stdin）、**`--extra-json-inline`** 或 **`api post --json`** |
| `invalid JSON` / parse error | 去 BOM、去尾逗号；stdin 是否为空；对 **`widgets.json`** 可先 **`util validate-widgets --json-file …`** |
| 控件 id / type 与定义不符（或想先缩略阅读） | **`util extract-widgets --json-file approval-data.json`**；创建时加 **`--validate-against-json approval-data.json`** |
| **明细 / `fieldList` 类**：报错指向 value 结构、或期望「数组的数组」 | **`value` 为二维数组**：外层 = 行，内层 = 该行内各子控件的值对象；**勿**写成单层扁平对象。对照 **`approval dump`** 的 **`form`** 与 **`embedded-docs/.../approval-instance-form-control-parameters.md`**（fieldList） |
| **日期类**：格式无效、类型不符 | **`value` 为 RFC3339 字符串**（例：`2025-03-23T10:00:00+08:00`）；**勿**用 Unix **时间戳数字**，也**勿**只写 `YYYY-MM-DD`。对照同上 **`form`** 与控件参数单页 |
| **formula**：空值、不合法 | 创建实例时 API **常要求 `value` 非空**（可与金额/计算结果一致），**勿**因「后台自动算」而省略 |
| 飞书响应 **`code` ≠ 0**（如「控件值不合法或者为空」） | 读 stderr 全文（含 CLI 附带的 **§7 提示**）；用 **`util extract-widgets`** 按 **id** 查 **type**，再对照明细/日期/formula 等规则改 **`value`** |
| 其他控件 value 业务错误 | **`approval get -c <code>`**（或 **`dump --data-only`**）对照 **`form`** 与 **`embedded-docs`** 对应 **type** 单页 |
| 仍不理解请求体字段 | **`embedded-docs/INDEX.md`** 打开**一行**对应该 API 的单页 |

## 8. 审批中文名 ↔ `approval_code`

仓库内**不再**单独维护 `approval-code-map.md`；映射规则与查表顺序只读本节。

- **私有表**：根目录 **`approval-code-map.local.md`**（已在 **`.gitignore`**，**不入库**）。**不要**把仅本企业的「中文名 → code」提交进 Git，以免合并冲突。
- **模板**：仓库 **`docs/approval-code-map.local.template.md`**，或 **`feishu-approval-tool util init`**（在目标目录生成 **`approval-code-map.local.md`**；已存在则**不覆盖**）。
- **查表与强制索要 code**：用户只给**审批显示名称**、未给 **`approval_code`** 时：
  1. **Read** 工作区根目录 **`approval-code-map.local.md`**（若存在）。
  2. **若文件不存在**：**必须**请用户提供 **`approval_code`**（可一并提示用户从管理后台 URL 或审批定义里复制，见下条）。**禁止**编造、猜测或用名称继续调用需 code 的 CLI。
  3. **用户给出 code 后**：**必须**落盘映射——若尚无 **`approval-code-map.local.md`**：先在工作区根（或当前技能根）执行 **`feishu-approval-tool util init`**，再 **编辑**生成的文件，在「映射表」中增加一行（两列表格：**审批显示名称**与用户说法一致，第二列为用户提供的 **`approval_code`**）；删除模板里占位示例行（若仍保留）。若文件已存在但**没有**该名称对应行：同样**追加一行**，勿覆盖他人行。
  4. **建议**：写入后用 **`approval get -c <code>`** 或 **`approval dump -c <code> --data-only`** 校验；失败先 **`util doctor`**。
- **如何拿到 `approval_code`**（可发给用户作说明）：[飞书审批管理后台](https://www.feishu.cn/approval/admin/approvalList) 打开定义编辑页，从地址栏 URL 读取（[官方说明](https://open.feishu.cn/document/uAjLw4CM/ukTMukTMukTM/reference/approval-v4/approval/overview-of-approval-resources#8151e0ae)）；或 Open API / **`instance get`** 响应中的字段。
- **校验 code**：**`approval get -c <code>`** 或 **`approval dump -c <code> --data-only`**；失败先 **`util doctor`**。

**费用报销控件形状**（不是映射表）：**`docs/examples/expense-reimbursement-widgets.sample.json`**，与 **§7** 对照；所有 id 以本租户 dump 为准。

## 9. 改本仓库代码时

见根目录 **`AGENTS.md`**。
