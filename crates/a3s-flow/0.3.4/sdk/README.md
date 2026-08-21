# A3S Flow SDK

A3S Flow 现在提供 Python 和 Node.js SDK,让你可以在这些语言中使用工作流引擎。

## 目录结构

```
crates/flow/sdk/
├── python/          # Python SDK (PyO3)
│   ├── Cargo.toml
│   ├── pyproject.toml
│   ├── README.md
│   └── src/
│       └── lib.rs
└── node/            # Node.js SDK (napi-rs)
    ├── Cargo.toml
    ├── build.rs
    ├── package.json
    ├── README.md
    └── src/
        └── lib.rs
```

## Python SDK

### 安装

```bash
cd crates/flow/sdk/python
pip install maturin
maturin develop
```

### 使用示例

```python
from a3s_flow import FlowEngine

# 创建引擎
engine = FlowEngine()

# 定义工作流
definition = {
    "nodes": [
        {"id": "start", "type": "noop"},
        {"id": "process", "type": "noop"}
    ],
    "edges": [
        {"source": "start", "target": "process"}
    ]
}

# 启动工作流
execution_id = engine.start(definition, {})

# 查询状态
state = engine.state(execution_id)
print(f"Status: {state.status}")

# 暂停/恢复
engine.pause(execution_id)
engine.resume(execution_id)

# 终止
engine.terminate(execution_id)

# 列出所有节点类型
node_types = engine.node_types()
print(f"Available nodes: {node_types}")
```

### API

- `FlowEngine()` - 创建引擎
- `start(definition: dict, variables: dict = None) -> str` - 启动工作流
- `pause(execution_id: str)` - 暂停
- `resume(execution_id: str)` - 恢复
- `terminate(execution_id: str)` - 终止
- `state(execution_id: str) -> ExecutionState` - 查询状态
- `node_types() -> list[str]` - 列出节点类型

## Node.js SDK

### 安装

```bash
cd crates/flow/sdk/node
npm install
npm run build:debug
```

### 使用示例

```typescript
import { FlowEngine } from '@a3s-lab/flow';

// 创建引擎
const engine = new FlowEngine();

// 定义工作流
const definition = {
  nodes: [
    { id: 'start', type: 'noop' },
    { id: 'process', type: 'noop' }
  ],
  edges: [
    { source: 'start', target: 'process' }
  ]
};

// 启动工作流
const executionId = await engine.start(definition, {});

// 查询状态
const state = await engine.state(executionId);
console.log(`Status: ${state.status}`);

// 暂停/恢复
await engine.pause(executionId);
await engine.resume(executionId);

// 终止
await engine.terminate(executionId);

// 列出所有节点类型
const nodeTypes = engine.nodeTypes();
console.log(`Available nodes: ${nodeTypes}`);
```

### API

- `new FlowEngine()` - 创建引擎
- `start(definition: object, variables?: object): Promise<string>` - 启动工作流
- `pause(executionId: string): Promise<void>` - 暂停
- `resume(executionId: string): Promise<void>` - 恢复
- `terminate(executionId: string): Promise<void>` - 终止
- `state(executionId: string): Promise<ExecutionState>` - 查询状态
- `nodeTypes(): string[]` - 列出节点类型

## 类型定义

### ExecutionState

```typescript
{
  status: 'running' | 'paused' | 'completed' | 'failed' | 'terminated';
  result?: FlowResult;  // 当 status 为 'completed' 时存在
  error?: string;       // 当 status 为 'failed' 时存在
}
```

### FlowResult

```typescript
{
  executionId: string;
  outputs: Record<string, unknown>;
  completedNodes: string[];
  skippedNodes: string[];
  context: Record<string, unknown>;
}
```

### FlowEvent

```typescript
{
  eventType: 'flow_started' | 'node_started' | 'node_completed' | 'node_skipped' | 'node_failed' | 'flow_completed' | 'flow_failed' | 'flow_terminated';
  executionId: string;
  nodeId?: string;
  output?: unknown;
  error?: string;
}
```

## 内置节点类型

两个 SDK 都支持所有内置节点类型:

- `noop` - 空操作
- `start` - 入口节点
- `end` - 出口节点
- `http-request` - HTTP 请求
- `if-else` - 条件分支
- `template-transform` - Jinja2 模板
- `variable-aggregator` - 变量聚合
- `code` - Rhai 脚本
- `csv-parse` - CSV 解析
- `iteration` - 迭代
- `sub-flow` - 子流程
- `llm` - LLM 调用
- `question-classifier` - 问题分类
- `assign` - 变量赋值
- `context-get` - 读取上下文
- `context-set` - 写入上下文
- `parameter-extractor` - 参数提取
- `loop` - 循环
- `list-operator` - 列表操作

## 发布

### Python SDK

```bash
cd crates/flow/sdk/python
maturin build --release
maturin publish
```

### Node.js SDK

```bash
cd crates/flow/sdk/node
npm run build
npm publish
```

## 技术细节

- **Python SDK**: 使用 PyO3 0.23 + maturin 构建
- **Node.js SDK**: 使用 napi-rs 2 构建,支持多平台 (macOS, Linux, Windows)
- **TypeScript**: Node.js SDK 自动生成 TypeScript 类型定义
- **异步运行时**:
  - Python: 多线程 Tokio (2x CPU cores)
  - Node.js: 单线程 Tokio
