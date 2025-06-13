// 示例002: Web Cookie获取
// 
// 此示例演示如何获取Steam Web Cookie (steamLoginSecure)
// 这些cookie可用于访问Steam网站API

use rust_steam::prelude::*;
use tokio::time::{sleep, Duration};
use std::env;

#[tokio::main]
async fn main() -> Result<(), SteamError> {
    println!("=== JavaSteam样例002: Web Cookie获取 ===\n");
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("使用方法: {} <用户名> <密码>", args[0]);
        return Ok(());
    }
    
    let username = &args[1];
    let password = &args[2];
    
    println!("🍪 开始Steam Web Cookie获取流程...");
    
    // 创建Steam客户端
    let mut client = SteamClient::new(ClientSettings::default());
    
    // 获取回调管理器
    let callback_manager = client.get_callback_manager();
    
    // 存储访问令牌的变量
    let mut access_token: Option<String> = None;
    let mut refresh_token: Option<String> = None;
    let mut steam_id: Option<SteamID> = None;
    
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
    });
    
    // 订阅登录成功事件
    let _logged_on_sub = callback_manager.subscribe(|callback: &LoggedOnCallback| {
        match callback.result {
            EResult::OK => {
                println!("🎉 登录成功！");
                println!("   Steam ID: {}", callback.steam_id.render());
                println!("   账户名: {}", callback.account_name);
                
                // 在实际实现中，这里会从登录回调中获取令牌信息
                println!("🍪 开始生成Web Cookie...");
            }
            _ => {
                println!("❌ 登录失败: {:?}", callback.result);
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
        persistent_session: false, // 不持久化会话以获取访问令牌
        guard_data: None,
        authenticator: Some(Arc::new(ConsoleAuthenticator)),
        device_friendly_name: "Rust Steam WebCookie Client".to_string(),
    };
    
    println!("🔑 执行认证会话...");
    
    // 执行认证
    match client.authenticate(auth_details).await {
        Ok(auth_result) => {
            println!("✅ 认证成功！获得访问令牌");
            
            // 保存令牌信息
            access_token = Some(auth_result.access_token.clone());
            refresh_token = Some(auth_result.refresh_token.clone());
            
            // 创建登录详情
            let login_details = LogOnDetails {
                username: auth_result.account_name.clone(),
                access_token: Some(auth_result.refresh_token.clone()),
                login_id: 149,
                client_os_type: 16,
                should_remember_password: false,
                machine_name: "Rust Steam WebCookie Client".to_string(),
                ..Default::default()
            };
            
            println!("📝 设置登录详情并登录...");
            
            // 登录
            client.log_on(login_details).await?;
            
            // 等待登录完成
            sleep(Duration::from_secs(2)).await;
            
            // 模拟获取Steam ID（在实际实现中从登录回调获取）
            steam_id = Some(SteamID::new(76561198000000000));
            
            if let (Some(access_token), Some(steam_id)) = (&access_token, &steam_id) {
                // 生成steamLoginSecure cookie
                let steam_login_secure = format!("{}||{}", steam_id.id, access_token);
                
                println!("\n🍪 === Steam Web Cookie 信息 ===");
                println!("steamLoginSecure: {}", steam_login_secure);
                println!();
                println!("📋 使用说明:");
                println!("1. 复制上面的steamLoginSecure值");
                println!("2. 在浏览器中访问Steam域名网站");
                println!("3. 在开发者工具中设置Cookie:");
                println!("   名称: steamLoginSecure");
                println!("   值: {}", steam_login_secure);
                println!("   域: .steamcommunity.com 或 .steampowered.com");
                println!("4. 刷新页面即可以已登录状态访问Steam网站");
                println!();
                println!("⚠️  注意事项:");
                println!("- 访问令牌有效期约24小时");
                println!("- 令牌过期后需要重新获取");
                println!("- 请妥善保管令牌，避免泄露");
                
                // 演示令牌续期功能
                println!("\n🔄 演示令牌续期功能...");
                
                if let Some(refresh_token) = &refresh_token {
                    let new_access_token = renew_access_token(steam_id, refresh_token).await?;
                    let new_steam_login_secure = format!("{}||{}", steam_id.id, new_access_token);
                    
                    println!("✅ 令牌续期成功!");
                    println!("新的steamLoginSecure: {}", new_steam_login_secure);
                    println!("💡 实际应用中，应该设置定时器在令牌过期前自动续期");
                }
            }
            
            println!("\n   为了演示，我们将在3秒后登出...");
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
    
    println!("\n✨ Web Cookie获取示例完成！");
    
    Ok(())
}

/// 模拟令牌续期功能
async fn renew_access_token(steam_id: &SteamID, refresh_token: &str) -> Result<String, SteamError> {
    println!("🔄 正在续期访问令牌...");
    
    // 模拟API调用延迟
    sleep(Duration::from_millis(500)).await;
    
    // 在实际实现中，这里会调用Steam的token续期API
    // 现在我们返回一个模拟的新令牌
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let new_token = format!("renewed_access_token_{}_{}_{}", steam_id.id, timestamp, &refresh_token[..8]);
    
    Ok(new_token)
}

/// 解析JWT令牌并显示内容（演示用）
#[allow(dead_code)]
fn parse_jwt_token(token: &str, token_name: &str) {
    println!("\n🔍 === {} 令牌信息 ===", token_name);
    
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        println!("❌ 无效的JWT令牌格式");
        return;
    }
    
    // 在实际实现中，这里会解码JWT payload
    println!("令牌类型: JWT");
    println!("头部: {}", parts[0]);
    println!("载荷: {} (已编码)", parts[1]);
    println!("签名: {} (已编码)", parts[2]);
    println!("💡 实际应用中可以解码载荷查看过期时间和作用域");
}