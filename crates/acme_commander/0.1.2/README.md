# ACME Commander

一个现代化的 ACME 客户端，专注于 SSL/TLS 证书的自动化管理。项目名取自经典 RTS 游戏《Command & Conquer》的"指挥官"角色，寓意自动化证书调度。

## 🚀 核心特性

- **🔐 强制 ECDSA P-384**：专门使用 secp384r1 密钥，符合现代 TLS 最佳实践
- **🌐 DNS-01 专用**：专注于 DNS 挑战验证，无需公网 IP
- **☁️ Cloudflare 集成**：原生支持 Cloudflare DNS API
- **🔄 自动续期**：智能证书轮转，支持热加载
- **🧪 Dry-Run 模式**：安全的演练功能，验证配置无误
- **📊 详细日志**：基于 rat_logger 的高性能日志系统
- **⚡ 高性能**：基于 Tokio 异步运行时
- **🌍 多语言支持**：中文、日文、英文自动切换

## 📦 安装与构建

### 前置要求

- Rust 1.75+ (edition 2024)
- Cargo

### 构建

```bash
# 克隆项目
git clone https://git.sukiyaki.su/0ldm0s/acme_commander
cd acme_commander

# 构建发布版本
cargo build --release

# 安装到系统
cargo install --path .
```

## 🎯 快速开始

### 1. 获取新证书

```bash
# 基本用法
acme-commander certonly \
  --domains example.com \
  --domains www.example.com \
  --email admin@example.com \
  --cloudflare-token YOUR_CF_TOKEN

# 生产环境
acme-commander certonly \
  --domains example.com \
  --email admin@example.com \
  --cloudflare-token YOUR_CF_TOKEN \
  --production

# 演练模式（推荐首次使用）
acme-commander certonly \
  --domains example.com \
  --email admin@example.com \
  --cloudflare-token YOUR_CF_TOKEN \
  --dry-run
```

### 2. 续订证书

```bash
# 自动扫描并续订
acme-commander renew --cert-dir ./certs

# 强制续订所有证书
acme-commander renew --cert-dir ./certs --force
```

### 3. 验证 DNS 提供商

```bash
# 验证 Cloudflare Token
acme-commander validate cloudflare --cloudflare-token YOUR_CF_TOKEN
```

### 4. 生成密钥

```bash
# 生成证书密钥
acme-commander keygen --output cert.key --key-type certificate

# 生成账户密钥
acme-commander keygen --output account.key --key-type account
```

### 5. 查看证书信息

```bash
# 基本信息
acme-commander show cert.crt

# 详细信息
acme-commander show cert.crt --detailed
```

### 6. 撤销证书

```bash
acme-commander revoke cert.crt \
  --account-key account.key \
  --reason superseded \
  --production
```

## ⚙️ 配置选项

### 日志配置

```bash
# 启用详细日志
acme-commander --verbose certonly ...

# 启用调试日志
acme-commander --debug certonly ...

# 日志输出到文件
acme-commander --log-output file --log-file acme.log certonly ...

# 同时输出到终端和文件
acme-commander --log-output both --log-file acme.log certonly ...
```

## 📁 文件结构

默认情况下，证书文件将保存在 `./certs` 目录：

```
certs/
├── cert.crt          # 证书文件
├── cert.key          # 私钥文件
├── cert-account.key  # 账户密钥（如果自动生成）
└── cert-chain.crt    # 完整证书链（包含中间证书）
```

## 🔧 高级用法

### 自定义输出目录和文件名

```bash
acme-commander certonly \
  --domains example.com \
  --email admin@example.com \
  --cloudflare-token YOUR_CF_TOKEN \
  --output-dir /etc/ssl/private \
  --cert-name example-com
```

### 使用现有密钥

```bash
acme-commander certonly \
  --domains example.com \
  --email admin@example.com \
  --cloudflare-token YOUR_CF_TOKEN \
  --account-key ./existing-account.key \
  --cert-key ./existing-cert.key
```

### 强制续订

```bash
acme-commander certonly \
  --domains example.com \
  --email admin@example.com \
  --cloudflare-token YOUR_CF_TOKEN \
  --force-renewal
```

## 🏗️ 架构设计

### 核心模块

- **`acme/`** - ACME 协议实现
- **`crypto/`** - 加密算法和密钥管理
- **`dns/`** - DNS 提供商集成
- **`certificate/`** - 证书生命周期管理
- **`auth/`** - 认证和授权
- **`config/`** - 配置管理

### 依赖项目

- **`rat_logger`** - 高性能日志系统
- **`rat_quickdns`** - DNS 解析优化
- **`rat_quickmem`** - 内存管理优化

## 🔒 安全特性

- **强制 ECDSA P-384**：使用 secp384r1 曲线，提供更高的安全性
- **DNS-01 验证**：避免 HTTP-01 的安全风险
- **密钥隔离**：账户密钥和证书密钥分离管理
- **安全存储**：使用 `secrecy` crate 保护敏感数据
- **速率限制**：内置 ACME 服务器速率限制保护

## 🚨 注意事项

### 生产环境使用

1. **首次使用建议先进行 dry-run**：
   ```bash
   acme-commander certonly --dry-run ...
   ```

2. **备份重要密钥**：
   - 账户密钥丢失将无法管理现有证书
   - 建议将账户密钥存储在安全位置

3. **监控证书过期**：
   - 设置定时任务自动续期
   - 监控续期日志确保成功

### Cloudflare Token 权限

确保 Cloudflare API Token 具有以下权限：
- Zone:Zone:Read
- Zone:DNS:Edit
- 包含所有需要管理的域名

## 📈 性能优化

- **异步 I/O**：基于 Tokio 的高性能异步运行时
- **连接复用**：HTTP 客户端连接池
- **内存优化**：集成 rat_quickmem 内存管理
- **DNS 缓存**：集成 rat_quickdns 加速 DNS 解析

## 🐛 故障排除

### 常见问题

1. **Cloudflare Token 无效**
   ```bash
   # 验证 token
   acme-commander validate cloudflare --cloudflare-token YOUR_TOKEN
   ```

2. **DNS 传播延迟**
   - ACME Commander 会自动等待 DNS 传播
   - 如果失败，请检查 DNS 记录是否正确设置

3. **速率限制**
   - Let's Encrypt 有严格的速率限制
   - 建议使用测试环境进行调试

### 调试模式

```bash
# 启用详细调试信息
RUST_LOG=debug acme-commander --debug certonly ...
```

## 👥 维护者

- **0ldm0s** <oldmos@gmail.com>

## 🔗 相关链接

- [Let's Encrypt](https://letsencrypt.org/)
- [ACME RFC 8555](https://tools.ietf.org/html/rfc8555)
- [Cloudflare API](https://api.cloudflare.com/)

---

## 🛣️ 开发路线图

### 🎯 短期目标 (v0.2.x)

#### 新增 ACME 提供商支持
- **ZeroSSL 集成** [进行中]
  - ZeroSSL API 密钥验证
  - EAB 外部账户绑定支持
  - 商用证书管理接口

- **FreeSSL.cn 集成** [计划中]
  - 国产免费 SSL 证书服务
  - API 接口适配
  - 域名验证流程优化

#### 新增 DNS 提供商支持
- **阿里云 DNS** [计划中]
  - 阿里云 RAM 权限管理
  - AccessKey 认证支持
  - 批量域名管理

- **DNSPod** [计划中]
  - 腾讯云 DNSPod API 集成
  - 域名解析记录管理
  - 权限精细化控制

### 🚀 中期目标 (v0.3.x)

#### 增强功能
- **HTTP-01 挑战支持**
  - 为无法使用 DNS 挑战的场景提供替代方案
  - 端口自动检测和配置
  - 临时 HTTP 服务器

- **多证书批量管理**
  - 批量证书申请
  - 统一到期监控
  - 批量续期策略

- **证书生命周期管理**
  - 证书状态跟踪
  - 历史记录管理
  - 审计日志功能

### 🔮 长期目标 (v1.0.x)

#### 企业级功能
- **Web UI 管理界面**
  - 基于 Web 的证书管理面板
  - 可视化证书状态监控
  - 用户权限管理

- **分布式部署**
  - 多节点负载均衡
  - 集群化证书管理
  - 高可用性设计

- **API 服务器模式**
  - RESTful API 接口
  - 第三方系统集成
  - Webhook 通知机制

#### 安全增强
- **硬件安全模块 (HSM) 支持**
  - 私钥硬件存储
  - 国密算法支持
  - 等保合规要求

- **证书透明度 (CT) 支持**
  - SCT 签名嵌入
  - CT 监控集成
  - 证书可信度验证

### 📋 技术改进计划

#### 性能优化
- **并发处理优化**
  - DNS 挑战并行化
  - 批量操作优化
  - 内存使用优化

- **缓存机制**
  - DNS 查询缓存
  - ACME 账户信息缓存
  - 证书元数据缓存

#### 可观测性
- **指标监控**
  - Prometheus 指标导出
  - 性能指标收集
  - 健康检查端点

- **结构化日志**
  - JSON 格式日志
  - 日志聚合支持
  - 日志分析集成

### 🌍 生态扩展

#### 插件系统
- **DNS 提供商插件**
  - 插件化架构设计
  - 第三方 DNS 提供商扩展
  - 自定义 DNS 脚本支持

- **通知插件**
  - 邮件通知
  - 钉钉/企业微信通知
  - Slack/Teams 集成

#### 平台支持
- **容器化部署**
  - Docker 镜像发布
  - Kubernetes Operator
  - Helm Charts

- **包管理器支持**
  - DEB/RPM 包构建
  - Homebrew 支持
  - Windows MSI 安装包

---

**ACME Commander** - 让 SSL/TLS 证书管理变得简单而安全。