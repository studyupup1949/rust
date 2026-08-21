# acmap（中文说明）

`acmap` 是一个基于 `tokio` channel 的 Actor + 分片（sharded）异步 Map。

它提供了接近 DashMap 常见能力的 API，并包含两种写入路径：
- `insert`：请求-响应写入（返回旧值）
- `insert_fast`：仅发送写入（高吞吐场景）

## 特性

- 分片 Actor 模型，提升并发写入吞吐
- 异步 API
- `insert_fast` 快路径
- `examples/benchmark.rs` 内置直观 benchmark 输出

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
acmap = { path = "." }
```

## 快速开始

```rust
use acmap::AcMap;

#[tokio::main]
async fn main() {
    let map = AcMap::<u64, u64>::new();

    assert_eq!(map.insert(1, 10).await, None);
    assert_eq!(map.get(1).await, Some(10));

    map.insert_fast(2, 20);
    assert!(map.contains_key(2).await);

    assert_eq!(map.len().await, 2);
}
```

## 运行

```bash
cargo test -q
cargo run --example benchmark -q
```

## 项目结构

- `src/acmap/mod.rs`：公开 API、分片路由
- `src/acmap/messages.rs`：消息定义
- `src/acmap/shard.rs`：分片 Actor 运行循环
- `examples/benchmark.rs`：基准演示

## 相关文档

- 英文 README: [README.md](README.md)
- 中文贡献指南: [CONTRIBUTING.zh-CN.md](CONTRIBUTING.zh-CN.md)
- 中文安全策略: [SECURITY.zh-CN.md](SECURITY.zh-CN.md)

## 许可证

本项目采用 [MIT License](LICENSE)。
