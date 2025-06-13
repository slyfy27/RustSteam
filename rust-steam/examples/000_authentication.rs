// 示例000: 基础认证
// 
// 此示例演示如何使用现代Steam认证系统（JWT令牌）进行登录
// 包含2FA支持和完整的认证流程

use rust_steam::prelude::*;
use tokio::time::{sleep, Duration};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== JavaSteam样例000: 基础认证 ===\n");
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("使用方法: {} <用户名> <密码>", args[0]);
        println!("示例: {} myusername mypassword", args[0]);
        return Ok(());
    }
    
    let username = &args[1];
    let password = &args[2];
    
    println!("🔐 开始Steam认证流程...");
    println!("👤 用户名: {}", username);
    
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
    match client.connect().await {
        Ok(_) => println!("✅ 连接成功！"),
        Err(e) => {
            println!("❌ 连接失败: {}", e);
            return Ok(());
        }
    }
    
    // 等待连接完成
    sleep(Duration::from_secs(2)).await;
    
    // 创建认证详情
    let auth_details = AuthSessionDetails {
        username: username.to_string(),
        password: password.to_string(),
        guard_code: None,
        authenticator: Some(Arc::new(ConsoleAuthenticator)),
        device_friendly_name: "Rust Steam Client".to_string(),
        platform_type: EAuthTokenPlatformType::MobileApp,
        language: "chinese".to_string(),
        persist_login: false,
        website_id: "Client".to_string(),
        client_id: "DE45CD61".to_string(),
    };
    
    println!("🔑 执行认证会话...");
    
    // 执行认证
    match client.authenticate(auth_details).await {
        Ok(auth_result) => {
            if auth_result.success {
                println!("✅ 认证成功！获得访问令牌");
                
                // 创建登录详情
                let mut login_details = LogOnDetails::new();
                if let Some(username) = auth_result.account_name {
                    login_details.username = Some(username);
                }
                if let Some(access_token) = auth_result.access_token {
                    login_details.access_token = Some(access_token);
                }
                if let Some(refresh_token) = auth_result.refresh_token {
                    login_details.refresh_token = Some(refresh_token);
                }
                
                println!("📝 设置登录详情并登录...");
                
                // 登录
                match client.log_on(login_details).await {
                    Ok(_) => println!("✅ 登录请求已发送"),
                    Err(e) => println!("❌ 登录失败: {}", e),
                }
                
                // 等待登录完成
                sleep(Duration::from_secs(3)).await;
                
                println!("   为了演示，我们将在3秒后登出...");
                sleep(Duration::from_secs(3)).await;
                
                // 登出
                match client.log_off().await {
                    Ok(_) => println!("✅ 登出请求已发送"),
                    Err(e) => println!("❌ 登出失败: {}", e),
                }
                
                // 等待登出完成
                sleep(Duration::from_secs(1)).await;
            } else {
                println!("❌ 认证失败");
                if let Some(error) = auth_result.error_message {
                    println!("   错误: {}", error);
                }
                if auth_result.requires_2fa {
                    println!("   需要2FA验证");
                }
                if auth_result.requires_email_verification {
                    println!("   需要邮箱验证");
                }
            }
        }
        Err(e) => {
            println!("❌ 认证过程出错: {}", e);
        }
    }
    
    // 断开连接
    println!("🔌 断开与Steam服务器的连接...");
    match client.disconnect().await {
        Ok(_) => println!("✅ 断开连接成功"),
        Err(e) => println!("❌ 断开连接失败: {}", e),
    }
    
    println!("\n✨ 基础认证示例完成！");
    println!("💡 提示: 这个示例展示了Steam认证的基本流程");
    println!("   实际使用时需要提供真实的Steam凭据");
    
    Ok(())
}