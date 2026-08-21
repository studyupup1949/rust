# aditor

一个简单的时间计算工具库，支持：

- 时间加法：`add_seconds(timestamp, seconds)`
- 时间减法：`sub_seconds(timestamp, seconds)`
- 时间差值：`diff_seconds(ts1, ts2)`

## 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
aditor = "0.1"
```

## 使用

```rust
use aditor::{add_seconds, sub_seconds, diff_seconds};

fn main() {
    let t = 1_000u64;
    let t2 = add_seconds(t, 60);
    assert_eq!(t2, 1_060);

    let t3 = sub_seconds(t, 60);
    assert_eq!(t3, 940);

    let d = diff_seconds(1_000, 800);
    assert_eq!(d, 200);
}
```

## 许可证

MIT OR Apache-2.0
