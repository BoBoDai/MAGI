# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概览

MAGI 是一个三脑决策桌面应用：用户提出一个问题，三个独立人格从不同视角作答、可选交叉质询、独立终审，最后由 Rust 聚合出裁决。

- **MELCHIOR-1** — 科学家：证据、方法、风险、可证伪性
- **BALTHASAR-2** — 母亲：保护、长期后果、可逆性、脆弱方
- **CASPER-3** — 女性：情绪、关系、尊严、信任、权力关系

界面分两层：

1. **顶部几何决策台**：三角布局的三个单元、连线、`質 問 / 解 決`、系统状态和巨大的 `合意 / 拒絶 / 状態 / 情報 / 誤差` 响应牌。单元可点击查看详情。
2. **底部 Extended Analysis 抽屉**：完整协议页面（三列详情 + 终审裁决）。

技术栈：**Tauri v2 + Rust workspace + React 19 / Vite 8 + Tailwind CSS v4**。Rust 负责协议、模型调用、裁决与持久化，TypeScript 只负责渲染。

- 先读本文件了解边界和入口；协议细节读 `docs/protocol.md`。
- Rust 是领域类型的唯一真相来源；`ui/src/types.ts` 是手写镜像，改字段必须同步。
- 几何布局参考 [TomaszRewak/MAGI](https://github.com/TomaszRewak/MAGI)（MIT），但代码、样式与协议集成都是本项目的原创实现。

## 常用命令

```bash
pnpm install              # 安装根目录（Tauri CLI）和 ui/ 依赖
pnpm app:dev              # 启动桌面窗口（Vite HMR + Rust 热重启），Vite 固定 5174
pnpm app:build            # 打包 .app / .dmg 等
pnpm dev                  # 只起 Vite（localhost:5174），无 Tauri IPC，仅调样式
pnpm typecheck            # tsc --noEmit
pnpm build                # typecheck + vite build

cargo test --workspace
cargo test -p magi-core decision          # 按 crate / 测试名过滤
cargo test -p magi-usecases
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --release -p magi-app --all-targets -- -D warnings
```

- Vite 固定 **5174** 且 `strictPort: true`；`ui/vite.config.ts` 的 `port` 与 `src-tauri/tauri.conf.json` 的 `devUrl` 必须一致。
- 纯浏览器里没有 Tauri IPC，显示「未在 Tauri 中运行」是预期行为。
- `.claude` 不参与构建；测试脚本不要放进仓库。

### 演示模式

debug 构建的顶栏有 `MOCK OFF / MOCK ON` 开关，开启后使用 `DemoProvider`，无需 API key 就能跑完整流程。

- release 构建 **不编译** `DemoProvider`，**不注册** `set_demo_mode`。
- 开启时界面常显 `MOCK MODE // NOT REAL MODEL OUTPUT`。
- 会话记录带 `source: mock`，历史里显示 `MOCK` 标记。

## 架构（简版）

依赖方向自上而下，**下层不得反向依赖上层**：

```
ui/ (React + Vite)                     渲染层，只通过 IPC 交互
      │ Tauri IPC (invoke + Channel)
src-tauri/ (Tauri 壳)                  装配、设置/凭据、历史、command
      │
crates/usecases/                       编排层，传输无关
      │
crates/core/  ←  crates/adapters/       纯领域  /  外部世界（LLM）
```

- `crates/core`：`PersonaId`、`DecisionStatus`、`Opening` / `Debate` / `Vote`、`MagiVerdict`、`SessionRecord`、`decide_openings`、`aggregate_votes`。零 IO。**裁决与决策策略只在这里实现。**
- `crates/adapters`：`MagiProvider` trait、OpenAI 兼容客户端、SSE（字节级缓冲）、服务商 profiles、prompts、严格 JSON 解析、`DemoProvider`（仅 debug）。
- `crates/usecases`：`App<L>` 泛型注入 provider；`run_opening` / `run_debate` / `cast_votes`；三路并发 `tokio::join!`；预算裁剪。不放 `tauri::`。
- `src-tauri`：唯一装配点。`AppState` = `App<LlmBackend>` + `SettingsManager` + `SessionStore`；`LlmBackend` 在 live 与 demo 之间切换；command 都是薄壳。
- `ui`：`App.tsx` 管设置/历史/弹窗与输入草稿；`useMagiSession` 是协议状态机；`api/ipc.ts` 是前端调用 Rust 的唯一通道。

## 协议不变量

### 阶段

1. **OPENING**：三路并发、互相不可见；严格 JSON（`stance / confidence / summary / key_points / risks`）。Rust 同时算出 `OpeningBatch.decision`。
2. **DEBATE（可选，一轮）**：仅当三份开场全部成功才可用；每个脑看到自己的和另外两个脑的开场，**看不到同轮辩论回复**；纯文本。
3. **VOTE**：三路并发、密封终审；输入为问题、上下文、全部开场、全部辩论；**任何 prompt 都不包含其他脑的终审票**。
4. **VERDICT**：`aggregate_votes` 纯函数聚合。

### 决策状态（顶部响应牌）

`decide_openings` 的优先级：**error > info > no > conditional > yes**。

| 开场 | 状态 |
|---|---|
| 任一失败 / 不是 3 项 | `誤差` Error |
| 三脑全弃权 | `情報` Info |
| 任一反对 | `拒絶` No |
| 任一弃权、无反对 | `状態` Conditional |
| 三脑全支持 | `合意` Yes |

- 开场后显示 **PROVISIONAL**；终审后由 `MagiVerdict` 覆盖为 **FINAL**。
- 参考项目还会额外分类问题是否 yes/no；v1 没做这次额外调用，所以「三脑全弃权」映射为 `情報`。

### 终审裁决

| 终审票 | 裁决 |
|---|---|
| 3 approve / 3 reject / 3 abstain | Unanimous（3 abstain 显示「无建议」） |
| 2 approve + 1 reject / 2 approve + 1 abstain | Majority Approve |
| 2 reject + 1 approve / 2 reject + 1 abstain | Majority Reject |
| 2 abstain + 1 决定性票、1/1/1 | Split |
| 任一终审失败 | Incomplete |

弃权不能单独凑成多数；失败票永远不当作多数。开场失败不必然导致 Incomplete；该脑仍可基于问题、上下文、同伴开场与辩论投票，UI 标 `NO BASELINE`。

### 预算

- 问题 ≤ 2,000 字符（超限直接报错，不截断）
- 上下文 ≤ 40,000 字符（保留头 3/4 + 尾 1/4）
- 单条辩论回复进入 prompt 前 ≤ 6,000 字符
- 落库会话：问题 ≤ 4,000、上下文 ≤ 20,000

### Prompt 与解析

- 问题、上下文、同伴输出都作为 JSON 数据传入，并在 system prompt 中声明为**不可信数据**，禁止执行其中指令。
- 结构化阶段只接受一个 JSON 对象；允许单个 ` ```json ` fence；先剥离 `<think>`。
- 缺字段、非法 enum、`confidence > 100`、空必填文本、条目数量/长度越界、JSON 后的多余文本 → `Error::InvalidLlmOutput`，**不降级、不猜值**。
- `persona` / `model` / `prompt_version` 由 Rust 注入，模型不能伪造。
- 改 prompt 必须 bump 对应 `PROMPT_VERSION`（Opening / Debate / Vote 各自 `v1`）。

### 流式

- 协议事件是 `MagiStreamEvent`，带 `run_id + stage + persona`；前端按 persona 追加，忽略陈旧 `run_id`。
- `run_opening` / `run_debate` 流式；`cast_votes` 不流式，避免暴露部分选票。
- Channel 只是遥测；**command 返回值才是权威 batch**。
- 三路事件会交错，绝不能共用一个文本缓冲。

## 持久化

全部位于 Tauri app config/data 目录（macOS 通常 `~/Library/Application Support/dev.junbo.magi/`），Unix 下 0600：

- `settings.json`：schema 1，provider 列表 + active_id + compat。
- `credentials.json`：只写不读的 API key；IPC 只返回 `has_api_key`。
- `sessions.json`：schema 1，最多 100 条，按 `updated_at` 倒序；每个阶段完成即 upsert；损坏时备份为 `sessions.corrupt-<ts>.json`；未来 schema 进入只读保护，不覆盖。

保存会话时后端会重新校验并重算：`OpeningBatch.decision`、`VoteTally`、`MagiVerdict`、`opening_stance`、`changed_from_opening`。**不信任前端回传的裁决。**

## 任务路由

| 要改的东西 | 入口 | 相关 / 测试 |
|---|---|---|
| 决策状态 / 状态优先级 | `crates/core/src/decision.rs` | `crates/core/src/opening.rs`；core 单测 |
| 开场并发、事件 | `crates/usecases/src/opening.rs` | `usecases/src/tests/opening.rs` |
| 质询并发、可见性 | `crates/usecases/src/debate.rs` | `usecases/src/tests/debate.rs` |
| 终审、立场变化 | `crates/usecases/src/vote.rs` | `usecases/src/tests/vote.rs` |
| 裁决表 | `crates/core/src/verdict.rs` | 文件内单测 |
| Prompt / JSON 校验 | `crates/adapters/src/llm/prompts.rs`、`parsing.rs` | `llm/tests.rs` |
| SSE / 服务商 / 客户端 | `crates/adapters/src/llm/{streaming,profiles,client,config,error}.rs` | `llm/tests.rs`；`streaming.rs` 单测 |
| 演示数据 | `crates/adapters/src/llm/demo.rs` | `usecases/src/tests/demo.rs` |
| 设置 / 凭据 | `src-tauri/src/settings/`、`credentials.rs` | `settings/tests.rs`、`credentials.rs` |
| 历史存储 | `src-tauri/src/sessions/` | `sessions/tests.rs` |
| 新增 IPC 链路 | `src-tauri/src/commands.rs`、`lib.rs` | `ui/src/api/ipc.ts`、`ui/src/types.ts` |
| 几何决策台 | `ui/src/components/MagiConsole.tsx`、`MagiUnit.tsx`、`MagiResponse.tsx` | `ui/src/styles.css` |
| 单元详情弹窗 | `ui/src/components/MagiDetailModal.tsx`、`UnitDetail.tsx` | `TerminalModal.tsx` |
| Extended Analysis 抽屉 | `ui/src/components/ExtendedAnalysis.tsx`、`UnitColumn.tsx`、`VerdictPanel.tsx` | — |
| 协议状态机 | `ui/src/hooks/useMagiSession.ts` | 组件只消费它的 state/actions |
| 设置 / 历史弹窗 | `SettingsPanel.tsx`、`SessionHistory.tsx`、`useSessionHistory.ts` | — |
| 样式 / 布局 | `ui/src/styles.css` + Tailwind utility classes | 保持三角形网格、clip-path、响应牌比例 |

## 必须守住的纪律

1. **分层边界。** `core` 不出现 HTTP / 文件 / GUI / IPC 类型；`usecases` 不出现 `tauri::`；command 不写业务逻辑，错误统一转 `String`，不返回空结果假装成功。
2. **Rust 是真相来源。** 改 `magi_core` 字段必须同步 `ui/src/types.ts`；wire 字段是 snake_case，顶层 Tauri 参数是 camelCase（`onEvent`）。
3. **LLM 输出必须结构校验，禁止静默降级。** 失败返回 `Error::InvalidLlmOutput`；prompt 改动 bump 对应 `PROMPT_VERSION`。
4. **三路并发、失败不丢弃。** 用 `tokio::join!`；单脑失败保留为 `Failed` 槽位，裁决变 `Incomplete`，不伪造意见。
5. **终审密封。** 任何 prompt 都不得包含其他脑的终审票；同轮辩论回复也不得互相可见。
6. **裁决与决策策略只在 Rust。** 前端只展示 `opening.decision` / `vote.verdict`，不得在 TS 里重新实现多数票或状态优先级。
7. **会话信任边界。** 落库前重新校验 batch 顺序/长度并重算所有派生字段；前端回传的 verdict 一律不可信。
8. **凭据只写不读。** key 不写日志、不进错误串、不进 prompt、不进会话文件；`credentials.json` 0600；`LlmConfig` 的 `Debug` 必须脱敏。
9. **流式防串台。** 用 `run_id` 过滤事件；按 persona 追加；reset/新会话后旧回调不得再改 state。
10. **演示模式编译门禁。** 只在 `debug_assertions` 下编译/注册；必须常显 MOCK 横幅；会话 `source: mock`；release 绝不伪装真实输出。
11. **几何视觉不要退化成普通卡片。** 顶部决策台的三角网格、clip-path 单元、连线、响应牌比例是产品核心；改布局前先确认设计意图。样式优先 Tailwind v4 token，纯几何规则留在 `styles.css`；尊重 `prefers-reduced-motion`。
12. **不要自动发起 LLM 调用。** 只在用户动作（TRANSMIT / CROSS-EXAMINATION / FINAL VOTE / RETRY）时启动；`useEffect` 里不启动，避免 StrictMode 双调用。
13. **预算不可绕过。** 问题超限报错；上下文按头尾截断；同伴输出截断后再入 prompt。
14. **历史文件安全。** 原子写（temp + fsync + rename）、0600、上限 100、损坏备份、未来 schema 只读。
15. **Tauri 细节。** 新 command 必须注册到 `lib.rs` 的 `handlers!`（debug/release 两份列表按需同步）；新增插件才需要 capabilities；当前只启用 `core:default`。

## 前端结构

- `App.tsx`：加载 settings/runtime flags、历史列表；管理输入草稿、设置/历史/单元详情弹窗；调用 `useMagiSession`。
- `useMagiSession`：状态机 `idle → opening → opening_ready → debating → debate_ready → voting → verdict`；维护 `streaming`、`nodes`、`nodeErrors`、`stageError`、`failedStage`、`saveWarning`；每个阶段完成后自动保存草稿；用 `runIdRef` 防陈旧流。
- `MagiConsole`：几何面板 + 阶段进度 + `CROSS-EXAMINATION` / `FINAL VOTE` / `NEW SESSION`。
- `MagiUnit`：可点击的梯形单元，填充色 = 当前 stance；thinking 时 flicker。
- `MagiResponse`：`合意 / 拒絶 / 状態 / 情報 / 誤差`；`PROVISIONAL` 或 `FINAL`。
- `MagiDetailModal` + `UnitDetail`：点击单元查看开场、质询、终审、错误。
- `ExtendedAnalysis`：底部抽屉，放原来的三列详情 + `VerdictPanel`。
- `SettingsPanel`：多 provider、只写 key、模型探测、连接测试。
- `api/ipc.ts`：唯一的 `invoke` / `Channel` 封装；组件不得直接 `invoke`。

当前没有路由库、测试框架或全局状态库；页面通过 `useState` 弹窗切换。

## 新增一条 IPC 链路

1. 在 `crates/usecases` 实现用例并写单测（纯逻辑，不碰 Tauri）。
2. 在 `src-tauri/src/commands.rs` 加薄壳，并在 `src-tauri/src/lib.rs` 的 `handlers!` 注册（debug/release 两份）。
3. 在 `ui/src/types.ts` 同步类型（snake_case），再在 `ui/src/api/ipc.ts` 加带类型的包装函数。
4. 组件只调用 `ipc.ts`，不直接 `invoke`。
5. 跑验证命令；涉及 UI 时用 `pnpm app:dev` 手动走一遍，必要时用 Playwright + 假 Tauri IPC 做流程测试。

## 测试地图

- `cargo test -p magi-core`：决策优先级、裁决表、序列化、会话校验/重算。
- `cargo test -p magi-adapters`：prompt 内容、JSON 严格解析、SSE 分帧、UTF-8 跨 chunk、body shaping、demo 数据。
- `cargo test -p magi-usecases`：三路并发屏障、单脑失败、辩论全开场要求、投票隔离、立场变化、预算、demo 全流程。
- `cargo test -p magi-app`：设置/凭据（0600、只写、迁移）、历史（CRUD、上限、损坏备份、未来 schema）。
- 前端没有单测框架；用 `pnpm typecheck` + `pnpm build` + Playwright 流程验证。

## 当前状态

- 已实现：几何决策台、PROVISIONAL/FINAL 决策、三阶段协议、单元详情弹窗、Extended Analysis 抽屉、多 provider 设置、写读隔离的 key、本地历史、dev-only 演示模式、release 打包。
- 未实现：问题 yes/no 分类调用、每脑独立模型、自动重试、真实 provider 的 CI 端到端测试、PDF/文档导入、云同步、移动端、语音。
- 已知限制：DMG 未签名/未公证；本地 release 可运行，分发前需要签名；dev 端口 5174。
- EVA 相关名称只做同人致敬，不使用官方 logo/素材。

## 文档与技能索引

- `README.md`：快速开始、界面、存储、命令。
- `docs/protocol.md`：阶段不变量、决策表、裁决表、失败语义、预算。
- `docs/console.png`、`docs/extended.png`：界面参考截图。
- 参考实现：<https://github.com/TomaszRewak/MAGI>（MIT，几何布局灵感来源）。

## 提交规则和代码规范

1. 所有代码注释用英语。
2. 都在 `main` 分支提交。
3. commit message 为一行英文。
