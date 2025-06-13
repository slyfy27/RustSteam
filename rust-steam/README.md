# Rust Steam

[![Crates.io](https://img.shields.io/crates/v/rust-steam.svg)](https://crates.io/crates/rust-steam)
[![Documentation](https://docs.rs/rust-steam/badge.svg)](https://docs.rs/rust-steam)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**rust-steam** 是一个纯Rust实现的Steam协议客户端库，移植自优秀的 [JavaSteam](https://github.com/Longi94/JavaSteam) 项目。该库允许您连接到 Valve 的 Steam 网络并与之交互。

## 功能特性

- ✅ **现代认证** - 支持JWT令牌和各种2FA方法（TOTP、短信、邮箱）
- ✅ **多种连接协议** - 支持TCP、UDP和WebSocket连接
- ✅ **异步/等待支持** - 完全异步API，基于Tokio
- ✅ **类型安全** - 使用Rust的类型系统确保Steam协议消息的类型安全
- ✅ **回调系统** - 事件驱动的架构，易于处理Steam事件
- ✅ **内容下载** - 支持从Steam下载游戏内容和文件
- ✅ **好友管理** - 管理Steam好友列表和状态
- ✅ **商店功能** - 访问Steam商店和社区功能
- 🚧 **实时消息** - Steam聊天和消息功能（开发中）
- 🚧 **游戏协调** - 游戏启动和协调功能（开发中）

## 快速开始

### 添加依赖

在您的 `Cargo.toml` 中添加：

```toml
[dependencies]
rust-steam = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 基本使用示例

```rust
use rust_steam::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建Steam客户端
    let client = SteamClient::new().await?;
    
    // 连接到Steam
    client.connect().await?;
    
    // 认证
    let auth_details = AuthSessionDetails {
        username: "your_username".to_string(),
        password: "your_password".to_string(),
        authenticator: Some(Box::new(ConsoleAuthenticator)),
        ..Default::default()
    };
    
    let poll_result = client.authenticate(auth_details).await?;
    
    // 登录Steam
    let mut logon_details = LogOnDetails::new();
    logon_details.set_username(poll_result.account_name);
    logon_details.set_access_token(poll_result.access_token);
    logon_details.set_refresh_token(poll_result.refresh_token);
    
    client.log_on(logon_details).await?;
    
    // 在这里进行Steam操作...
    
    // 断开连接
    client.disconnect().await?;
    
    Ok(())
}
```

### 使用回调系统

```rust
use rust_steam::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SteamClient::new().await?;
    let callback_manager = client.get_callback_manager();
    
    // 订阅连接事件
    let _connected = callback_manager.subscribe(|callback: ConnectedCallback| {
        println!("已连接到Steam！");
    });
    
    // 订阅登录事件
    let _logged_on = callback_manager.subscribe(|callback: LoggedOnCallback| {
        match callback.get_result() {
            EResult::OK => println!("登录成功！Steam ID: {}", callback.steam_id),
            result => println!("登录失败：{:?}", result),
        }
    });
    
    // 订阅断开连接事件
    let _disconnected = callback_manager.subscribe(|callback: DisconnectedCallback| {
        if callback.is_user_initiated() {
            println!("用户主动断开连接");
        } else {
            println!("意外断开连接：{:?}", callback.reason);
        }
    });
    
    // 连接和认证...
    client.connect().await?;
    
    // 运行事件循环
    while client.is_running().await {
        callback_manager.run_wait_callbacks(1000).await;
    }
    
    Ok(())
}
```

## 示例

查看 `examples/` 目录获取更多示例：

- **authentication.rs** - 基本认证示例
- **friends_list.rs** - 好友列表管理示例

运行示例：

```bash
# 认证示例
cargo run --example authentication your_username your_password

# 好友列表示例  
cargo run --example friends_list your_username your_password
```

## 配置

### 自定义客户端配置

```rust
use rust_steam::prelude::*;
use std::time::Duration;

let config = SteamConfiguration::new()
    .with_protocol_types(vec![ProtocolType::TCP, ProtocolType::WebSocket])
    .with_connection_timeout(Duration::from_secs(30))
    .with_web_api_key("your_web_api_key".to_string())
    .with_cell_id(123);

let client = SteamClient::with_configuration(config).await?;
```

### 双因素认证

库支持多种2FA方法：

```rust
use rust_steam::authentication::*;

// 使用控制台认证器（手动输入代码）
let auth_details = AuthSessionDetails {
    username: "username".to_string(),
    password: "password".to_string(),
    authenticator: Some(Box::new(ConsoleAuthenticator)),
    ..Default::default()
};

// 或者实现自定义认证器
#[derive(Debug)]
struct CustomAuthenticator {
    totp_secret: String,
}

impl Authenticator for CustomAuthenticator {
    async fn get_device_code(&self, _previous_incorrect: bool) -> Result<String, SteamError> {
        // 使用TOTP secret生成代码
        Ok(generate_totp_code(&self.totp_secret))
    }
    
    // 实现其他方法...
}
```

## 架构概览

rust-steam 采用模块化架构：

- **client** - 主要的Steam客户端类
- **authentication** - 现代Steam认证系统
- **callbacks** - 事件驱动的回调系统
- **handlers** - 各种Steam服务的处理器
- **networking** - 网络连接管理
- **types** - Steam协议类型和枚举
- **utils** - 实用工具函数

## 与JavaSteam的差异

虽然rust-steam是从JavaSteam移植的，但有一些重要差异：

1. **异步优先** - 所有网络操作都是异步的
2. **类型安全** - 利用Rust的类型系统提供更好的安全性
3. **内存安全** - 无需担心内存泄漏或空指针
4. **错误处理** - 使用Rust的Result类型进行明确的错误处理
5. **并发性** - 基于Tokio的高性能异步运行时

## 开发状态

这是一个正在开发中的项目。核心功能（连接、认证、基本回调）已经实现，但仍有许多功能需要添加：

- [x] 基本连接和认证
- [x] 回调系统
- [x] 用户会话管理
- [ ] 好友系统
- [ ] 游戏协调
- [ ] 内容下载
- [ ] Steam聊天
- [ ] 商店API
- [ ] 社区功能

## 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解如何参与开发。

## 许可证

本项目基于 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 致谢

- 感谢 [JavaSteam](https://github.com/Longi94/JavaSteam) 项目提供了出色的基础
- 感谢 [SteamKit2](https://github.com/SteamRE/SteamKit2) 项目的原始协议实现
- 感谢Steam协议逆向工程社区的贡献

## 免责声明

本项目不隶属于Valve Corporation。Steam是Valve Corporation的商标。