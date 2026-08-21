# A3S Flow 发布指南

## 前置要求

### 1. GitHub Secrets 配置

在 GitHub 仓库设置中添加以下 secrets:

```bash
# crates.io token
gh secret set CARGO_TOKEN --repo A3S-Lab/Flow

# npm token (需要 @a3s-lab org 权限)
gh secret set NPM_TOKEN --repo A3S-Lab/Flow

# PyPI token
gh secret set PYPI_TOKEN --repo A3S-Lab/Flow
```

### 2. 获取 tokens

**crates.io:**
```bash
cat ~/.cargo/credentials.toml
# 复制 token
```

**npm:**
```bash
npm login
cat ~/.npmrc | grep authToken
# 复制 token (去掉 //registry.npmjs.org/:_authToken= 前缀)
```

**PyPI:**
```bash
# 在 https://pypi.org/manage/account/token/ 创建 token
```

## 发布流程

### 1. 更新版本号

```bash
cd /Users/roylin/Desktop/code/a3s/crates/flow

# 设置新版本
NEW_VERSION="0.3.4"

# 更新 Rust crate
sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

# 更新 Python SDK
sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" sdk/python/pyproject.toml
sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" sdk/python/Cargo.toml

# 更新 Node SDK (主包 + 所有平台包)
find sdk/node -name "package.json" -not -path "*/node_modules/*" \
  -exec sed -i '' "s/\"0.3.3\"/\"$NEW_VERSION\"/g" {} \;
```

### 2. 提交并打标签

```bash
# 提交更改
git add -A
git commit -m "chore: bump version to v$NEW_VERSION"

# 打标签
git tag "v$NEW_VERSION"

# 推送
git push origin main --tags
```

### 3. 监控发布

```bash
# 查看 workflow 运行状态
gh run list --repo A3S-Lab/Flow --limit 5

# 查看特定 run 的详情
gh run view <run-id> --repo A3S-Lab/Flow

# 实时查看日志
gh run watch <run-id> --repo A3S-Lab/Flow
```

## 发布内容

一次发布会自动发布到:

1. **crates.io** - Rust crate `a3s-flow`
2. **npm** - Node.js SDK `@a3s-lab/flow` (8 个包: 主包 + 7 个平台包)
3. **PyPI** - Python SDK `a3s-flow` (7 个平台的 wheels)
4. **GitHub Releases** - 自动创建 release 并生成 release notes

## 平台支持

### Node.js SDK
- macOS: arm64, x64
- Linux: x64 (glibc/musl), arm64 (glibc/musl)
- Windows: x64

### Python SDK
- macOS: arm64, x64
- Linux: x64 (glibc/musl), arm64 (glibc/musl)
- Windows: x64
- Python 版本: 3.9, 3.10, 3.11, 3.12, 3.13

## 验证发布

### crates.io
```bash
cargo search a3s-flow
```

### npm
```bash
npm view @a3s-lab/flow
npm view @a3s-lab/flow-darwin-arm64
```

### PyPI
```bash
pip search a3s-flow  # 或访问 https://pypi.org/project/a3s-flow/
```

## 故障排查

### crates.io 发布失败
- 检查 `CARGO_TOKEN` secret 是否正确
- 确认版本号未被使用

### npm 发布失败
- 检查 `NPM_TOKEN` secret 是否正确
- 确认有 @a3s-lab org 的发布权限
- 平台包必须先于主包发布

### PyPI 发布失败
- 检查 `PYPI_TOKEN` secret 是否正确
- 确认版本号未被使用
- `--skip-existing` 会跳过已存在的版本

### Workflow 失败
```bash
# 查看失败的 job
gh run view <run-id> --repo A3S-Lab/Flow

# 重新运行失败的 job
gh run rerun <run-id> --repo A3S-Lab/Flow
```

## 手动发布 (备用)

如果 CI 失败,可以手动发布:

### Rust crate
```bash
cargo publish
```

### Node.js SDK
```bash
cd sdk/node
npm run build
npm publish --access public
```

### Python SDK
```bash
cd sdk/python
pip install maturin
maturin build --release
maturin publish
```

## 回滚

如果发布有问题:

### crates.io
无法删除已发布的版本,只能 yank:
```bash
cargo yank --vers 0.3.4 a3s-flow
```

### npm
```bash
npm unpublish @a3s-lab/flow@0.3.4
```

### PyPI
在 PyPI 网站上删除版本

## CI/CD 架构

```
.github/
├── setup-workspace.sh          # 重构 repo 为 workspace
└── workflows/
    ├── ci.yml                  # Push/PR: fmt, clippy, test
    ├── release.yml             # Tag: 编排所有发布
    ├── publish-node.yml        # Node SDK 多平台构建 + npm 发布
    └── publish-python.yml      # Python SDK 多平台构建 + PyPI 发布
```

## 注意事项

1. **版本同步**: 确保 Cargo.toml, pyproject.toml, 和所有 package.json 的版本号一致
2. **测试通过**: 发布前确保 `cargo test --all-features` 通过
3. **格式检查**: 确保 `cargo fmt` 和 `cargo clippy` 无警告
4. **标签格式**: 必须使用 `v` 前缀 (如 `v0.3.4`)
5. **网络问题**: 如果构建超时,可以重新运行 workflow
