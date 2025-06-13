// 示例001: QR码认证
// 
// 此示例演示如何使用QR码进行Steam认证
// 用户需要使用Steam手机应用扫描QR码来完成登录

use rust_steam::prelude::*;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), SteamError> {
    println!("=== JavaSteam样例001: QR码认证 ===\n");
    
    println!("🔐 开始Steam QR码认证流程...");
    println!("注意：此认证方式无需用户名和密码，仅需使用Steam手机应用扫描QR码");
    
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
    });
    
    // 订阅登录成功事件
    let _logged_on_sub = callback_manager.subscribe(|callback: &LoggedOnCallback| {
        match callback.result {
            EResult::OK => {
                println!("🎉 QR码登录成功！");
                println!("   Steam ID: {}", callback.steam_id.render());
                println!("   账户名: {}", callback.account_name);
                println!("   现在可以在Steam上执行各种操作");
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
    
    println!("📱 准备生成QR码...");
    
    // 生成模拟的QR码认证URL
    let challenge_url = generate_mock_qr_challenge_url();
    
    println!("\n=== QR码认证 ===");
    println!("挑战URL: {}", challenge_url);
    println!();
    
    // 显示简化的QR码（用ASCII字符模拟）
    display_mock_qr_code(&challenge_url);
    
    println!("\n📱 请使用Steam手机应用扫描上方QR码进行认证");
    println!("⏳ 正在等待认证确认...");
    
    // 模拟等待QR码扫描和确认过程
    for i in 1..=30 {
        print!(".");
        if i % 10 == 0 {
            print!(" {}s", i);
        }
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        sleep(Duration::from_secs(1)).await;
        
        // 模拟在第15秒时认证成功
        if i == 15 {
            println!("\n✅ QR码认证成功！获得访问令牌");
            break;
        }
    }
    
    // 模拟认证成功后的登录流程
    let mock_auth_result = AuthPollResult {
        account_name: "QR_User_12345".to_string(),
        access_token: "mock_access_token_qr".to_string(),
        refresh_token: "mock_refresh_token_qr".to_string(),
        requires_2fa: false,
        new_guard_data: None,
    };
    
    // 创建登录详情
    let login_details = LogOnDetails {
        username: mock_auth_result.account_name.clone(),
        access_token: Some(mock_auth_result.refresh_token.clone()),
        login_id: 149,
        client_os_type: 16,
        should_remember_password: false,
        machine_name: "Rust Steam QR Client".to_string(),
        ..Default::default()
    };
    
    println!("📝 使用QR码认证结果登录...");
    
    // 登录
    client.log_on(login_details).await?;
    
    // 等待登录完成
    sleep(Duration::from_secs(3)).await;
    
    println!("   为了演示，我们将在5秒后登出...");
    sleep(Duration::from_secs(5)).await;
    
    // 登出
    client.log_off().await?;
    
    // 等待登出完成
    sleep(Duration::from_secs(1)).await;
    
    // 断开连接
    println!("🔌 断开与Steam服务器的连接...");
    client.disconnect().await?;
    
    println!("\n✨ QR码认证示例完成！");
    
    Ok(())
}

/// 生成模拟的QR码挑战URL
fn generate_mock_qr_challenge_url() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    format!("https://s.team/q/{}/mock_challenge_id", timestamp)
}

/// 显示模拟的QR码（用ASCII字符表示）
fn display_mock_qr_code(url: &str) {
    println!("┌─────────────────────────┐");
    println!("│ ██ ▄▄▄▄▄▄▄ ██ ▄▄▄▄▄▄▄ │");
    println!("│ ██ █     █ ██ █     █ │");
    println!("│ ██ █ ███ █ ██ █ ███ █ │");
    println!("│ ██ █ ███ █ ██ █ ███ █ │");
    println!("│ ██ █ ███ █ ██ █ ███ █ │");
    println!("│ ██ █     █ ██ █     █ │");
    println!("│ ██ ▀▀▀▀▀▀▀ ██ ▀▀▀▀▀▀▀ │");
    println!("│ ▄▄▄▄▄ ▄ ▄ ▄▄▄▄▄ ▄ ▄ ▄ │");
    println!("│ █▀▀▀█ ▀█▄ █▀▀▀█ ▀█▄ █ │");
    println!("│ ▀ ▄ ▀ ██▄ ▀ ▄ ▀ ██▄ ▀ │");
    println!("│ ██ ▄▄▄▄▄▄▄ ██ ▄▄▄▄▄▄▄ │");
    println!("│ ██ █     █ ██ █     █ │");
    println!("│ ██ █ ███ █ ██ █ ███ █ │");
    println!("│ ██ █ ███ █ ██ █ ███ █ │");
    println!("│ ██ █ ███ █ ██ █ ███ █ │");
    println!("│ ██ █     █ ██ █     █ │");
    println!("│ ██ ▀▀▀▀▀▀▀ ██ ▀▀▀▀▀▀▀ │");
    println!("└─────────────────────────┘");
    println!("QR码内容: {}", url);
}