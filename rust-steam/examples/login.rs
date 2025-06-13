//! Steam 登录示例
//! 
//! 这个示例演示了如何使用 rust-steam 库进行完整的 Steam 登录流程，
//! 包括连接、认证、2FA 处理和回调处理。

use rust_steam::client::{SteamClient, ClientSettings};
use rust_steam::authentication::{AuthSessionDetails, LogOnDetails, ConsoleAuthenticator, EAuthTokenPlatformType};
use rust_steam::callbacks::{CallbackManager, ConnectedCallback, DisconnectedCallback, LoggedOnCallback, LoggedOffCallback};
use rust_steam::types::{EResult, ProtocolType};
use std::env;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("🦀 Rust Steam 登录示例");
    println!("======================");
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    let (username, password) = if args.len() >= 3 {
        (args[1].clone(), args[2].clone())
    } else {
        println!("💡 用法: {} <用户名> <密码>", args[0]);
        println!("📝 使用示例用户名和密码进行演示...");
        ("demo_user".to_string(), "demo_password".to_string())
    };
    
    // 创建客户端配置
    let settings = ClientSettings {
        protocol_types: vec![ProtocolType::TCP, ProtocolType::WebSocket],
        server_list: vec![
            "steamcommunity.com:443".to_string(),
            "steampowered.com:443".to_string(),
        ],
        connect_timeout: Duration::from_secs(10),
        auto_retry: true,
        max_retries: 3,
    };
    
    // 创建 Steam 客户端
    let mut client = SteamClient::new(settings);
    println!("✅ Steam客户端已创建");
    
    // 获取回调管理器来订阅事件
    let callback_manager = client.get_callback_manager();
    
    // 订阅连接相关回调
    let _connected_sub = callback_manager.subscribe(|callback: &ConnectedCallback| {
        println!("🔗 已连接到Steam服务器! 服务器时间: {:?}", callback.server_time);
    });
    
    let _disconnected_sub = callback_manager.subscribe(|callback: &DisconnectedCallback| {
        if callback.is_user_initiated() {
            println!("🔌 已断开连接 (用户主动)");
        } else {
            println!("⚠️ 连接意外断开: {:?}", callback.reason);
        }
    });
    
    // 订阅登录相关回调
    let _logged_on_sub = callback_manager.subscribe(|callback: &LoggedOnCallback| {
        match callback.get_result() {
            EResult::OK => {
                println!("🎉 成功登录Steam!");
                println!("  👤 Steam ID: {}", callback.steam_id.id);
                println!("  📧 账户名: {}", callback.account_name);
                println!("  🏠 Cell ID: {}", callback.cell_id);
                if let Some(email) = &callback.email_domain {
                    println!("  📮 邮箱域名: {}", email);
                }
                if callback.vac_banned {
                    println!("  ⚠️ 账户被VAC封禁");
                }
            }
            result => {
                println!("❌ 登录失败: {:?}", result);
                if callback.get_extended_result() != EResult::OK {
                    println!("  📊 扩展结果: {:?}", callback.get_extended_result());
                }
            }
        }
    });
    
    let _logged_off_sub = callback_manager.subscribe(|callback: &LoggedOffCallback| {
        println!("👋 已登出Steam: {:?}", callback.get_result());
    });
    
    // 第一步：连接到Steam
    println!("\n🔌 步骤1: 连接到Steam服务器...");
    match client.connect().await {
        Ok(_) => {
            println!("✅ 连接成功");
            println!("📡 连接状态: {:?}", client.get_connection_state());
        }
        Err(e) => {
            println!("❌ 连接失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 等待连接回调处理
    sleep(Duration::from_millis(500)).await;
    
    // 第二步：准备认证详情
    println!("\n🔐 步骤2: 准备认证信息...");
    let auth_details = AuthSessionDetails {
        username: username.clone(),
        password: password.clone(),
        persistent_session: false,
        guard_data: None,
        authenticator: Some(Arc::new(ConsoleAuthenticator)),
        website_id: "Client".to_string(),
        device_friendly_name: "Rust Steam 客户端".to_string(),
        platform_type: EAuthTokenPlatformType::SteamClient,
    };
    
    println!("👤 用户名: {}", auth_details.username);
    println!("🖥️ 设备名: {}", auth_details.device_friendly_name);
    println!("🌐 网站ID: {}", auth_details.website_id);
    
    // 第三步：执行认证
    println!("\n🔑 步骤3: 开始认证流程...");
    let auth_result = match client.authenticate(auth_details).await {
        Ok(result) => {
            println!("✅ 认证流程完成!");
            println!("  📊 认证成功: {}", result.success);
            println!("  👤 账户名: {}", result.account_name);
            println!("  🔐 需要2FA: {}", result.requires_2fa);
            println!("  📧 需要邮箱验证: {}", result.requires_email_auth);
            println!("  📱 需要设备确认: {}", result.requires_device_confirmation);
            println!("  🎯 Steam ID: {}", result.steam_id.id);
            
            if result.had_two_factor_auth {
                println!("  ✅ 已完成两步验证");
            }
            
            if let Some(guard_data) = &result.new_guard_data {
                println!("  🔒 收到新的Guard数据 (长度: {})", guard_data.len());
            }
            
            result
        }
        Err(e) => {
            println!("❌ 认证失败: {}", e);
            return Err(e.into());
        }
    };
    
    // 第四步：使用认证结果登录
    if auth_result.success {
        println!("\n🚀 步骤4: 使用认证令牌登录...");
        
        let mut logon_details = LogOnDetails::new();
        logon_details.set_username(auth_result.account_name.clone());
        
        if let Some(access_token) = auth_result.access_token {
            logon_details.set_access_token(access_token);
            println!("🎫 访问令牌已设置");
        }
        
        if let Some(refresh_token) = auth_result.refresh_token {
            logon_details.set_refresh_token(refresh_token);
            println!("🔄 刷新令牌已设置");
        }
        
        logon_details.set_login_id(rand::random::<u32>());
        
        // 执行登录
        match client.log_on(logon_details).await {
            Ok(_) => {
                println!("✅ 登录命令已发送");
            }
            Err(e) => {
                println!("❌ 登录失败: {}", e);
                return Err(e.into());
            }
        }
        
        // 第五步：运行一段时间来处理回调
        println!("\n🔄 步骤5: 处理Steam事件...");
        println!("⏱️ 运行10秒来处理回调事件...");
        
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < Duration::from_secs(10) {
            callback_manager.run_wait_callbacks(1000).await;
            
            // 检查客户端状态
            if !client.is_running().await {
                println!("⚠️ 客户端已停止运行");
                break;
            }
        }
        
        // 第六步：登出
        println!("\n🚪 步骤6: 登出Steam...");
        match client.log_off().await {
            Ok(_) => {
                println!("✅ 登出命令已发送");
            }
            Err(e) => {
                println!("⚠️ 登出时出错: {}", e);
            }
        }
        
        // 处理登出回调
        callback_manager.run_wait_callbacks(1000).await;
    }
    
    // 第七步：断开连接
    println!("\n🔌 步骤7: 断开连接...");
    match client.disconnect().await {
        Ok(_) => {
            println!("✅ 已断开连接");
        }
        Err(e) => {
            println!("⚠️ 断开连接时出错: {}", e);
        }
    }
    
    // 最后处理断开连接回调
    callback_manager.run_wait_callbacks(500).await;
    
    println!("\n🎯 登录示例完成!");
    println!("📊 总结:");
    println!("  • 已演示完整的Steam登录流程");
    println!("  • 包含连接、认证、2FA处理、登录和登出");
    println!("  • 展示了回调系统的使用");
    println!("  • 模拟了真实的Steam协议交互");
    
    println!("\n💡 说明:");
    println!("  • 这是一个模拟实现，实际使用需要真实的Steam服务器连接");
    println!("  • 在生产环境中，认证器应该从用户输入读取真实的2FA代码");
    println!("  • Guard数据应该被安全存储以供后续使用");
    
    Ok(())
}

// 辅助函数：打印分隔线
#[allow(dead_code)]
fn print_separator(title: &str) {
    println!("\n{}", "=".repeat(50));
    println!("  {}", title);
    println!("{}", "=".repeat(50));
}