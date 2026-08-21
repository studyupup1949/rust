# 设计：将 MCP 改造为外部工具节点

## 背景

当前 `a3s-flow` 将 MCP 节点作为内置节点实现（`src/nodes/mcp.rs`），这违反了"Minimal Core + External Extensions"的设计原则。MCP 应该作为外部工具节点由上层应用（如 SafeClaw）来实现和注册。

## 目标

1. **保持核心精简**：从 `a3s-flow` 核心移除 MCP 节点
2. **外部扩展**：将 MCP 作为可选的外部工具节点
3. **向后兼容**：提供迁移路径，不破坏现有工作流

## 设计方案

### 1. 核心架构调整

#### 1.1 移除内置 MCP 节点

**删除文件：**
- `src/nodes/mcp.rs`

**修改文件：**
- `src/nodes/mod.rs` — 移除 `pub mod mcp;`
- `src/registry.rs` — 从 `with_defaults()` 中移除 `McpNode` 注册
- `src/lib.rs` — 更新文档注释，移除 MCP 节点说明

#### 1.2 定义核心节点清单

**核心节点（保留在 `with_defaults()`）：**

| 类别 | 节点类型 | 说明 |
|------|---------|------|
| **基础** | `noop`, `start`, `end` | 工作流基础结构 |
| **逻辑** | `if-else`, `iteration`, `loop`, `sub-flow` | 控制流 |
| **数据** | `template-transform`, `variable-aggregator`, `assign`, `list-operator` | 数据处理 |
| **计算** | `code` | 沙盒脚本执行 |
| **网络** | `http-request` | HTTP 调用 |
| **LLM** | `llm`, `question-classifier`, `parameter-extractor` | AI 能力 |

**移除节点（改为外部扩展）：**
- `mcp` — 外部工具调用，由上层应用实现

### 2. 外部工具节点机制

#### 2.1 创建独立的 MCP 工具节点 crate

**新建 crate：`a3s-flow-mcp`**

```
crates/flow-mcp/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── mcp_node.rs  (从 flow/src/nodes/mcp.rs 迁移)
└── README.md
```

**`Cargo.toml`：**
```toml
[package]
name = "a3s-flow-mcp"
version = "0.1.0"
edition = "2021"

[dependencies]
a3s-flow = { path = "../flow" }
async-trait = "0.1"
serde_json = "1.0"
reqwest = { version = "0.12", features = ["stream"] }
futures-util = "0.3"
```

**`lib.rs`：**
```rust
//! MCP tool node for a3s-flow
//!
//! This is an external extension that adds Model Context Protocol support
//! to a3s-flow workflows.

mod mcp_node;
pub use mcp_node::McpNode;
```

#### 2.2 在 SafeClaw 中注册 MCP 节点

**`apps/safeclaw/crates/safeclaw/Cargo.toml`：**
```toml
[dependencies]
a3s-flow = { path = "../../../../crates/flow" }
a3s-flow-mcp = { path = "../../../../crates/flow-mcp" }
```

**`apps/safeclaw/crates/safeclaw/src/workflows/mod.rs`：**
```rust
use a3s_flow::{FlowEngine, NodeRegistry};
use a3s_flow_mcp::McpNode;
use std::sync::Arc;

pub fn create_flow_engine() -> FlowEngine {
    let mut registry = NodeRegistry::with_defaults();

    // 注册外部工具节点
    registry.register(Arc::new(McpNode));

    FlowEngine::new(registry)
}
```

### 3. 前端节点目录调整

#### 3.1 SafeClaw 前端节点分类

**`apps/safeclaw/src/pages/workflow/node-catalog.ts`：**

```typescript
export const NODE_CATALOG: NodeCatalogEntry[] = [
  // ── 内置节点（来自 a3s-flow core）──────────────────────
  { type: "http-request", label: "HTTP 请求", category: "网络", ... },
  { type: "if-else", label: "条件分支", category: "逻辑", ... },
  { type: "code", label: "代码", category: "逻辑", ... },
  // ... 其他核心节点
];

export const TOOL_CATALOG: NodeCatalogEntry[] = [
  // ── 工具节点（外部扩展）──────────────────────────────────
  { type: "mcp", label: "MCP 工具", category: "工具",
    icon: Wrench,
    iconColor: "text-purple-500",
    headerBg: "bg-purple-50 dark:bg-purple-950/30",
    description: "调用 Model Context Protocol 服务器上的工具",
    defaultData: { transport: "sse", server_url: "", tool_name: "", arguments: {} }
  },
  // 未来可以添加更多工具节点：
  // - Zapier 集成
  // - Slack 通知
  // - 数据库查询
  // - 自定义 API 调用
];
```

#### 3.2 节点选择面板 Tab 实现

**已实现（`editor.tsx`）：**
- ✅ Tab 栏：内置节点 / 工具节点
- ✅ 内置节点显示 `NODE_CATALOG`
- ✅ 工具节点显示 `TOOL_CATALOG`（当前为空状态）

**下一步：**
1. 创建 `TOOL_CATALOG` 数组
2. 在"工具节点" Tab 中渲染 `TOOL_CATALOG`
3. 添加 MCP 节点到 `TOOL_CATALOG`

### 4. 迁移路径

#### 4.1 Phase 1：创建外部 MCP crate（不破坏现有代码）

1. 创建 `crates/flow-mcp/` 目录
2. 复制 `mcp.rs` 到新 crate
3. 在 SafeClaw 中注册 MCP 节点
4. 测试确保功能正常

#### 4.2 Phase 2：从核心移除 MCP（破坏性变更）

1. 从 `a3s-flow` 移除 `src/nodes/mcp.rs`
2. 更新 `registry.rs` 和 `lib.rs`
3. 更新版本号：`a3s-flow` → `0.2.0`（breaking change）
4. 更新文档和 CHANGELOG

#### 4.3 Phase 3：前端 UI 调整

1. 将 MCP 从内置节点移到工具节点
2. 更新节点目录分类
3. 更新节点选择面板

### 5. 优势

#### 5.1 核心精简
- `a3s-flow` 核心减少 ~700 行代码
- 移除 MCP 协议依赖（`reqwest`, `futures-util`）
- 更清晰的职责边界

#### 5.2 灵活扩展
- 用户可以选择是否启用 MCP 支持
- 可以独立升级 MCP 实现
- 为其他工具节点提供模板（Zapier, Slack, etc.）

#### 5.3 符合设计原则
- ✅ Minimal Core：核心只包含必需的节点类型
- ✅ External Extensions：MCP 作为可选扩展
- ✅ 清晰的扩展点：`NodeRegistry::register()`

### 6. 实施检查清单

**Phase 1：创建外部 crate**
- [ ] 创建 `crates/flow-mcp/` 目录结构
- [ ] 迁移 `mcp.rs` 代码
- [ ] 添加 `Cargo.toml` 和 `README.md`
- [ ] 在 SafeClaw 中注册 MCP 节点
- [ ] 运行测试确保功能正常

**Phase 2：从核心移除**
- [ ] 删除 `src/nodes/mcp.rs`
- [ ] 更新 `src/nodes/mod.rs`
- [ ] 更新 `src/registry.rs`
- [ ] 更新 `src/lib.rs` 文档
- [ ] 更新 `Cargo.toml` 版本号 → `0.2.0`
- [ ] 更新 `CHANGELOG.md`
- [ ] 运行所有测试

**Phase 3：前端调整**
- [ ] 创建 `TOOL_CATALOG` 数组
- [ ] 添加 MCP 节点到工具目录
- [ ] 更新节点选择面板逻辑
- [ ] 测试节点创建和配置

**Phase 4：文档更新**
- [ ] 更新 `crates/flow/README.md`
- [ ] 更新 `crates/flow-mcp/README.md`
- [ ] 更新 `apps/safeclaw/README.md`
- [ ] 添加迁移指南

### 7. 未来扩展

基于这个模式，可以添加更多工具节点：

**计划中的工具节点：**
- `zapier` — Zapier 集成
- `slack` — Slack 通知
- `email` — 邮件发送
- `database` — 数据库查询
- `webhook` — Webhook 调用
- `file-system` — 文件系统操作
- `git` — Git 操作

**实现方式：**
1. 每个工具节点一个独立 crate：`a3s-flow-{tool}`
2. 在 SafeClaw 中按需注册
3. 前端在"工具节点" Tab 中展示

## 总结

这个设计将 MCP 从核心移到外部扩展，符合"Minimal Core + External Extensions"原则，同时为未来添加更多工具节点提供了清晰的模式。
