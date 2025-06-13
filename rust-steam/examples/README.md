# Rust Steam 示例集合

这个目录包含了从 JavaSteam 项目移植的所有示例代码。每个示例都演示了 Rust Steam 库的不同功能。

## 📋 示例列表

### 认证相关

| 示例 | 文件名 | 功能描述 |
|------|--------|----------|
| 000 | `000_authentication.rs` | 基础认证 - 演示现代Steam认证系统（JWT令牌）登录流程 |
| 001 | `001_qr_authentication.rs` | QR码认证 - 使用QR码进行Steam认证，无需用户名密码 |
| 002 | `002_webcookie.rs` | Web Cookie获取 - 获取steamLoginSecure cookie用于网站API |

### 连接管理

| 示例 | 文件名 | 功能描述 |
|------|--------|----------|
| 003 | `003_serverlist.rs` | 服务器列表管理 - 优化连接成功率，Cell ID管理 |
| 011 | `011_debuglog.rs` | 调试日志 - 启用详细日志记录，诊断连接问题 |

### 社交功能

| 示例 | 文件名 | 功能描述 |
|------|--------|----------|
| 020 | `020_friends.rs` | 好友管理 - 好友列表、请求处理、状态更新 |
| 021 | `021_webapi.rs` | Web API - 调用Steam Web API获取游戏信息 |
| 022 | `022_pics.rs` | PICS系统 - 产品信息和变更系统 |

### 高级功能

| 示例 | 文件名 | 功能描述 |
|------|--------|----------|
| 013 | `013_unified_messages.rs` | 统一消息 - 使用Steam统一服务API |
| 014 | `014_matchmaking.rs` | 匹配系统 - Steam游戏匹配功能 |
| 023 | `023_download_app.rs` | 应用下载 - 下载Steam应用内容 |
| 024 | `024_download_userfiles.rs` | 用户文件下载 - Steam云存储文件下载 |

### 扩展示例

| 示例 | 文件名 | 功能描述 |
|------|--------|----------|
| 010 | `010_extending.rs` | 扩展功能 - 自定义处理器和回调 |
| 004 | `004_legacy_login.rs` | 传统登录 - 旧版登录方式（仅供参考） |
| 005 | `005_legacy_steamguard.rs` | 传统Steam Guard - 旧版2FA系统 |

## 🚀 运行示例

### 基本用法

```bash
# 运行基础认证示例
cargo run --example 000_authentication -- <用户名> <密码>

# 运行QR码认证示例（无需用户名密码）
cargo run --example 001_qr_authentication

# 运行好友管理示例
cargo run --example 020_friends -- <用户名> <密码>
```

### 高级示例

```bash
# 运行调试日志示例（查看详细内部信息）
cargo run --example 011_debuglog -- <用户名> <密码>

# 运行服务器列表管理示例
cargo run --example 003_serverlist -- <用户名> <密码>

# 运行Web Cookie获取示例
cargo run --example 002_webcookie -- <用户名> <密码>
```

## 📚 学习路径

### 新手推荐顺序

1. **000_authentication** - 了解基本的Steam认证流程
2. **001_qr_authentication** - 学习免密码认证方式
3. **020_friends** - 掌握社交功能基础
4. **011_debuglog** - 学习调试技巧

### 进阶开发者

1. **003_serverlist** - 优化连接性能
2. **002_webcookie** - 集成Web功能
3. **021_webapi** - 调用Steam API
4. **013_unified_messages** - 使用高级API

## 🔧 配置要求

### 环境准备

- Rust 1.70+ 
- Tokio 异步运行时
- 有效的Steam账户

### 依赖说明

所有示例都使用以下核心功能：
- `rust_steam::prelude::*` - 核心类型和功能
- `tokio::time` - 异步时间操作
- `std::env` - 命令行参数处理

## ⚠️ 重要提示

### 安全注意事项

1. **账户安全**
   - 不要在代码中硬编码用户名和密码
   - 使用环境变量或配置文件存储敏感信息
   - 启用Steam Guard两步验证

2. **API限制**
   - 遵守Steam API使用条款
   - 避免频繁请求导致限流
   - 适当添加请求间隔

3. **数据保护**
   - 访问令牌具有时效性，需要定期续期
   - 不要分享或泄露认证令牌
   - 合理设置会话持久化选项

### 故障排除

1. **认证问题**
   - 检查用户名和密码是否正确
   - 确认是否需要2FA验证
   - 查看是否有Steam Guard邮件

2. **连接问题**
   - 检查网络连接
   - 查看防火墙设置
   - 尝试使用调试日志示例排查

3. **API错误**
   - 查看返回的错误代码
   - 检查API调用参数
   - 确认Steam服务状态

## 📖 相关文档

- [Rust Steam 库文档](../README.md)
- [Steam API 官方文档](https://steamapi.xpaw.me/)
- [JavaSteam 原始项目](https://github.com/Longi94/JavaSteam)

## 🤝 贡献指南

欢迎提交新的示例或改进现有示例：

1. 保持示例简洁明了
2. 添加充分的中文注释
3. 包含错误处理逻辑
4. 提供清晰的使用说明

## 📜 许可证

本示例集合遵循与主项目相同的许可证条款。