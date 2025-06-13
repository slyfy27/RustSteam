// 示例000: 基础认证
// 
// 此示例演示如何使用现代Steam认证系统（JWT令牌）进行登录
// 包含2FA支持和完整的认证流程

use rust_steam::prelude::*;
use tokio::time::{sleep, Duration};
use std::env;

#[tokio::main]
async fn main() -> Result<(), SteamError> {
    println!("=== JavaSteam样例000: 基础认证 ===\n");
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("使用方法: {} <用户名> <密码>", args[0]);
        return Ok(());
    }
    
    let username = &args[1];
    let password = &args[2];
    
    println!("🔐 开始Steam认证流程...");
    
    // 创建Steam客户端
    let mut client = SteamClient::new(ClientSettings::default());
    
    // 获取回调管理器
    let callback_manager = client.get_callback_manager();
    
    // 订阅连接事件
    let _connected_sub = callback_manager.subscribe(|callback: &ConnectedCallback| {
        println!("✅ 已连接到Steam服务器!");
        println!("   服务器时间: {:?}", callback.server_time);
    });
    
    // 订阅断开连接事件
    let _disconnected_sub = callback_manager.subscribe(|callback: &DisconnectedCallback| {
        println!("❌ 与Steam服务器的连接已断开");
        if let Some(reason) = &callback.reason {
            println!("   原因: {}", reason);
        }
        println!("   用户主动断开: {}", callback.user_initiated);
    });
    
    // 订阅登录成功事件
    let _logged_on_sub = callback_manager.subscribe(|callback: &LoggedOnCallback| {
        match callback.result {
            EResult::OK => {
                println!("🎉 登录成功！");
                println!("   Steam ID: {}", callback.steam_id.render());
                println!("   账户名: {}", callback.account_name);
                println!("   Cell ID: {}", callback.cell_id);
                if let Some(domain) = &callback.email_domain {
                    println!("   邮箱域名: {}", domain);
                }
                println!("   VAC封禁状态: {}", callback.vac_banned);
                println!("   现在可以在Steam上执行各种操作");
            }
            _ => {
                println!("❌ 登录失败: {:?}", callback.result);
                if callback.extended_result != EResult::OK {
                    println!("   扩展错误: {:?}", callback.extended_result);
                }
            }
        }
    });
    
    // 订阅登出事件
    let _logged_off_sub = callback_manager.subscribe(|callback: &LoggedOffCallback| {
        println!("👋 已从Steam登出: {:?}", callback.result);
    });
    
    // 连接到Steam
    println!("🌐 正在连接Steam服务器...");
    client.connect().await?;
    
    // 等待连接完成
    sleep(Duration::from_secs(1)).await;
    
    // 创建认证详情
    let auth_details = AuthSessionDetails {
        username: username.to_string(),
        password: password.to_string(),
        persistent_session: false,
        guard_data: None,
        authenticator: Some(Arc::new(ConsoleAuthenticator)),
        device_friendly_name: "Rust Steam Client".to_string(),
        website_id: "Client".to_string(),
        platform_type: EPlatformType::Unknown,
    };
    
    println!("🔑 执行认证会话...");
    
    // 执行认证
    match client.authenticate(auth_details).await {
        Ok(auth_result) => {
            println!("✅ 认证成功！获得访问令牌");
            
            // 创建登录详情
            let login_details = LogOnDetails {
                username: auth_result.account_name.clone(),
                access_token: auth_result.refresh_token.unwrap_or_default(),
                refresh_token: auth_result.access_token,
                login_id: 149, // 如果有多个客户端连接，设置不同的登录ID
            };
            
            println!("📝 设置登录详情并登录...");
            
            // 登录
            client.log_on(login_details).await?;
            
            // 等待登录完成
            sleep(Duration::from_secs(3)).await;
            
            println!("   为了演示，我们将在3秒后登出...");
            sleep(Duration::from_secs(3)).await;
            
            // 登出
            client.log_off().await?;
            
            // 等待登出完成
            sleep(Duration::from_secs(1)).await;
        }
        Err(e) => {
            println!("❌ 认证失败: {}", e);
        }
    }
    
    // 断开连接
    println!("🔌 断开与Steam服务器的连接...");
    client.disconnect().await?;
    
    println!("\n✨ 基础认证示例完成！");
    
    Ok(())
}