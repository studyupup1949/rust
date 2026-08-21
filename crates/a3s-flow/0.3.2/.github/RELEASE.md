# GitHub Actions 发布指南

## 自动发布到 crates.io

### 前置条件

1. **获取 crates.io API token**
   - 访问 https://crates.io/settings/tokens
   - 点击 "New Token"
   - 命名为 `a3s-flow-github-actions`
   - 复制生成的 token

2. **配置 GitHub Secret**
   - 访问 https://github.com/A3S-Lab/Flow/settings/secrets/actions
   - 点击 "New repository secret"
   - Name: `CARGO_REGISTRY_TOKEN`
   - Value: 粘贴你的 crates.io token
   - 点击 "Add secret"

### 发布流程

1. **更新版本号**
   ```bash
   # 编辑 Cargo.toml，修改 version 字段
   # 例如：version = "0.1.0" → version = "0.2.0"
   ```

2. **提交更改**
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git push origin main
   ```

3. **创建并推送 tag**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **自动发布**
   - GitHub Actions 会自动触发
   - 运行测试
   - 发布到 crates.io
   - 查看进度：https://github.com/A3S-Lab/Flow/actions

### CI 工作流

每次推送到 `main` 分支或创建 PR 时，会自动运行：
- `cargo fmt --check` — 检查代码格式
- `cargo clippy` — 运行 linter
- `cargo test` — 运行所有测试
- `cargo build --release` — 构建 release 版本
- `cargo doc` — 生成文档

### 手动发布（备用方案）

如果需要手动发布：

```bash
# 1. 确保所有测试通过
cargo test --all-features

# 2. 发布到 crates.io
cargo publish

# 3. 创建 git tag
git tag v0.2.0
git push origin v0.2.0
```

### 版本号规范

遵循 [Semantic Versioning](https://semver.org/)：

- **MAJOR** (1.0.0) — 不兼容的 API 变更
- **MINOR** (0.1.0) — 向后兼容的新功能
- **PATCH** (0.0.1) — 向后兼容的 bug 修复

### 故障排查

**发布失败：版本号已存在**
- crates.io 不允许覆盖已发布的版本
- 需要增加版本号后重新发布

**发布失败：权限错误**
- 检查 `CARGO_REGISTRY_TOKEN` secret 是否正确配置
- 确认 token 有发布权限

**CI 失败：测试不通过**
- 本地运行 `cargo test` 确认所有测试通过
- 修复失败的测试后重新推送
