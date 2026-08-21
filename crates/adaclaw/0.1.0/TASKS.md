# AdaClaw — 分阶段实施计划

> 每个 Task 开始时，把本文件和 ARCHITECTURE.md 贴给 AI，保持设计一致性。

---

## Phase 0：项目骨架（Task 1）

**目标**：`cargo build` 通过，`adaclaw --help` 能运行，所有 trait 定义完毕

### 交付物

- [x] Cargo workspace 初始化（根 `Cargo.toml` + `crates/adaclaw-core`）
- [x] `.cargo/config.toml`（release 优化：opt-level="z", lto="fat", strip）
- [x] `crates/adaclaw-core/src/` 全部 trait 定义：
  - `provider.rs`：`Provider` trait + `ChatMessage` + `ChatRequest` + `ChatResponse` + `ProviderCapabilities`
  - `channel.rs`：`Channel` trait + `InboundMessage` + `OutboundMessage` + `MessageContent`
  - `memory.rs`：`Memory` trait + `MemoryEntry` + `Category`
  - `tool.rs`：`Tool` trait + `ToolSpec` + `ToolResult`
  - `observer.rs`：`Observer` trait + `ObserverEvent`
  - `sandbox.rs`：`Sandbox` trait
  - `tunnel.rs`：`Tunnel` trait
- [x] `src/config/schema.rs`：完整 `Config` 结构体（serde + toml）
- [x] `src/main.rs`：CLI 骨架（clap derive，子命令：run / chat / config / stop / status / doctor / onboard）
- [x] `tracing` 日志初始化
- [x] CI 检查：`cargo clippy --deny warnings` + `cargo test`

### 关键设计决策

- `adaclaw-core` 只依赖 `async-trait` + `serde` + `anyhow`，保持最小依赖
- 所有 trait 方法用 `async_trait` 宏
- `Config` 支持环境变量覆盖（`ADACLAW_API_KEY` 等）

---

## Phase 1：核心 Agent Loop（Task 2）

**目标**：CLI 里能和 LLM 对话，能调用 shell / file 工具

### 交付物

- [x] `crates/adaclaw-providers/`：
  - `ProviderSpec` 数据驱动注册表（`static PROVIDER_REGISTRY: &[ProviderSpec]`）
  - `create_provider()` 工厂函数
  - `ReliabilityChain`（故障切换，参考 picoclaw FallbackChain）
  - 实现：`openai.rs`（OpenAI + 所有 compatible 端点）、`anthropic.rs`、`ollama.rs`
- [x] `crates/adaclaw-memory/`：
  - `SqliteMemory`（rusqlite + FTS5，暂不做向量）
  - `NoneMemory`
  - `create_memory()` 工厂
- [x] `crates/adaclaw-tools/`：
  - `shell`（workspace 路径隔离）
  - `file_read` / `file_write` / `file_list`
  - `memory_store` / `memory_recall` / `memory_forget`
  - `http_request`
- [x] `src/agents/engine.rs`：Tool Call Loop
  - 工具调用多格式解析器（`src/agents/parser.rs`）：
    - OpenAI native JSON `tool_calls`
    - XML `<tool_call>` 标签系列
    - Markdown ` ```tool_call ``` ` fence
    - GLM shortened `tool_name>value` 格式
  - 并行工具执行（`futures_util::join_all`，无审批时）
  - 工具去重（签名哈希 `HashSet`）
  - 历史硬裁剪（`trim_history`）
  - 凭证脱敏（`src/security/scrub.rs`，最早实现的安全功能）
- [x] `src/agents/compact.rs`：LLM 摘要压缩（`auto_compact_history`）
- [x] CLI Channel（`src/channels/cli.rs`）：本地交互式 REPL
- [x] 安全 P0：workspace 路径隔离（符号链接检测 + 系统目录黑名单）

### 工具调用解析器安全原则

```
SECURITY: 绝对不从裸 JSON 中提取工具调用（防 prompt injection）
工具调用必须在明确的边界标记内（XML/Markdown/GLM格式）
```

---

## Phase 2：生产基础（Task 3）

**目标**：能通过 Telegram 使用，可长期后台运行

### 交付物

- [x] `src/bus/`：Message Bus
  - `InboundMessage` / `OutboundMessage` 完整类型
  - `MessageBus`（`tokio::sync::mpsc` 输入 + `broadcast` 输出）
  - `AgentRouter` + `RoutingRule`（channel_pattern / sender_id / default）
- [x] `src/agents/registry.rs`：`AgentRegistry`（多 Agent 配置）
- [x] `crates/adaclaw-channels/src/manager.rs`：`ChannelManager`（并发管理）
- [x] `crates/adaclaw-channels/src/telegram.rs`：Telegram Bot API
  - 长轮询 + Webhook 两种模式
  - HMAC-SHA256 签名验证
  - 支持文本 / 图片 / 语音 / 文件消息
- [x] `crates/adaclaw-server/`：Gateway（axum）
  - `POST /v1/chat`
  - `GET  /v1/status`
  - `POST /v1/stop`（Estop 入口）
  - `GET  /pair`（配对码，6位一次性）
  - Bearer Token 中间件
- [x] `src/daemon/`：守护进程模式（`adaclaw daemon start/stop/restart`）
- [x] 安全 P1：
  - Channel 白名单（`allowlist: deny-by-default`）
  - Gateway Pairing（配对码）
  - 密钥加密存储（`ChaCha20-Poly1305`，`crates/adaclaw-security/src/secrets.rs`）
- [x] `src/cron/`：基础定时任务
- [x] Webhook HMAC 验证（Telegram）
- [x] `docker-compose.yml` 模板（仓库根目录）：
  - `restart: unless-stopped`，仅挂载 `./workspace` 和 `config.toml:ro`
  - `read_only: true`，`tmpfs: /tmp`，`cap_drop: ALL`，`no-new-privileges: true`
  - Gateway 端口仅绑定 `127.0.0.1:8080`

---

## Phase 3：高级记忆系统（Task 4）

**目标**：记忆检索质量达到生产级，支持语义搜索

### 交付物

- [x] `sqlite-vec` 集成（`crates/adaclaw-memory/src/sqlite.rs` 升级，feature-gated）
- [x] `crates/adaclaw-memory/src/embeddings/`：
  - `EmbeddingProvider` trait
  - `FastEmbedProvider`（本地，AllMiniLML6V2，384维，零 API 依赖，spawn_blocking 异步化）
  - `OpenAIEmbedProvider`（text-embedding-3-small）
  - `NoopEmbedProvider`（降级为纯 FTS5）
- [x] `crates/adaclaw-memory/src/rrf.rs`：Reciprocal Rank Fusion（k=60，独立模块，完整单元测试）
- [x] `SqliteMemory` 升级：
  - 向量搜索（sqlite-vec BLOB，feature = "sqlite-vec"）
  - FTS5 关键词搜索（BM25）
  - RRF 混合融合（三路策略：hybrid → FTS5 → LIKE fallback）
  - 嵌入失败自动降级，不中断主流程
- [x] `memory_hygiene`：过期记忆自动清理（TTL per category）
- [x] `MarkdownMemory`（文件式存储后端，YAML front-matter + Markdown body）
- [x] `MemoryConfig` 升级：`embedding_provider` / `vector_weight` / `keyword_weight` / `ttl_days`
- [x] `create_memory_with_config()` 工厂：支持完整 embedding 参数传递
- [x] `run.rs` 升级：daemon 启动时读取 embedding 配置并初始化
- [x] 24 个单元测试全部通过（rrf / embeddings / sqlite / markdown）

### Phase 3 补充改进（Task 4b）

- [x] **OpenRouter provider**（`crates/adaclaw-providers/src/openrouter.rs`）：单 API key 访问数百模型，支持 `HTTP-Referer` / `X-Title` attribution headers
- [x] **DeepSeek provider**（`crates/adaclaw-providers/src/deepseek.rs`）：OpenAI-compatible，支持 `deepseek-chat` / `deepseek-reasoner`
- [x] **Provider 注册表扩充**：PROVIDER_REGISTRY 新增 OpenRouter + DeepSeek
- [x] **历史会话索引**（`src/agents/engine.rs`）：每轮对话结束时自动将用户输入 + Agent 回复存入 `Category::Conversation`，key 格式 `conv:{session_id}:{ts}`；`AgentEngine` 新增 `.with_memory()` builder
- [x] **Congee 滚动摘要**（`src/agents/compact.rs` 重写）：`rolling_compact()` 替代单次大批量压缩，保留最新 N 轮完整历史 + LLM 生成滚动摘要，失败自动降级为 hard-trim
- [x] **QMD 查询分解**（`crates/adaclaw-memory/src/query.rs`）：`recall_with_qmd()` 把复杂查询 LLM 拆解成 2-5 个子查询，并行检索，N 路 RRF 合并；LLM 失败自动降级为单次 recall
- [x] **Category::Global**（`crates/adaclaw-core/src/memory.rs`）：新增全局共享参考记忆分类；sqlite.rs / markdown.rs 同步更新 category 序列化
- [x] **GlobalMemory wrapper**（`crates/adaclaw-memory/src/global.rs`）：`recall()` 自动在私有结果前置全局知识；提供 `store_global()` / `list_global()` 辅助方法
- [x] **记忆刷写整理**（`crates/adaclaw-memory/src/consolidation.rs`）：`consolidate()` 两阶段 LLM 去重合并（聚类 + merge），批量处理，失败不破坏原始数据，可接入 cron 调度

---

## Phase 4：多 Agent 系统（Task 5）

**目标**：按渠道/发送者路由不同 Agent，支持异步 Agent 委托

> **设计参考**：对标 picoclaw（Go）的 AgentInstance + FallbackChain + spawn tool 模式；
> sub-agent 结果回传参考 nanobot（Python）的 bus.publish_inbound 模式。

### 交付物

#### 1. `AgentInstance` + `AgentRegistry` 完整实现

- [x] `src/agents/instance.rs`：`AgentInstance` 结构体完整重构
  - `Arc<dyn Provider>`（跨任务共享），`allow_delegate`，`build_tools()` 方法
  - Per-agent workspace 目录（支持 `~` 展开，缺省：`~/.adaclaw/workspace-{agent_id}`）
  - `allowed_tools` 白名单在实例化时过滤，`build_tools()` 运行时重建（规避 Box<dyn Tool> 无法 Clone）

- [x] `src/agents/registry.rs`：从存 `AgentConfig` 改为存 `AgentInstance`
  - 启动时根据配置批量创建所有 `AgentInstance`
  - `get(id) -> Option<&AgentInstance>`
  - `list_agents() -> Vec<&str>`
  - `can_delegate(parent_id, target_id) -> bool`（允许名单检查）
  - `get_default() -> Option<&AgentInstance>`（兜底 Agent）

#### 2. `RoutingRule` 完整实现（`AgentRouter`）

- [x] `src/bus/router.rs`：明确 3 级优先级文档注释 + `route_or()` 辅助方法
  - Priority 1: `channel_pattern`（Glob → Regex，如 `telegram:*`）
  - Priority 2: `sender_id`（精确匹配）/ `sender_name`（Glob）
  - Priority 3: `default = true`（兜底）
- [x] `src/daemon/run.rs`：特殊处理 `channel = "system"` 的 InboundMessage（sub-agent 回传通道）

#### 3. `ReliabilityChain` 完整版（`crates/adaclaw-providers/src/reliable.rs`）

- [x] 指数退避重试（最大 3 次，初始 1 s，2× 增长，上限 30 s）
- [x] 熔断器（连续失败 `circuit_threshold` 次后进入冷却，冷却期后自动恢复）
- [x] `CooldownTracker`（`HashMap<provider_name, Instant>`，速率限制 429 快速跳转）

#### 4. `DelegateTool`（异步 Agent 间任务委托）

- [x] `src/agents/delegate.rs`：`DelegateTool`
  - **异步设计**：`tokio::spawn` 后台执行，立即返回"已接受"，不阻塞主 Agent
  - 完成后通过 `MessageBus.send_inbound_bypass()` 发布 `channel = "system"` 的 `InboundMessage`
  - 允许名单检查：调用 `AgentRegistry::can_delegate(parent, target)` — 失败直接返回错误
  - **防递归**：sub-agent 的工具列表中不注入 `delegate` 工具（由 run.rs 保证）
  - Sub-agent 独立运行在 `tokio::spawn` 内，有独立超时（默认 300 s）
- [x] `src/bus/queue.rs`：新增 `send_inbound_bypass()`（绕过白名单，专供 system 消息）

#### 5. 配置示例更新（`config.toml`）

```toml
[agents.assistant]
model = "openrouter/auto"
# 不限制工具，默认允许所有

[agents.coder]
model = "anthropic/claude-sonnet-4"
temperature = 0.2
tools = ["shell", "file_read", "file_write"]
workspace = "~/.adaclaw/workspace-coder"
# coder 不允许委托其他 Agent（防递归）
[agents.coder.subagents]
allow = []

[agents.assistant.subagents]
allow = ["coder"]   # assistant 可以把编码任务委托给 coder

[[routing]]
channel_pattern = "telegram:@dev_bot"
agent = "coder"

[[routing]]
default = true
agent = "assistant"
```

#### ~~`model_routing_config` 工具~~（推迟，移至后续规划）

> 对标五个项目均无运行时路由配置修改功能；实现复杂（需 `Arc<RwLock<Config>>` + 持久化），
> 用户场景不明确，暂不实现。

### 关键设计决策

1. **delegate 异步设计**：参考 nanobot `SubagentManager.spawn()` + picoclaw `processSystemMessage`
   - 主 Agent 不卡住等待子 Agent，体验更流畅
   - sub-agent 结果通过 `channel = "system"` 的 InboundMessage 回注 Bus
2. **AgentInstance 工作隔离**：参考 picoclaw `AgentInstance`，每个 Agent 有独立 workspace + session
3. **allowlist 强制校验**：参考 picoclaw `CanSpawnSubagent`，防止 Agent 越权委托
4. **防递归**：sub-agent 的工具列表中不注册 `delegate`（参考 nanobot 设计）

---

## Phase 5：安全强化（Task 6）

**目标**：生产级安全，7层纵深防御全部到位

### 交付物

- [x] `crates/adaclaw-security/src/estop.rs`：紧急停止
  - 4级：`KillAll` / `NetworkKill` / `DomainBlock` / `ToolFreeze`
  - 状态持久化（重启后 Estop 状态保留）
  - broadcast 通知订阅者；GlobalEstop 单例
- [x] `crates/adaclaw-security/src/otp.rs`：TOTP（RFC 6238）
  - 手动实现 HMAC-SHA1 + 动态截断，无外部 TOTP crate 依赖
  - Base32 编解码内置；RFC 4226 Appendix D 向量测试
  - 可选：恢复 Estop 时需要 OTP 验证
- [x] `crates/adaclaw-security/src/approval.rs`：`ApprovalManager`
  - `AutonomyLevel`：`ReadOnly` / `Supervised` / `Full`
  - CLI 渠道：工具执行前交互式确认（有框提示 y/N）
  - 其他渠道：`Supervised` 模式自动 deny（不中断）
- [x] `crates/adaclaw-security/src/ratelimit.rs`：
  - `per_user`：每用户每分钟消息数（滑动窗口）
  - `per_channel`：每渠道每分钟消息数
  - `daily_cost_budget`：每日 LLM 费用上限（美元）
  - `max_actions_per_hour`：每小时工具调用上限
- [x] `crates/adaclaw-security/src/audit.rs`：结构化审计日志
  - `AuditKind` 枚举（ToolExecuted / FileAccessed / UnauthorizedAccess / EstopEngaged / RateLimitExceeded / MessageReceived / AgentStarted / AgentError / DaemonStarted / DaemonStopped / ...）
  - 写入 JSONL 文件（可接 SIEM）；`read_all()` 返回已记录事件
- [x] `crates/adaclaw-security/src/sandbox/landlock.rs`：Linux Landlock LSM
  - `#[cfg(target_os = "linux")]`，非 Linux 优雅降级为 no-op
  - 限制进程可访问的文件路径；graceful degradation（内核不支持时不报错）
- [x] `crates/adaclaw-security/src/scrub.rs` 完整版：
  - 三路正则：Bearer token → URL 凭证 → KV 键值对
  - 覆盖：token / api_key / api_secret / password / passwd / passphrase / secret / credential / private_key / access_key / client_secret / auth_token / x_api_key / webhook_secret / signing_secret / session_token / refresh_token / encryption_key / database_password / db_pass / smtp_password
  - URL 嵌入凭证（`https://user:password@host`）脱敏
  - 保留前4位便于调试；26 个单元测试全部通过
- [x] `crates/adaclaw-security/src/sandbox/docker.rs`：`ContainerEnvironment`
  - `is_running_in_container()`：Linux 检测 `/.dockerenv` + `/proc/1/cgroup` + 环境变量；macOS/Windows 检测环境变量
  - `check_autonomy_safety(level)`：`Full` 模式下若不在容器内返回 `SecurityWarning`
  - 启动时自动调用：非容器 + Full 模式 → stderr 打印红色警告
  - `config: security.allow_full_outside_container = true` 可跳过
- [x] `crates/adaclaw-security/src/sandbox/workspace.rs`：工作区路径隔离
  - 符号链接检测 + 系统目录黑名单（Unix/Windows）
- [x] `src/config/schema.rs` 更新：`SecurityConfig` 新增 `rate_limit` / `audit_log` / `estop_state_path` / `require_otp_for_estop`
- [x] `src/daemon/run.rs` 集成安全子系统：
  - 启动时容器环境检测 + Full 模式警告
  - Estop 控制器初始化（磁盘状态恢复）
  - AuditLogger 初始化（记录 daemon 启动/消息/Agent 错误）
  - RateLimiter 初始化（消息分发前检查）
  - Estop 检查（KillAll 时拒绝所有消息）
- [x] 85 个单元测试全部通过，`cargo build` 干净

---

## Phase 6：多渠道扩展（Task 7）✅

**目标**：覆盖国内主流渠道 + 国际主流渠道

### 交付物

#### 基础架构改造（Phase 6 新增）
- [x] `base.rs`：`BaseChannel` 辅助结构体（白名单、消息上报、运行状态）
  - `is_allowed()` 支持 `id|username` 复合格式（参考 picoclaw）
  - `handle_message()` 统一上报 + 白名单拦截
  - `is_running()` / `set_running()` AtomicBool
- [x] `Channel` trait 新增 `is_running()` 默认方法
- [x] `ChannelConfig` 新增 `allow_from` / `allow_from_groups` / `require_mention` / `send_progress`
- [x] `ChannelManager` 重构：
  - Outbound Dispatch Loop（从 `broadcast::Receiver<OutboundMessage>` 消费）
  - `Arc<RwLock<HashMap>>` 支持运行时热插拔（`register_channel` / `unregister_channel`）
  - `start_all(bus, outbound_rx)` 新签名
- [x] `run.rs` 更新：`outbound_rx` 传给 `channel_manager.start_all()`

#### 国内渠道（优先）
- [x] `dingtalk.rs`：钉钉群机器人 + Outgoing Webhook
  - axum HTTP 内嵌服务器，HMAC-SHA256 签名验证
  - session_id = sessionWebhook URL，send() 直接 POST 回复
- [x] `feishu.rs`：飞书/Lark 机器人（事件订阅 Webhook）
  - URL 验证挑战（旧格式 + schema 2.0）
  - verification_token 校验
  - tenant_access_token 自动刷新（带缓存）
  - 通过 Feishu Open API 发送消息
- [x] `wechat_work.rs`：企业微信 AIBot 机器人 + 回调
  - SHA1 签名验证（4参数排序）
  - AES-256-CBC 解密（block_size=32 非标准 PKCS7，参考 picoclaw 关键细节）
  - 消息去重（HashMap，超 1000 条自动清空）
  - GET 验证 URL + POST 消息处理

#### 国际渠道
- [x] `discord.rs`：Discord Bot（Gateway WebSocket + Message Intent）
  - HELLO/IDENTIFY/HEARTBEAT/MESSAGE_CREATE 完整协议
  - mpsc 分离写端，支持心跳 + 主循环并发
  - 指数退避自动重连
  - REST API 发送消息，rate-limit 重试
- [x] `slack.rs`：Slack App（Events API Webhook）
  - HMAC-SHA256 签名 + 防重放（5分钟时间窗口）
  - URL 验证挑战
  - chat.postMessage 发送
- [x] `webhook.rs`：通用 HTTP Webhook（适合自定义集成）
  - HMAC-SHA256 签名验证（可选）
  - 标准 JSON 入站格式 `{sender_id, sender_name, content, session_id}`
  - 可选 outbound_url 回调
- [ ] `email.rs`：Email（IMAP 收信 + SMTP 发信，推迟）
- [ ] `matrix.rs`：Matrix（E2EE 可选，feature-gated，推迟）
- [ ] `irc.rs`：IRC（简单实现，推迟）

#### 技术亮点（借鉴对标项目）
- **Thinking... 占位消息**：Telegram 收到消息立即发占位符，AI 完成后 editMessageText（来自 picoclaw）
- **Markdown → Telegram HTML**：完整转换器（粗体/斜体/代码/链接/删除线/列表，来自 nanobot + picoclaw）
- **id|username 复合白名单**：同时匹配 Telegram user_id 和 username（来自 picoclaw）
- **双层白名单**：`allow_from`（私聊）+ `allow_from_groups`（群聊）分开控制（来自 openclaw）
- **企业微信 AES 坑**：block_size=32 非标准 PKCS7，完整实现（来自 picoclaw）
- **DingTalk 消息路由**：session_id = sessionWebhook URL，Agent 回复直接 POST（来自 nanobot）
- **Discord 心跳分离**：mpsc channel 分离写端，心跳任务和读取循环并发（来自 nanobot discord.py）

### 渠道通用要求
- [x] 每个渠道实现 `Channel` trait（含 `is_running()`）
- [x] HMAC 签名验证（各自的签名格式：Telegram HMAC-SHA256 / 钉钉 HMAC-SHA256 / Slack HMAC-SHA256 / 企业微信 SHA1）
- [x] 用户白名单（per-channel `allow_from` + BaseChannel 统一过滤）
- [x] 连接断线自动重连（Telegram 指数退避 / Discord 指数退避）
- [x] 支持文本 / 图片（文件ID）/ 语音 / 文件消息标注

---

## Phase 7：可观察性 + 技能系统（Task 8）✅

**目标**：生产运维完备，支持技能扩展

### 交付物

- [x] `src/observability/prometheus.rs`：Prometheus 指标（依赖-free，纯 atomic 实现）
  - `adaclaw_agent_turns_total{agent_id, provider, model}`
  - `adaclaw_tool_calls_total{tool, success}`
  - `adaclaw_llm_requests_total{provider, model, success}`
  - `adaclaw_llm_input_tokens_total` / `adaclaw_llm_output_tokens_total`
  - `adaclaw_channel_messages_total{channel, direction}`
  - `adaclaw_errors_total{component}`
  - `adaclaw_heartbeat_ticks_total`
  - `/metrics` 端点（`adaclaw-server`）
- [x] `src/observability/noop.rs`：零开销 noop 观察者
- [x] `src/observability/log.rs`：基于 `tracing` 的日志观察者
- [x] `src/observability/trace.rs`：`RuntimeTracer`（结构化运行时事件，JSONL，滚动模式）
- [x] `src/observability/mod.rs`：工厂函数 `create_observer()` + 全局 observer 单例
- [x] `src/skills/`：技能系统
  - `SKILL.md` / `SKILL.toml` 工作目录加载
  - 符号链接拒绝 + 目录名安全校验（防 prompt injection）
  - `skills_to_prompt()` XML 安全注入系统提示（32KB 上限）
- [x] `src/identity/`：Agent 身份
  - `IDENTITY.md` 加载（自动解析 `**Name:**` 字段）
  - 符号链接拒绝，未找到时使用内置默认
  - `to_prompt_section()` 注入系统提示
- [x] `src/cli/doctor.rs`：`adaclaw doctor` 诊断（完整版）
  - Provider / API Key 检查
  - 内存系统健康检查（SQLite / Markdown / 嵌入提供商）
  - 渠道配置验证（token 完整性）
  - 可观察性后端状态
  - 技能目录检查
  - 隧道配置检查
  - 二进制大小检查（目标 <10 MB）
  - 容器环境检测：`Full` 模式下若不在容器内打印警告
- [x] `src/cli/onboard.rs`：`adaclaw onboard` 引导向导
  - 交互式首次配置（无外部依赖，纯 stdin/stdout）
  - 自动生成 `config.toml`（含 provider、agent、memory、security、observability）
  - 创建 workspace / skills / IDENTITY.md 骨架
  - `Full` 模式时展示容器安全警告
- [x] `src/tunnel/`：隧道集成（进程外启动）
  - `cloudflare.rs`：Cloudflare Tunnel（cloudflared CLI）
  - `tailscale.rs`：Tailscale Funnel / Serve
  - `ngrok.rs`：ngrok（支持 auth token 临时配置）
  - `mod.rs`：统一工厂 `start_tunnel()` + `TunnelHandle`（Drop 自动停止）
- [x] `src/config/schema.rs` 新增：`ObservabilityConfig` / `TunnelConfig`
- [x] `src/daemon/run.rs`：集成可观察性 + 隧道到守护进程启动流程
  - 步骤 3：Observer 初始化 + Prometheus `/metrics` 编码器注册
  - 步骤 10：隧道启动（配置驱动）
  - Agent 调度循环：每轮记录 `AgentTurn` / `AgentTurnEnd` / `ChannelMessage` / `Error` 事件
- [x] `crates/adaclaw-server/src/routes/metrics.rs`：`GET /metrics` 端点

---

## Phase 8：开源版本完善与发布（Task 9）✅

**目标**：让项目能被外部用户顺畅安装和使用，补齐发布就绪度。
*注意：Web UI 部分将作为单独的闭源/企业版本（AdaClaw Dashboard）在独立仓库中进行，不在本开源仓库范围内。*

### 8.1 代码库清理与优化

- [x] 运行 `cargo clippy --deny warnings`，修复所有 lint 警告（零警告零错误）
- [x] 统一日志输出格式，去掉开发期残留的 `dbg!` 和 `println!`
- [x] 确认 `cargo clippy --all-targets -- -D warnings` 全部通过
- [ ] 补充核心逻辑集成测试：多渠道路由、Memory RRF、Provider 熔断 ReliabilityChain 边缘场景（后续）

### 8.2 文档完善

- [x] `README.md`：全新英文版（30秒快速入门 + 功能说明 + 对比表 + 安装指南）
- [x] `README.md`：补充与竞品的对比表（安全体系/记忆系统/国内渠道/性能）
- [x] `README.md`：「30秒快速入门」（单命令安装 → onboard → adaclaw chat）
- [x] 完善 `config.example.toml`（覆盖所有配置项，每项附注释说明，与实际 schema 对齐）
- [x] 更新 `CONTRIBUTING.md`，添加 GitHub Issue / PR 模板（`.github/` 目录）

### 8.3 CI/CD（GitHub Actions）

- [x] `.github/workflows/ci.yml`：clippy(-D warnings) + test + 二进制大小检查 + format
- [x] `.github/workflows/release.yml`：tag 触发跨平台编译（Linux/macOS/Windows x86_64+aarch64）+ Release Assets
- [x] `.github/workflows/security.yml`：cargo audit + cargo-deny（每周定时 + PR 触发）

### 8.4 安装体验

- [x] `scripts/install.sh`：Linux/macOS 一键安装脚本（架构检测 + SHA256 校验 + PATH 配置）
- [x] `scripts/install.ps1`：Windows PowerShell 安装脚本（自动 PATH 更新）
- [x] 首次运行自动引导：`adaclaw onboard` 向导（已在 Phase 7 实现）
- [x] Homebrew formula（`Formula/adaclaw.rb`）

---

## Phase 9：渠道扩展（Task 10）✅

**目标**：补齐国际渠道覆盖缺口，追平主要竞品渠道广度。
*注意：QQ 渠道不在开源版本范围内。*

### 9.1 WhatsApp（高优先，国际用户核心需求）

- [x] `crates/adaclaw-channels/src/whatsapp.rs`
  - 实现方案：WhatsApp Business Cloud API（官方 Meta Webhook，对标 zeroclaw）
  - `GET /whatsapp`：Meta webhook 验证（hub.mode / hub.verify_token / hub.challenge）
  - `POST /whatsapp`：接收消息，`X-Hub-Signature-256` HMAC 验证（配置 `app_secret` 时必须）
  - 发送消息：`POST https://graph.facebook.com/v18.0/{phone_number_id}/messages`
  - 配置项：`access_token` / `phone_number_id` / `verify_token` / `app_secret`（可选）/ `allowed_numbers`
  - 需要隧道（HTTPS），常量时间比较防时序攻击
  - 支持文本/图片/语音/文件消息接收
- [x] `src/config/schema.rs`：添加 `WhatsAppConfig`
- [x] `crates/adaclaw-server/src/routes/whatsapp.rs`：`GET /whatsapp` + `POST /whatsapp` 路由（共享 HTTPS 端口模式）
- [x] `crates/adaclaw-server/src/server.rs`：`build_router()` + `start_server_with_whatsapp()` 支持可选 WhatsApp 路由
- [x] 文档：`docs/whatsapp-setup.md`（Meta 应用创建 → Webhook 配置 → 测试步骤）

### 9.2 Email（异步沟通场景）

- [x] `crates/adaclaw-channels/src/email.rs`
  - IMAP 收信（`imap` crate 同步实现，via spawn_blocking，TLS 支持，轮询间隔可配置）
  - SMTP 发信（`lettre` crate，STARTTLS + SMTP over TLS 两种模式）
  - 支持：Gmail App Password / 通用 IMAP/SMTP 服务器
  - `allow_from`：空 = 接受所有，非空 = 邮件发件人白名单
  - `auto_reply_enabled`：可选禁用自动回复（只读取分析）
  - `consent_granted`：安全门，必须显式设为 true 才启用（对标 nanobot）
  - MIME 解析：text/plain 优先，回退 text/html（简单标签剥离）
  - 配置项：`imap_host/port/username/password` + `smtp_host/port/username/password` + `from_address`
- [x] `src/config/schema.rs`：添加 `EmailConfig`
- [x] 文档：`docs/email-setup.md`（Gmail App Password 申请 → 配置示例 → 隐私说明）

### 9.3 Matrix（去中心化用户群体，feature-gated）

- [x] `crates/adaclaw-channels/src/matrix.rs`（`feature = "matrix"`）
  - Matrix Client-Server API：`/_matrix/client/v3/sync` 长轮询接收消息
  - 发送消息：`PUT /_matrix/client/v3/rooms/{roomId}/send/m.room.message/{txnId}`
  - 认证：`access_token` + `device_id`（稳定跨重启）
  - E2EE：预留 `e2ee_enabled` 配置项，当前版本未实现（未来可用 `vodozemac` 扩展）
  - 白名单支持 room_id 或 user_id 双重匹配
  - 配置项：`homeserver` / `user_id` / `access_token` / `device_id` / `allow_from` / `sync_timeout_ms`
  - 指数退避自动重连（sync 失败重试 10s）
- [x] `src/config/schema.rs`：添加 `MatrixConfig`
- [x] `crates/adaclaw-channels/Cargo.toml`：`matrix` feature 门控（`#[cfg(feature = "matrix")]`）
- [x] 文档：`docs/matrix-setup.md`

---

## Phase 10：生态对接（Task 11）

**目标**：接入外部工具生态，支持主动任务，让用户能扩展功能。

### 10.1 MCP 客户端完整实现

> ARCHITECTURE.md 已完成设计，本 Phase 实现代码。
> 配置格式与 Claude Desktop / nanobot 兼容，用户可直接复用现有 MCP 配置。

- [x] `crates/adaclaw-tools/src/mcp/mod.rs`：`McpClient` + `McpTool`（实现 `Tool` trait）
  - `McpTool::execute()` → 调用外部 MCP Server 的工具，透明包装为原生工具
  - 支持 `tool_timeout`（默认 30s，可配置）；`McpTool` 实现 `Clone`
- [x] `crates/adaclaw-tools/src/mcp/stdio.rs`：Stdio transport
  - 启动本地进程（npx / uvx / 可执行文件）
  - JSON-RPC over stdin/stdout（NDJSON）
  - 进程崩溃自动重启（最多3次）
- [x] `crates/adaclaw-tools/src/mcp/http.rs`：HTTP transport
  - 连接远程 MCP Server（HTTP POST JSON-RPC）
  - 支持自定义 `headers`（Authorization 等）
- [x] `crates/adaclaw-tools/src/mcp/loader.rs`：启动时自动发现并注册
  - 读取 `config.tools.mcp_servers`
  - 调用 `initialize` + `tools/list` 获取工具清单
  - `load_all_clonable()` 返回 `Vec<McpTool>`，注入 dispatch loop 工具列表
- [x] `src/config/schema.rs`：`McpServerConfig`（untagged enum：Stdio / Http）+ `ToolsConfig` + `HeartbeatConfig`
- [x] `src/daemon/run.rs`：daemon 启动时调用 `McpLoader::load_all_clonable()` + MCP 工具注入 dispatch loop
- [x] 单元测试：config 反序列化测试（McpServerConfig Stdio/Http）+ McpTool 输出解析测试
- [x] 文档：`docs/mcp.md`（配置示例 + 与 Claude Desktop 格式的兼容说明）

### 10.2 Heartbeat 系统（主动任务）

> 对标 nanobot / picoclaw / zeroclaw 均已实现，补齐此缺口。

- [x] `HEARTBEAT.md`：workspace 根目录文件格式定义（Markdown 任务列表格式 `- [ ] 任务描述`）
- [x] `src/cron/scheduler.rs`：扩展 Cron，实现 `HeartbeatScheduler`
  - 可配置间隔（默认 30 分钟，最小 5 分钟）
  - 读取 `HEARTBEAT.md`，提取 `- [ ] 任务描述` 行（跳过 `[x]` 已完成）
  - 将任务内容注入 Agent 上下文（通过 MessageBus），触发 Agent 执行
  - 4 个单元测试全部通过（parse_heartbeat_tasks / 空内容 / 无待办 / 不存在文件）
- [x] `src/config/schema.rs`：`HeartbeatConfig`（`enabled` / `interval_minutes` / `target_channel` / `heartbeat_file`）
- [x] `src/daemon/run.rs`：Heartbeat scheduler 在 daemon 启动时启动（独立 tokio task）
- [x] 文档：`docs/heartbeat.md`（HEARTBEAT.md 语法 + 配置 + 工作原理 + 示例）

### 10.3 技能市场对接（ClawHub）

> 对标 nanobot/picoclaw 已对接 ClawHub，补齐生态接入。

- [x] `adaclaw skill list`：列出已安装技能（来自 workspace/skills/）
- [x] `adaclaw skill install <url-or-name>`：从 URL 或 ClawHub 安装技能
  - 支持 HTTPS URL（直接下载 SKILL.md）
  - 支持 `clawhub:<name>` 短前缀（从 clawhub.ai 解析）
  - 支持本地路径安装
  - 安全审计：拒绝符号链接、检测 script 注入模式、检测 Trojan Source 双向控制字符
- [x] `adaclaw skill remove <name>`：删除已安装技能（含确认提示）
- [x] `adaclaw skill audit <name>`：手动运行安全检查（符号链接 + 注入模式检测）
- [x] `src/cli/skill.rs`：完整实现，`src/cli/mod.rs` 注册，`src/main.rs` 添加 Skill 子命令

### 10.4 语音转文字（Groq Whisper）

> 对标 nanobot/picoclaw：Telegram 语音消息自动转文字，极低配置成本。

- [x] `crates/adaclaw-providers/src/groq.rs`：Groq provider（OpenAI-compatible LLM + `GroqWhisper` 转录器）
  - LLM：`https://api.groq.com/openai/v1/chat/completions`
  - Whisper API：`POST https://api.groq.com/openai/v1/audio/transcriptions`
  - `GroqWhisper::transcribe(audio_bytes, filename, language)` 异步接口
- [x] `crates/adaclaw-providers/src/registry.rs`：PROVIDER_REGISTRY 新增 Groq（`groq` + aliases `groq-llama`, `llama-3`）
- [x] `crates/adaclaw-providers/Cargo.toml`：新增 `multipart` feature（Whisper multipart upload）
- [x] `src/config/schema.rs`：`providers.groq.api_key` 通过已有 `ProviderConfig` 框架支持
- [x] 文档：`docs/voice-transcription.md`（配置 + 工作原理 + 格式支持 + 隐私说明）

---

## Phase 11：开源版质量提升 — 对标竞品系列（分轮进行）

> 参考项目：openclaw（TypeScript）、picoclaw（Go）、nanobot（Python）、zeroclaw（Rust）
>
> 原则：不是照搬，而是借鉴经过市场和外界考验的设计细节，补足我们的实现缺陷。
> 每轮聚焦一个模块，方便管理和验证。

---

### 第 1 轮：渠道接入细节（已完成）✅

**对比范围**：`crates/adaclaw-channels/`
**参考项目**：picoclaw wecom.go、nanobot telegram.py/slack.py、zeroclaw telegram.rs/slack.rs/discord.rs

#### 已修复

- [x] **WeChat Work — PKCS7 padding 安全修复**
  - 原来：只检查最后一个字节的数值范围（`pad == 0 || pad > 32`）
  - 修复：逐字节验证所有 padding 字节均为相同值（标准 PKCS7 要求，参考 picoclaw `pkcs7UnpadWeCom`）
  - 文件：`crates/adaclaw-channels/src/wechat_work.rs`

- [x] **Telegram — 消息分片字符数计算错误**
  - 原来：`split_message(content, 4000)` —— 两个错误：上限 4000 非 4096，用字节数 `.len()` 而非 Unicode 字符数
  - 修复：常量 `TG_MAX_MESSAGE_CHARS = 4096`，`split_message` 改为 `.chars().count()` 计算，中文/emoji 正确处理
  - 文件：`crates/adaclaw-channels/src/telegram.rs`

- [x] **Telegram — Typing 指示器一次性发送（5 秒后过期）**
  - 原来：只在收到消息时 `sendChatAction` 一次，Agent 运行超 5 秒后用户看不到处理中状态
  - 修复：`start_typing_loop()` 持续循环（每 4 秒刷新），收到消息时启动，`send()` 发出回复时停止
  - 参考：picoclaw `thinkingCancel`、nanobot `_typing_loop`、zeroclaw `start_typing`

- [x] **Telegram — 群组 mention-only 模式缺失**
  - 原来：群组中响应所有消息，没有 @提及过滤
  - 修复：添加 `mention_only: bool` 字段，启动时 `getMe` 获取 bot username，群组消息检测 `@botname` 后才处理
  - 参考：picoclaw/nanobot/zeroclaw 均有 `mention_only` 配置

- [x] **Telegram — 无 Bot 命令支持**
  - 原来：没有 `/start`、`/help` 等命令处理
  - 修复：`process_update()` 前置命令路由，处理 `/start`（欢迎消息）和 `/help`（命令列表）
  - 参考：picoclaw telegram_commands.go、nanobot BOT_COMMANDS

- [x] **Telegram — 多实例 409 Conflict 无处理**
  - 原来：重启后若上一个实例还在 long polling，直接进入 30s 轮询会持续 409，消息无法收取
  - 修复：`startup_probe()` 用 `timeout=0` 探测，直到成功获得 slot 才进入正常轮询；409 时等待 35s
  - 参考：zeroclaw 的 startup probe 设计

- [x] **Discord — 完全没有 Typing 指示器**
  - 原来：`handle_message_create()` 和 `send()` 均无任何 typing 调用
  - 修复：添加 `start_typing_loop()` / `stop_typing_loop()`，收到消息启动（每 8 秒刷新 `POST /channels/{id}/typing`），发出回复停止
  - 参考：picoclaw/nanobot/zeroclaw 均有持续循环

- [x] **Slack — 回复 Markdown 无法渲染（格式错误）**
  - 原来：`post_message()` 直接发 Markdown 原文，但 Slack 用 mrkdwn 格式（`*bold*`、`~strike~`、`<url|text>`）
  - 修复：添加 `markdown_to_slack_mrkdwn()` 转换函数，覆盖：粗体/斜体/删除线/标题/链接/列表/代码块
  - 参考：nanobot `slackify_markdown`、zeroclaw slack 实现

- [x] **Slack — 不支持 Thread 回复**
  - 原来：无论是否在 thread 中，回复都发到频道根层级
  - 修复：事件解析时提取 `thread_ts`，session_id 编码为 `channel_id/thread_ts`；`send()` 解析后带 `thread_ts` 参数
  - 参考：picoclaw Slack Socket Mode thread routing

- [x] **Feishu — 非文本消息静默丢弃**
  - 原来：非 `text` 类型直接 `return OK`，用户发图片/语音/文件时毫无反馈
  - 修复：image/audio/file/video/sticker 生成占位文本，`post` 富文本消息提取标题；用户能感知消息被收到
  - 参考：nanobot feishu.py 的 MSG_TYPE_MAP 设计

---

### 第 2 轮：Provider 层稳定性（已完成）✅

**对比范围**：`crates/adaclaw-providers/`（`reliable.rs`、`router.rs`、错误分类）
**参考项目**：picoclaw `error_classifier.go`、`cooldown.go`、`fallback.go`、`factory_provider.go`

#### 已修复

- [x] **错误分类器**：新建 `crates/adaclaw-providers/src/error.rs`，实现 `ProviderErrorKind`（RateLimit / AuthError / BadRequest / ServerError / Unknown）+ `ProviderError` 结构体 + `classify_error()` 函数（支持 downcast + 字符串 fallback）
  - `AuthError`（401/403）和 `BadRequest`（400）：不重试，不触发熔断器
  - `RateLimit`（429）：不重试，计入熔断器，读取 `Retry-After` 头
  - `ServerError`（5xx）：退避重试，计入熔断器

- [x] **冷却策略细节**：`openai.rs` 和 `anthropic.rs` 在 HTTP 错误时先提取 `Retry-After` 头（`resp.headers().get("retry-after")`），再消费 response body，不再丢失该头信息；`ReliabilityChain` 从 `ProviderError` 中读取 `retry_after_secs` 并记录日志

- [x] **Fallback 顺序 + 无限循环防护**：`has_available_provider()` 前置检查；所有 Provider 均在冷却时返回清晰错误消息 `"All N provider(s) in circuit-breaker cooldown"`，而非误导性的 `"no providers configured"`

- [x] **Provider 测试覆盖**：新增 8 个 async 集成测试（Mock Provider）：
  - `test_chain_falls_back_to_second_provider`
  - `test_auth_error_skips_provider_without_circuit`
  - `test_bad_request_skips_provider_without_circuit`
  - `test_rate_limit_skips_provider_and_counts_failure`
  - `test_all_providers_fail_returns_error`
  - `test_all_providers_in_cooldown_returns_clear_error`
  - `error::tests` 12 个单元测试

- [x] **代码去重**：`ReliabilityChain::chat_with_system()` 委托给 `self.chat()`，消除两份重复的重试/熔断逻辑

- [x] **Tool Call 解析边界**：`parser.rs` GLM 格式验证增强（增加 `"` `'` `=` `/` `\\` 到排除集），防止 HTML 属性假阳性；新增 17 个边界测试（GLM 连字符/下划线/HTML属性/空args/XML多行/优先级/安全测试）

- [x] **预存在 bug 修复**：`scheduler.rs` 缺少 `.await`（`send_inbound(msg).await?`）；根 `Cargo.toml` 缺少 `reqwest` 依赖

- [x] **全部 135 个测试通过**（adaclaw: 91，adaclaw-providers: 47 — 新增 Billing/Timeout/Overloaded 模式 + 指数冷却公式测试）

---

### 第 3 轮：Agent Loop 边界情况（已完成）✅

**对比范围**：`src/agents/`（`engine.rs`、`compact.rs`、`parser.rs`）
**参考项目**：picoclaw `loop.go`、`loop_test.go`；zeroclaw `agent/` 模块

#### 已验证

- [x] **工具调用解析**：`parser.rs` 完整覆盖 Markdown fence / XML / GLM 格式，含 HTML 假阳性过滤、路径注入拒绝等 17 个边界测试（第 2 轮已完成）
- [x] **上下文超长处理**：`force_compress_messages`（系统提示保留 + 最近消息保留 + 无系统提示路径）共 8 个测试；`detect_context_window_error` 覆盖 OpenAI / Anthropic / Groq / GLM 等 9 种错误模式
- [x] **并行工具执行安全性**：
  - 新增 `test_join_all_collects_all_results_on_partial_failure`：验证 `join_all` 在某工具返回 `Err` 时不短路，其余工具结果仍全部收集
  - 新增 `test_join_all_all_failing_tools_still_collects_all`：所有工具失败时仍能收集所有错误
  - ⚠️ 已记录 panic 隔离局限性：`panic!` 会穿透 `join_all`，完全隔离需改用 `tokio::task::spawn`（工具实现者应返回 `Err` 而非 panic）
- [x] **工具输出大小限制**：
  - `crates/adaclaw-tools/src/shell.rs` 提取 `pub(crate) fn truncate_output(raw: &str) -> String` 辅助函数，`execute()` 改用统一调用
  - 新增 8 个单元测试：精确边界（exactly 10 000 chars）、大输入截断、CJK Unicode 按字符数计数（不按字节）、截断通知格式验证
- [x] **Compact 失败降级**：
  - 新增 `test_rolling_compact_calls_llm_and_inserts_summary`：成功路径验证摘要插入 index 1 + recent tail 保留
  - 新增 `test_rolling_compact_result_stays_below_rolling_threshold`：第二次调用是 no-op，不会无限循环调用 LLM
  - 新增 `test_rolling_compact_llm_failure_falls_back_to_hard_trim`：LLM 不可用时 `rolling_compact` 返回 `Ok`（不传播错误），hard-trim 生效
  - 新增 `test_auto_compact_history_*` 3 个端到端测试：正常路径、LLM 失败路径、安全网触发路径

#### 新增/修改文件

- `src/agents/compact.rs`：+9 个测试（mock PanicProvider / SummaryProvider / FailProvider 复用）
- `src/agents/engine.rs`：+7 个测试（`truncate()` 辅助函数 4 个 + `join_all` 行为 2 个 + dedup 逻辑 2 个）
- `crates/adaclaw-tools/src/shell.rs`：提取 `truncate_output()` + +11 个测试（截断逻辑 8 个 + `safe_path` / `normalize_path` 3 个）

**全部测试：74 agents + 16 adaclaw-tools，0 failed**

---

### 第 4 轮：Config 版本迁移（已完成）✅

**对比范围**：`src/config/`
**参考项目**：picoclaw `config/migration.go`、`config/migration_test.go`

#### 已完成

- [x] **配置版本升级路径**：新增 `src/config/migration.rs`
  - `config_version` 字段加入 `Config`（`#[serde(default)]`，缺失时自动为 0）
  - `migrate()` 函数：v0→v1 自动迁移，未来版本 bail! 提示用户升级
  - `Config::load_from_file` 集成迁移：TOML 语法错误添加文件路径、迁移通知通过 `tracing::warn!` 输出
  - `adaclaw config check [--file <path>]`：加载 + 迁移 + 校验三合一，错误逐条列出，exit(1)
  - `adaclaw config version`：显示当前 schema 版本
  - 6 个迁移单元测试全部通过（v0 迁移、幂等性、字段保留、未来版本拒绝等）

- [x] **必填字段缺失处理**：新增 `src/config/validation.rs`
  - `ValidationError` 含 TOML 风格字段路径（如 `agents.assistant.model`）+ 清晰错误信息
  - `validate()` 一次返回所有错误（不在第一个出错时停止）
  - 覆盖：agents（model 必填、temperature 范围、max_iterations≥1、subagents 引用未知 agent）、routing（agent 引用、无 default 路由警告）、security（autonomy_level 枚举）、channels（kind 枚举、Telegram/Discord/Slack/Feishu/WeCom/WhatsApp 必填字段）、memory（backend/embedding_provider 枚举、weight 范围）、observability（backend 枚举）
  - `Config::validate()` 便捷方法（委托给 `validation::validate()`）
  - 28 个语义验证单元测试全部通过

- [x] **默认值一致性修复**：`security.rate_limit.per_channel` 从 120 修正为 200（与 config.example.toml 一致）
  - 新增 `config.example.toml` 顶部 `config_version = 1` + 说明注释
  - `config.example.toml` 说明行更新（添加 `adaclaw config check` 快速验证提示）

---

### 第 5 轮：Security / Approval UX ✅

**对比范围**：`crates/adaclaw-security/src/approval.rs`
**参考项目**：zeroclaw `approval/`；openclaw Approval 流程

#### 已完成

- [x] **Telegram Inline Keyboard 审批**：
  - `crates/adaclaw-channels/src/telegram.rs`：`send_approval_prompt_msg()` 发送带 ✅ Approve / ❌ Deny 按钮的 Telegram 消息
  - callback prefix `"acadapr:yes:"` / `"acadapr:no:"` 解析按钮点击
  - `process_callback_query()` 将按钮点击转换为 `/approve-allow {id}` / `/approve-deny {id}` inbound 消息注入 Bus
  - `answer_callback_query_nonblocking()` 发送 toast 确认提示
  - `clear_inline_keyboard_nonblocking()` 点击后自动移除按钮
  - `Channel` trait 新增 `send_approval_prompt()` 可选方法（默认 no-op）+ `supports_approval_prompts()` 标志位
  - `TelegramChannel` 实现 `send_approval_prompt()` → 调用 `send_approval_prompt_msg()`
  - 10 个单元测试（callback 前缀解析 / HTML 转义 / 64 字节上限等）

- [x] **Approval 超时**：
  - `PendingApprovalRequest` 结构体：`expires_at` 字段（ISO 8601），默认 30 分钟后过期
  - `prune_expired()` 在每次访问 `pending_requests` 时自动清理过期请求
  - `has_pending_request()` / `list_pending_requests()` 均自动剔除过期项
  - `confirm_pending_request()` / `reject_pending_request()` 检测 `Expired` 错误
  - 超时时长通过 `security.approval_timeout_minutes`（默认 30）配置
  - 测试：`pending_request_expired_is_pruned`

- [x] **批量审批（"批准此类操作"）**：
  - **CLI "Always" 选项**：`prompt_interactive()` 新增 `[a]lways` 选项，添加到 `session_allowlist`，本次 session 内同名工具不再询问
  - **Non-CLI session allowlist**：`grant_non_cli_session(tool)` / `revoke_non_cli_session(tool)` — 用户点击 Approve 后，引擎可调用 `grant_non_cli_session()` 批准该工具在本 session 内自动执行
  - **一次性全通令牌**：`grant_non_cli_allow_all_once()` / `consume_non_cli_allow_all_once()` — 一次性授权当前 turn 内所有工具

- [x] **新增配置项**（`src/config/schema.rs`：`SecurityConfig`）：
  - `auto_approve = ["file_read", "memory_recall"]` — 永不需要审批的工具列表
  - `always_ask = ["shell", "file_write"]` — 永远需要审批（覆盖 session allowlist）
  - `approval_timeout_minutes = 30` — 待审批请求超时时间

- [x] **核心 `ApprovalManager` 升级**：
  - `auto_approve` / `always_ask` per-tool 列表（config + 运行时可变）
  - `approve_tool_supervised()` — 带 sender/channel/reply_target 上下文，非 CLI 时创建 pending request 并在 denial message 中嵌入 request_id
  - `needs_approval()` — 快速检查（不修改状态）
  - `apply_auto_approve()` / `apply_auto_approve_revoke()` — 运行时策略更新
  - 审计日志（`ApprovalLogEntry`：timestamp / tool / args / approved / channel）
  - `clear_pending_for_tool()` — 按工具名批量清除 pending requests

- [x] **全部 151 个测试通过**，`cargo build` 干净（无 warnings）

---

### 对比进度

| 轮次 | 模块 | 状态 |
|---|---|---|
| 第 1 轮 | 渠道接入细节 | ✅ 完成 |
| 第 2 轮 | Provider 稳定性（错误分类/冷却/Fallback） | ✅ 完成 |
| 第 3 轮 | Agent Loop 边界情况 | ✅ 完成 |
| 第 4 轮 | Config 版本迁移 | ✅ 完成 |
| 第 5 轮 | Security / Approval UX（Telegram Inline Keyboard） | ✅ 完成 |

---

## Phase 12：闭源版本规划（AdaClaw Dashboard）

> 以下功能面向企业用户和商业化场景，在独立的闭源仓库实现，不纳入本开源版本。

### 规划功能（不在本仓库实现）

- **Web UI（AdaClaw Dashboard）**：Chat 界面、记忆管理、配置编辑、审计日志可视化、Prometheus 指标图表
- **多租户**：per-user Agent 隔离，独立 workspace + session + 速率限制，用户管理 API
- **WASM 技能运行时**：技能沙盒执行，`adaclaw skill new <name>` 脚手架，ZeroMarket 对标
- **高级 Provider**：Gemini、Zhipu GLM、Qwen/DashScope、GitHub Copilot OAuth（按需在闭源版本添加）
- **PostgreSQL 记忆后端**：生产级分布式部署，多实例共享记忆（`crates/adaclaw-memory/src/postgres.rs`）
- **硬件支持**：GPIO / I2C / SPI / Arduino 上传（feature-gated，IoT 场景）
- **语音渠道**：WebRTC + Whisper ASR + TTS，实时语音对话

---

## Phase 14：对标 Moltis 架构健壮性改进

> 来源：2026-03-01 与 D:\Projects\moltis 深度对比分析。
> 不做 Web UI，聚焦架构、性能、安全、信息流转、记忆存储的健康度。
> 按优先级分组，逐项改进。

---

### P0：架构关键缺陷（健壮性前提）

#### 14-P0-1：`InboundMessage` / `OutboundMessage` 迁移到 `adaclaw-core`

**问题**：`InboundMessage` / `OutboundMessage` 定义在主二进制 `src/bus/mod.rs`，
library crate（channels / server 等）无法直接引用，造成循环依赖隐患，与 core trait 设计原则矛盾。

**对标**：Moltis 有独立的 `moltis-protocol` crate 专门存放 wire 类型。

**修复方案**：
- 将 `InboundMessage` / `OutboundMessage` / `MessageContent` 移入 `crates/adaclaw-core/src/channel.rs`（已有 `Channel` trait，放在一起语义一致）
- `src/bus/mod.rs` 改为 `pub use adaclaw_core::channel::{InboundMessage, OutboundMessage, MessageContent}`
- 更新所有引用（channels / server / daemon）

**修改文件**：
- `crates/adaclaw-core/src/channel.rs`
- `src/bus/mod.rs`
- 相关引用文件

- [ ] 将消息类型迁移到 `adaclaw-core`
- [ ] 更新所有引用，`cargo build` 通过

---

#### 14-P0-2：Library crate 的错误类型改用 `thiserror`

**问题**：`adaclaw-providers` / `adaclaw-memory` / `adaclaw-tools` / `adaclaw-channels` 等 library crate 全部使用 `anyhow::Error`，调用方无法 match 具体错误类型，错误语义丢失。

**对标**：Moltis CLAUDE.md 明确：`anyhow::Result` 用于应用层，`thiserror` 用于 library 层。

**修复方案（逐 crate 进行）**：
- `adaclaw-providers`：定义 `ProviderError`（已有 `error.rs`，扩展即可）
  - `AuthError(String)` / `RateLimit { retry_after_secs: Option<u64> }` / `BadRequest(String)` / `ServerError(String)` / `Network(#[from] reqwest::Error)` / `Json(#[from] serde_json::Error)`
- `adaclaw-memory`：定义 `MemoryError`
  - `Sqlite(#[from] rusqlite::Error)` / `Embedding(String)` / `NotFound(String)` / `Io(#[from] std::io::Error)`
- `adaclaw-tools`：定义 `ToolError`
  - `Io(#[from] std::io::Error)` / `SandboxViolation(String)` / `Timeout` / `McpError(String)`
- `adaclaw-channels`：定义 `ChannelError`
  - `Http(#[from] reqwest::Error)` / `Auth(String)` / `Disconnected` / `Config(String)`

- [ ] `adaclaw-providers`：`ProviderError` 完整定义，`Provider` trait 返回类型改为 `Result<T, ProviderError>`
- [ ] `adaclaw-memory`：`MemoryError` 完整定义，`Memory` trait 返回类型改为 `Result<T, MemoryError>`
- [ ] `adaclaw-tools`：`ToolError` 完整定义，`Tool` trait 返回类型改为 `Result<ToolResult, ToolError>`
- [ ] `adaclaw-channels`：`ChannelError` 完整定义

---

#### 14-P0-3：Session（对话历史）与 Memory（长期记忆）分离

**问题**：当前 `Category::Conversation` 把对话历史写入 Memory 库，会污染长期知识检索（用户查询时会混入大量对话碎片）。`AgentEngine` 的 history 是 in-memory，进程重启丢失（P13 已修复 engine 持久化，但仍存在分层问题）。

**对标**：Moltis 有独立的 `moltis-sessions` crate，Session ≠ Memory，两个独立系统：
- Session：按顺序存放 `(session_id, role, content, timestamp)`，顺序读取，不走 RRF
- Memory：只存用户明确保留的知识 + Agent 归纳的摘要，走向量/FTS 检索

**修复方案**：
- 在 `crates/adaclaw-memory/src/` 新增 `session_store.rs`
  - `SessionStore`：SQLite 表 `sessions(id, session_id, role, content, created_at)`
  - `append(session_id, role, content)` / `load(session_id, limit)` / `clear(session_id)` / `compact(session_id, summary)` 方法
- `AgentEngine` 的 history 优先从 `SessionStore` 加载，每轮追加写入
- `Category::Conversation` 从 Memory 中废弃，历史索引改走 SessionStore
- Memory 只保留 `Core / Daily / Global / Custom` 分类

**修改文件**：
- `crates/adaclaw-memory/src/session_store.rs`（新增）
- `crates/adaclaw-memory/src/lib.rs`
- `crates/adaclaw-core/src/memory.rs`（废弃 `Category::Conversation`）
- `src/agents/engine.rs`（history 加载/写入改用 SessionStore）
- `src/agents/instance.rs`

- [ ] 新增 `SessionStore`（SQLite 表 + CRUD 方法）
- [ ] `AgentEngine` 历史改为从 SessionStore 加载和写入
- [ ] 废弃 `Category::Conversation`，更新相关代码

---

### P1：安全补强（生产必须）

#### 14-P1-1：SSRF 防护（http_request 工具）

**问题**：`crates/adaclaw-tools/src/http.rs` 的 `http_request` 工具可以访问任意 URL，LLM 可能被 prompt injection 诱导访问 `http://192.168.1.1`、`http://localhost` 等内网地址，导致 SSRF 攻击。

**对标**：Moltis `moltis-network-filter` crate，DNS 解析级 SSRF 过滤，阻断 loopback / private / link-local / CGNAT。

**修复方案**：
- 在 `crates/adaclaw-tools/src/http.rs` 执行 HTTP 请求前进行 SSRF 检查：
  1. 解析 URL，提取 hostname
  2. 用 `tokio::net::lookup_host` 解析为 IP
  3. 拒绝：loopback（127.x.x.x / ::1）/ private（10.x / 172.16-31.x / 192.168.x）/ link-local（169.254.x）/ CGNAT（100.64-127.x）/ metadata（169.254.169.254）
  4. 拒绝非 80/443/8080/8443 等非标端口（可配置白名单）
- 错误信息清晰：`"SSRF blocked: target address 192.168.1.1 is a private IP"`

**修改文件**：
- `crates/adaclaw-tools/src/http.rs`
- 可选：抽取到 `crates/adaclaw-security/src/ssrf.rs`

- [ ] 实现 `is_ssrf_blocked(url)` 函数（DNS 解析 + IP 分类）
- [ ] `http_request` 工具执行前调用 SSRF 检查
- [ ] 添加单元测试（私有 IP / loopback / 公网 IP / metadata 地址）

---

#### 14-P1-2：运行时密钥改用 `secrecy::Secret<String>`

**问题**：`adaclaw-security/src/secrets.rs` 存储层用了 ChaCha20，但解密后的 key 在内存中是裸 `String`；Provider 实例中的 `api_key` 也是裸 `String`，可能出现在内存 dump / crash report 中。

**对标**：Moltis 全局使用 `secrecy::Secret<String>`，`expose_secret()` 只在实际消费点调用，`Debug` 输出为 `[REDACTED]`。

**修复方案**：
- 引入 `secrecy` crate（`secrecy = { features = ["serde"], version = "0.8" }`）
- Provider 结构体中 `api_key: String` → `api_key: Secret<String>`
- `secrets.rs` 中解密后包装为 `Secret<String>` 返回
- Provider 实现中只在调用 HTTP 请求时 `.expose_secret()`
- `Debug` impl 或 `#[derive(Debug)]` 需要注意 `Secret` 自动 redact

**修改文件**：
- 根 `Cargo.toml`（添加 `secrecy`）
- `crates/adaclaw-providers/src/openai.rs` 等各 provider
- `crates/adaclaw-security/src/secrets.rs`

- [ ] 添加 `secrecy` 依赖
- [ ] Provider 结构体 `api_key` 改为 `Secret<String>`
- [ ] `secrets.rs` 返回值改为 `Secret<String>`

---

#### 14-P1-3：Workspace 级别 Lint 强制

**问题**：当前根 `Cargo.toml` 没有 `[workspace.lints]`，导致各 crate 可以随意使用 `unwrap()` / `expect()` / unsafe，代码安全性完全依赖人工审查。

**对标**：Moltis 根 `Cargo.toml` 有：
```toml
[workspace.lints.rust]
unsafe_code = "deny"
[workspace.lints.clippy]
expect_used = "deny"
unwrap_used = "deny"
```

**修复方案**：
- 先以 `warn` 级别启用，消化存量 `unwrap`/`expect`（逐个改为 `?` / `ok_or_else` / `unwrap_or_default`）
- 全部消化后升级为 `deny`
- 所有 library crate 的 `Cargo.toml` 添加 `[lints] workspace = true`

**修改文件**：
- 根 `Cargo.toml`
- 各 crate `Cargo.toml`
- 消化现有 `unwrap`/`expect` 调用

- [ ] 根 `Cargo.toml` 添加 `[workspace.lints]`（先 warn 级别）
- [ ] 各 library crate `Cargo.toml` 添加 `[lints] workspace = true`
- [ ] 消化存量 `unwrap`/`expect`，升级到 `deny`

---

### P2：性能改善

#### 14-P2-1：SQLite 连接池改用 `deadpool-sqlite` 或 `sqlx`

**问题**：当前 `adaclaw-memory` 使用 `rusqlite` 直接操作，没有连接池。多个 Agent 并发写入时会争抢 SQLite 写锁，性能差；连接创建销毁频繁。

**对标**：Moltis 使用 `sqlx`（内置连接池 + migration 管理）。

**修复方案**：
- 方案 A（保守）：引入 `deadpool-sqlite`，在 `SqliteMemory` 中持有连接池
- 方案 B（彻底）：改用 `sqlx` + `sqlx::migrate!`，自动管理 migration
- 推荐方案 A 以减小改动范围：`deadpool-sqlite = "0.9"`
- `SqliteMemory::new()` 创建连接池（pool_size = 4），读操作用 pool，写操作显式 `BEGIN IMMEDIATE`

**修改文件**：
- `crates/adaclaw-memory/Cargo.toml`
- `crates/adaclaw-memory/src/sqlite.rs`

- [ ] 引入 `deadpool-sqlite`，`SqliteMemory` 改用连接池
- [ ] 写操作使用 `BEGIN IMMEDIATE` 事务，减少锁争用

---

#### 14-P2-2：MessageBus channel 改为 bounded，添加背压

**问题**：`src/bus/queue.rs` 的 MessageBus `mpsc` channel 类型未确认是否 bounded。若使用 `unbounded()`，高负载时会无限积压内存。

**修复方案**：
- 确认 / 改为 `tokio::sync::mpsc::channel(1024)`（bounded，1024 条消息缓冲）
- 当 channel 满时（`send` 超时），渠道应该 drop 消息并记录警告（而非无限等待）
- 添加 Prometheus 指标：`adaclaw_bus_queue_depth`

**修改文件**：
- `src/bus/queue.rs`
- `src/bus/mod.rs`

- [ ] 确认 mpsc channel 为 bounded(1024)
- [ ] channel 满时的背压处理（超时 + warn log）
- [ ] 可选：添加队列深度指标

---

#### 14-P2-3：全局内存分配器优化（jemalloc）

**问题**：默认使用系统 allocator，高并发场景下内存碎片较多，影响长期运行性能。

**对标**：Moltis 在 workspace 中引入 `tikv-jemallocator`。

**修复方案**：
- 在根 `Cargo.toml` 添加 `tikv-jemallocator = { version = "0.6", optional = true }`，feature = `jemalloc`
- `src/main.rs` 中条件编译启用：
  ```rust
  #[cfg(feature = "jemalloc")]
  #[global_allocator]
  static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
  ```
- `default` feature 不包含（不影响二进制大小目标）；release builds 可选开启

**修改文件**：
- 根 `Cargo.toml`
- `src/main.rs`

- [ ] 添加 `tikv-jemallocator` 可选依赖（feature = "jemalloc"）
- [ ] `src/main.rs` 条件编译全局 allocator

---

### P3：工程规范（代码质量）

#### 14-P3-1：QMD 子查询改为并发执行

**问题**：`crates/adaclaw-memory/src/query.rs` 的 `recall_with_qmd()` 可能串行执行多个子查询，性能差。

**修复方案**：
- 确认并发执行路径：`futures::future::join_all(sub_queries.iter().map(|q| memory.recall(q, ...)))`
- 所有子查询同时并发，多路 RRF 合并
- 添加单元测试验证并发性（通过执行时间或 join_all 断言）

**修改文件**：`crates/adaclaw-memory/src/query.rs`

- [ ] 确认/改为 `join_all` 并发执行子查询
- [ ] 添加测试验证

---

#### 14-P3-2：`time` crate 替代手动时间计算

**问题**：代码中存在手动时间戳计算（magic constant `86400` 等），应使用类型安全的 `time` 或 `chrono` crate。

**对标**：Moltis CLAUDE.md 明确：使用 `time` crate，不做手动 epoch 数学运算。

**修复方案**：
- 检查所有 `86400`、`3600` 等魔法常量，替换为 `Duration::days(1)` 等表达
- 统一使用已引入的 `chrono`（当前 workspace 已有）

**修改文件**：审查所有涉及时间计算的文件

- [ ] 搜索 magic time constants，替换为类型安全写法

---

#### 14-P3-3：Channel 消息静默失败改为明确错误响应

**问题**：Moltis CLAUDE.md 明确："Always respond to approved senders — no silent failures"。当前 Supervised 模式在消息渠道工具调用被拒绝时是静默 deny，用户不知道发生了什么。

**修复方案**：
- `approval.rs` `approve_tool_supervised()` 在非 CLI 渠道拒绝时，通过 `OutboundMessage` 发送"⚠️ 工具调用已被安全策略拒绝（Supervised 模式）"
- LLM 调用失败时也要回复用户（"AI 服务暂时不可用，请稍后再试"）
- `engine.rs` 中 LLM 错误捕获后，确保总能返回一条 outbound 消息

**修改文件**：
- `crates/adaclaw-security/src/approval.rs`
- `src/agents/engine.rs`

- [ ] Supervised 模式工具拒绝时回复用户明确消息
- [ ] Agent engine 中 LLM 失败时的兜底回复

---

### 改进进度汇总

| 优先级 | 条目 | 状态 |
|--------|------|------|
| P0-1 | InboundMessage/OutboundMessage 迁移到 adaclaw-core | ✅ 已完成（实现时已在 core 中） |
| P0-2 | Library crate 错误类型改用 thiserror | ✅ |
| P0-3 | Session 与 Memory 分离（SessionStore） | ✅ |
| P1-1 | SSRF 防护（http_request 工具） | ✅ |
| P1-2 | 运行时密钥改用 secrecy::Secret | ✅ |
| P1-3 | Workspace 级别 Lint 强制 | ✅ |
| P2-1 | SQLite 连接池（r2d2 + r2d2_sqlite） | ✅ |
| P2-2 | MessageBus bounded channel + 背压 | ✅ |
| P2-3 | jemalloc 全局分配器（可选 feature） | ✅ |
| P3-1 | QMD 子查询并发执行 | ✅ 已完成（实现时已用 join_all） |
| P3-2 | time crate 替代魔法常量 | ✅ |
| P3-3 | 消息静默失败改为明确错误响应 | ✅ |

---

## 使用说明

### 开始新 Task 时

1. 在新对话开头粘贴：
   ```
   我在做 AdaClaw 项目（D:\Projects\AdaClaw）。
   请读取 ARCHITECTURE.md 和 TASKS.md 了解架构设计。
   现在开始实现 Phase X（Task Y）。
   ```

2. 明确告知 AI 当前 Phase 的目标和交付物清单

3. 完成后在本文件对应 `- [ ]` 改为 `- [x]`

### 进度追踪

- Phase 0：✅ 完成
- Phase 1：✅ 完成
- Phase 2：✅ 完成
- Phase 3：✅ 完成
- Phase 4：✅ 完成
- Phase 5：✅ 完成
- Phase 6：✅ 完成（多渠道扩展：Telegram/CLI/DingTalk/Feishu/WeCom/Discord/Slack/Webhook）
- Phase 7：✅ 完成（可观察性：Prometheus/log/noop/RuntimeTrace；技能系统；身份系统；doctor/onboard 向导；Cloudflare/Tailscale/ngrok 隧道）
- Phase 8：✅ 完成（发布就绪：clippy 零警告 / README 完整英文版 / CI/CD / 安装脚本 / Homebrew formula）
- Phase 9：✅ 完成（渠道扩展：WhatsApp / Email / Matrix；server routes；3 个文档）
- Phase 10：✅ 完成（生态对接：MCP 完整实现 / Heartbeat / ClawHub 技能市场 / Groq Whisper）
- Phase 11：✅ 完成（对标竞品质量提升：第 1-5 轮全部完成）
- Phase 12：📦 闭源版本（Web UI / 多租户 / WASM / PostgreSQL / 硬件 / 语音渠道）
- Phase 13：🔄 进行中（代码审查修复 — 全面检查发现的 Bug 与遗漏）

---

*最后更新：2026-03-01（Phase 11 第 5 轮完成：Security / Approval UX — Telegram Inline Keyboard（callback_query + send_approval_prompt_msg + 按钮点击→Bus注入）；Approval 超时（PendingApprovalRequest 30min expiry + 自动剔除）；批量审批（CLI Y/N/A prompt + session_allowlist + non_cli_session_allowlist + allow_all_once token）；auto_approve/always_ask per-tool 列表；Channel trait send_approval_prompt() 可选方法；schema.rs SecurityConfig 新增 auto_approve/always_ask/approval_timeout_minutes；对标 zeroclaw approval/mod.rs 全部机制；151 个测试全部通过）*

---

## Phase 13：代码审查修复（全面检查发现的 Bug 与遗漏）

> 由全面代码审查（2026-03-01）发现。按优先级分组，每项修复后在此勾选。
> 所有 P0 必须在本 phase 完成才能发布。

---

### P0：严重 Bug（功能根本缺失，发布阻断）

#### P0-1：daemon 模式对话历史跨轮次丢失

**根因**：`src/daemon/run.rs` `agent_dispatch_loop()` 中每条消息都 `AgentEngine::new()`，
`AgentInstance::session_manager` 从未被利用，导致多轮对话完全无上下文。

**修复方案**：
- `AgentInstance` 中用 `Arc<Mutex<HashMap<String, AgentEngine>>>` 持久化每个 session 的 engine
- `agent_dispatch_loop()` 中按 `session_id` 从 instance 取出/创建 engine，执行后放回
- 注意：`AgentEngine` 含 `Mutex<Vec<MessageEntry>>` 已支持跨 spawn 访问

**修改文件**：
- `src/agents/instance.rs`：`SessionManager` 存 `AgentEngine`；新增 `get_or_create_engine()` 方法
- `src/daemon/run.rs`：`agent_dispatch_loop()` 改为从 instance 取 engine

- [ ] 实现 `AgentInstance::get_or_create_engine(session_id, memory)` 方法
- [ ] daemon dispatch loop 改为复用 engine 而非每次 new

---

#### P0-2：记忆后端初始化后丢弃（整个记忆系统失效）

**根因**：`src/daemon/run.rs` 步骤 4 中 `let _memory = ...` 下划线变量，初始化后再未使用；
没有任何地方调用 `.with_memory()` 接入 engine。

**修复方案**：
- 去掉下划线，改为 `let memory: Arc<dyn Memory> = ...`
- `get_or_create_engine()` 内对新 engine 调用 `.with_memory(memory.clone(), session_id)`
- daemon 中传 `memory` 进 dispatch loop

**修改文件**：
- `src/daemon/run.rs`：`_memory` → `memory`，传入 dispatch loop；dispatch loop 把 memory 传给 instance

- [ ] `_memory` → `memory`，传入 `agent_dispatch_loop`
- [ ] 新 engine 创建时附加 memory（`engine.with_memory(memory, session_id)`）

---

#### P0-3：Gateway Bearer Token 认证完全未实现

**根因**：`crates/adaclaw-server/src/middleware.rs` `require_auth` 是一行 TODO 空桩；
`server.rs` `build_router()` 没有挂载任何认证中间件，所有 API 端点对所有人开放。

**修复方案**：
- `middleware.rs` 实现 `require_auth`：从 `Authorization: Bearer <token>` 提取并与配置比较
- Bearer token 通过全局 `OnceLock<String>` 或 axum `State` 传入
- `build_router()` 对 `/v1/chat`、`/v1/stop` 挂载 `axum::middleware::from_fn(require_auth)`
- `/v1/status`、`/metrics`、`/pair` 可选择不要求认证

**修改文件**：
- `crates/adaclaw-server/src/middleware.rs`：实现 `require_auth` + Bearer token 全局存储
- `crates/adaclaw-server/src/server.rs`：`build_router()` 挂载中间件
- `crates/adaclaw-server/src/lib.rs`：导出 `set_bearer_token()` 初始化函数
- `src/daemon/run.rs`：daemon 启动时调用 `set_bearer_token()`

- [ ] 实现 `require_auth` middleware（Bearer token 比对）
- [ ] `build_router()` 对保护端点挂载认证中间件
- [ ] daemon 启动时注入 bearer token

---

#### P0-4：`POST /v1/chat` 路由完全未实现

**根因**：`crates/adaclaw-server/src/routes/chat.rs` 直接返回 `{"error": "Not implemented"}`。
Gateway 的核心 REST API 无法工作。

**修复方案**：实现基本的同步 chat 端点：
- 接受 `{"message": "...", "session_id": "...", "agent": "...（可选）"}` JSON body
- 通过 `AppMessageBus.send_inbound()` 注入消息，同步等待（用 `oneshot channel`）outbound 回复
- 设置合理超时（60s），超时返回 503
- 需要认证（P0-3 完成后自动保护）

**修改文件**：
- `crates/adaclaw-server/src/routes/chat.rs`：完整实现（含请求/响应类型）
- `crates/adaclaw-server/src/lib.rs`：导出 `set_message_bus()` 供 daemon 注入
- `src/daemon/run.rs`：daemon 启动时注入 bus 到 server

- [ ] 实现 `POST /v1/chat` 请求/响应结构体
- [ ] 实现同步等待机制（oneshot channel + 超时）
- [ ] daemon 注入 bus 到 gateway server

---

### P1：中等问题（影响完整性，发布前必须修复）

#### P1-1：WhatsApp 渠道实现完整但未注册到 daemon

**根因**：`src/daemon/run.rs` 渠道注册 match 中缺少 `"whatsapp"` 分支，
已实现的 WhatsApp channel 完全不可用。

**修改文件**：`src/daemon/run.rs`

- [ ] 在渠道注册 match 中添加 `"whatsapp"` 分支（参考其他渠道写法）

---

#### P1-2：`adaclaw stop` 和 `adaclaw status` 命令是静默空桩

**根因**：`src/main.rs` 两个命令只有 `info!(...)` 不打印给用户。
用户运行命令毫无反馈。

**修复方案**：
- `stop`：发送 HTTP POST 到 `gateway.bind/v1/stop`（从 config 读地址），带 bearer token
- `status`：发送 HTTP GET 到 `/v1/status`，打印响应；若连不上则说明 daemon 未运行

**修改文件**：`src/main.rs`，`src/cli/mod.rs`

- [ ] `adaclaw stop`：HTTP 调用 gateway `/v1/stop`，或打印"daemon not running"
- [ ] `adaclaw status`：HTTP 调用 gateway `/v1/status`，打印状态

---

#### P1-3：Pairing code 非密码学安全且无一次性机制

**根因**：`crates/adaclaw-server/src/pairing.rs` 使用 `rand::thread_rng()`（非 CSPRNG）；
没有存储/消耗机制，不是真正的"一次性"配对码。

**修复方案**：
- 改用 `OsRng`（`rand::rngs::OsRng`）生成 6 位数字码
- 在 `OnceLock<Mutex<Option<(String, Instant)>>>` 存储当前有效码（含过期时间，10分钟）
- `GET /pair`：生成并存储新码，旧码作废
- `POST /v1/chat`（或专用端点）：首次请求可用配对码换取 bearer token

**修改文件**：`crates/adaclaw-server/src/pairing.rs`

- [ ] 使用 `OsRng` 生成配对码
- [ ] 实现状态存储（全局 OnceLock + 10 分钟过期）
- [ ] GET /pair 消耗旧码、生成新码

---

### P2：轻微问题（代码质量，发布前修复）

#### P2-1：移除 `lazy_static` 无用依赖

**根因**：`Cargo.toml` 中有 `lazy_static = "1.4"` 但代码全用 `std::sync::LazyLock`。

**修改文件**：根 `Cargo.toml`

- [ ] 删除 `lazy_static` 依赖行，确认 `cargo build` 通过

---

#### P2-2：统一 `max_actions_per_hour` 默认值

**根因**：`src/config/schema.rs` `default_max_actions()` 返回 200，
而 `config.example.toml` 写 `max_actions_per_hour = 100`，不一致。

**修改文件**：`src/config/schema.rs` 或 `config.example.toml`

- [ ] 统一为 100（与 example 一致，更保守），更新函数和注释

---

#### P2-3：`docker-compose.yml` 占位符 + 缺少 Dockerfile

**根因**：`image: ghcr.io/your-org/adaclaw:latest` 中 `your-org` 未替换；
仓库中无 `Dockerfile`，用户无法本地构建。

**修改文件**：`docker-compose.yml`，新增 `Dockerfile`

- [ ] 替换 `your-org` 为 `worldflat21-lang`
- [ ] 添加多阶段构建 `Dockerfile`（builder + runtime，runtime 用 debian:bookworm-slim）
- [ ] docker-compose 中添加注释说明 `# build: .` 的使用方式

---

### P3：文档与细节（发布前修复，工作量小）

#### P3-1：`engine.rs` `truncate()` 函数参数名误导

**根因**：`fn truncate(s: &str, max_chars: usize)` 参数名叫 `max_chars` 但实际按字节截断。

**修改文件**：`src/agents/engine.rs`

- [ ] 重命名参数为 `max_bytes`，或改用 `.chars()` 计数实现真正的字符数截断

---

#### P3-2：`CHANGELOG.md` 补充初始版本条目

**修改文件**：`CHANGELOG.md`

- [ ] 在 `[Unreleased]` 下方添加 `## [0.1.0] - 2026-03-01` 及主要功能列表

---

#### P3-3：`ARCHITECTURE.md` 修正文件清单

**修改文件**：`ARCHITECTURE.md`

- [ ] 将 `LICENSE-MIT` / `LICENSE-APACHE` 改为 `LICENSE`（实际只有一个文件）
- [ ] 标注 `compatible.rs` 为"规划中，尚未实现"（当前已有此说明但措辞可更清晰）

---

### 修复进度汇总 ✅ 全部完成

| 优先级 | 条目 | 状态 |
|--------|------|------|
| P0-1 | daemon 对话历史跨轮次丢失 | ✅ |
| P0-2 | 记忆后端接入 engine | ✅ |
| P0-3 | Gateway Bearer Token 认证 | ✅ |
| P0-4 | POST /v1/chat 实现 | ✅ |
| P1-1 | WhatsApp 渠道注册 | ✅ |
| P1-2 | stop / status 命令 | ✅ |
| P1-3 | Pairing code 安全化（OsRng + TTL + 一次性） | ✅ |
| P2-1 | 移除 lazy_static | ✅ |
| P2-2 | max_actions_per_hour 默认值统一为 100 | ✅ |
| P2-3 | docker-compose 占位符 + 补 Dockerfile | ✅ |
| P3-1 | truncate() 参数名澄清（max_bytes） | ✅ |
| P3-2 | CHANGELOG v0.1.0 完整功能列表 | ✅ |
| P3-3 | ARCHITECTURE 修正 LICENSE 文件名 | ✅ |
