# 贡献指南（中文）

## 本地开发

1. Fork 并克隆仓库。
2. 从 `master` 创建分支。
3. 完成修改并补充测试。
4. 提交前执行：

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test -q
```

5. 提交 Pull Request，建议包含：
- 问题背景
- 解决思路
- 行为变化或 benchmark 变化

## Commit 建议

- 每个 commit 保持单一职责。
- 提交信息明确描述改动。
- 避免无关的大范围格式化改动。
