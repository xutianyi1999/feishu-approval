# AI 使用指南（单一入口）

执行任务以本文件为准。**安装、全局参数、技能包目录**见根目录 **`SKILL.md`**（不必通读全文，按需查表即可）。

### 必须 / 禁止（先扫一眼）

- **禁止**编造或猜测 **`approval_code`**；用户只给中文名时 **Read `approval-code-map.md`**。
- **必须**用 **`approval dump` / `get`** 里的 **`form`** / **`node_list`** 对照控件 **`id`**、**`type`**、选项 **value**；不要凭印象填 `widgets.json`。
- **勿通读** **`embedded-docs/INDEX.md`**：只打开与当前接口相关的**一行**链到单页。
- 怀疑环境或 token：先跑 **`feishu-approval-tool util doctor`**（不打印密钥；会尝试换票）。

## 0. 最小成功路径（推荐顺序）

1. **`feishu-approval-tool util doctor`** — 确认凭证与换票是否成功（可选但省时间）。
2. 有 **`approval_code`** 则 **`approval dump -c <code> --data-only -o approval-data.json`**（或 `get`），从 **`data.form`** 看清控件。
3. （推荐）**`feishu-approval-tool util extract-widgets --json-file approval-data.json`** — 打印紧凑 **控件骨架**（`id` / `type` / `name` / `options` / `children`），比整份 dump 更短、更好扫。
4. 编写 **`widgets.json`**（数组），可先 **`util validate-widgets --json-file widgets.json`** 做形状检查；创建时可选 **`--validate-against-json approval-data.json`** 核对 **id/type 与定义一致**（仍不调定义接口，只读本地 JSON）。
5. **`instance create --widgets-json-file widgets.json ...`**（可加 **`--validate-against-json`**；或 `util form-string` → `--form-file`）。
6. 若 API 报错，对照步骤 2–4；仍缺 HTTP 字段形状 → **`embedded-docs/INDEX.md`** 一行进单页。

## 1. 阅读顺序（能省则省）

| 场景 | 打开 |
|------|------|
| **任何操作** | 本文件 |
| 用户只说审批中文名、未给 code | **`approval-code-map.md`** |
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
| `util doctor` | 打印凭证是否配置（**不打印密钥**），并尝试 **`resolve_tenant_token`**（会访问换票接口） |
| `instance` | get / list / **create**（**`--widgets-json-file`** 或 `--form` / `--form-file` 三选一；可选 **`--validate-against-json`** 对照 dump）/ query / … |
| `task` | act（approve\|reject\|transfer\|resubmit）/ 同名子命令 / search / query |
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

示例控件数组：**`docs/examples/form-widgets.sample.json`**。脱敏的 dump **`data`** 形状示例（勿用于真实租户）：**`docs/examples/approval-data.sample.json`**。由 `widgets.json` 生成 `form.txt`：`util form-string --json-file widgets.json > form.txt`（或 **`--json-file -`** 从 stdin 读）。

### 对照定义与校验（离线）

- **`util extract-widgets --json-file approval-data.json`**：`form` 可为 **字符串**（API 常见）或 **数组**；骨架里 **`options`** 仅当定义里 `option` 为 `{value,text}[]`。
- **`instance create ... --validate-against-json approval-data.json`**：在 POST 前检查 **`widgets.json`（或序列化后的 form 数组）** 里每个控件的 **`id`/`type`** 是否在定义的 **`form` 树**（含 **`children`**）中出现且 **type 一致**。不检查 **value** 是否业务合法。

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
```

自选审批人：在 `extra.json` 里浅合并，例如 **`docs/examples/instance-create-extra.sample.json`**。

## 5. `user_id_type`

**`--user-id-type`** 默认 **`open_id`**。取值：`open_id` | `union_id` | `user_id`（与飞书文档一致）；命令行上的 `user_id` 须与该类型一致。

## 6. 并行建议

已有 `approval_code` 时：**解析用户 open_id** 与 **`approval dump` / `get`** 可并行；组 `widgets.json` 前以 **定义里的控件 id / 选项**为准。

## 7. 常见错误 → 先跑什么

| 现象 | 下一步（一条命令或动作） |
|------|---------------------------|
| 不确定 token / `.env` 是否生效 | **`feishu-approval-tool util doctor`** |
| `No such file or directory` 且参数像 JSON | 改用**真实文件路径**、**`-`**（stdin）、**`--extra-json-inline`** 或 **`api post --json`** |
| `invalid JSON` / parse error | 去 BOM、去尾逗号；stdin 是否为空；对 **`widgets.json`** 可先 **`util validate-widgets --json-file …`** |
| 控件 id / type 与定义不符（或想先缩略阅读） | **`util extract-widgets --json-file approval-data.json`**；创建时加 **`--validate-against-json approval-data.json`** |
| 控件 value 等业务错误 | **`approval get -c <code>`**（或 **`dump --data-only`**）对照 **`form`** 与 **`embedded-docs`** 控件单页 |
| 仍不理解请求体字段 | **`embedded-docs/INDEX.md`** 打开**一行**对应该 API 的单页 |

## 8. 改本仓库代码时

见根目录 **`AGENTS.md`**。
