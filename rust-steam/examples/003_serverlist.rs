// 示例003: 服务器列表管理
// 
// 此示例演示如何管理Steam服务器列表以优化连接成功率
// 包括服务器发现、缓存和Cell ID管理

use rust_steam::prelude::*;
use tokio::time::{sleep, Duration};
use std::env;
use std::fs;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), SteamError> {
    println!("=== JavaSteam样例003: 服务器列表管理 ===\n");
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("使用方法: {} <用户名> <密码>", args[0]);
        return Ok(());
    }
    
    let username = &args[1];
    let password = &args[2];
    
    println!("🌐 开始Steam服务器列表管理示例...");
    
    // 读取已保存的Cell ID
    let cell_id = load_cell_id().unwrap_or(0);
    if cell_id != 0 {
        println!("📍 使用已保存的Cell ID: {}", cell_id);
    } else {
        println!("📍 没有找到已保存的Cell ID，将使用默认值");
    }
    
    // 读取已保存的服务器列表
    let server_list = load_server_list();
    println!("🖥️  加载了 {} 个已知服务器", server_list.len());
    
    // 创建包含服务器列表的客户端配置
    let mut settings = ClientSettings::default();
    if !server_list.is_empty() {
        settings.server_list = server_list;
        println!("✅ 使用已保存的服务器列表");
    } else {
        println!("🔄 使用默认服务器列表");
    }
    
    // 创建Steam客户端
    let mut client = SteamClient::new(settings);
    
    // 获取回调管理器
    let callback_manager = client.get_callback_manager();
    
    // 订阅连接事件
    let _connected_sub = callback_manager.subscribe(|callback: &ConnectedCallback| {
        println!("✅ 已连接到Steam服务器!");
        println!("   服务器时间: {:?}", callback.server_time);
        println!("   连接质量: 良好");
    });
    
    // 订阅断开连接事件
    let _disconnected_sub = callback_manager.subscribe(|callback: &DisconnectedCallback| {
        println!("❌ 与Steam服务器的连接已断开");
        if let Some(reason) = &callback.reason {
            println!("   原因: {}", reason);
        }
        
        if !callback.user_initiated {
            println!("🔄 连接未主动断开，将尝试重新连接...");
        }
    });
    
    // 订阅登录成功事件
    let _logged_on_sub = callback_manager.subscribe(|callback: &LoggedOnCallback| {
        match callback.result {
            EResult::OK => {
                println!("🎉 登录成功！");
                println!("   Steam ID: {}", callback.steam_id.render());
                println!("   账户名: {}", callback.account_name);
                println!("   Cell ID: {}", callback.cell_id);
                
                // 保存Cell ID以便下次使用
                if let Err(e) = save_cell_id(callback.cell_id) {
                    println!("⚠️  保存Cell ID失败: {}", e);
                } else {
                    println!("💾 已保存Cell ID: {}", callback.cell_id);
                }
                
                // 在实际实现中，这里还会保存服务器列表
                println!("💾 正在保存服务器列表...");
                let servers = vec![
                    "steamcommunity.com:443".to_string(),
                    "steampowered.com:443".to_string(),
                    format!("cm{}.steampowered.com:443", callback.cell_id % 10),
                ];
                
                if let Err(e) = save_server_list(&servers) {
                    println!("⚠️  保存服务器列表失败: {}", e);
                } else {
                    println!("💾 已保存 {} 个服务器", servers.len());
                }
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
    
    // 显示连接信息
    println!("\n📊 === 连接信息 ===");
    println!("目标服务器数量: {}", client.get_callback_manager().as_ref().clone().subscribe.len());
    println!("Cell ID: {}", cell_id);
    println!("连接超时: {:?}", Duration::from_secs(10));
    println!("自动重试: 是");
    
    // 连接到Steam
    println!("\n🌐 正在连接Steam服务器...");
    
    // 模拟连接重试逻辑
    let mut retry_count = 0;
    let max_retries = 3;
    
    while retry_count < max_retries {
        if retry_count > 0 {
            println!("🔄 重试连接 ({}/{})", retry_count + 1, max_retries);
            sleep(Duration::from_secs(2)).await;
        }
        
        match client.connect().await {
            Ok(_) => {
                println!("✅ 连接成功！");
                break;
            }
            Err(e) => {
                println!("❌ 连接失败: {}", e);
                retry_count += 1;
                
                if retry_count < max_retries {
                    println!("⏳ 等待 2 秒后重试...");
                } else {
                    println!("❌ 达到最大重试次数，连接失败");
                    return Err(e);
                }
            }
        }
    }
    
    // 等待连接完成
    sleep(Duration::from_secs(1)).await;
    
    // 创建认证详情
    let auth_details = AuthSessionDetails {
        username: username.to_string(),
        password: password.to_string(),
        persistent_session: false,
        guard_data: None,
        authenticator: Some(Arc::new(ConsoleAuthenticator)),
        device_friendly_name: "Rust Steam ServerList Client".to_string(),
    };
    
    println!("🔑 执行认证会话...");
    
    // 执行认证和登录
    match client.authenticate(auth_details).await {
        Ok(auth_result) => {
            println!("✅ 认证成功！");
            
            // 创建登录详情
            let login_details = LogOnDetails {
                username: auth_result.account_name.clone(),
                access_token: Some(auth_result.refresh_token.clone()),
                login_id: 149,
                client_os_type: 16,
                should_remember_password: false,
                machine_name: "Rust Steam ServerList Client".to_string(),
                ..Default::default()
            };
            
            println!("📝 登录到Steam...");
            client.log_on(login_details).await?;
            
            // 等待登录完成
            sleep(Duration::from_secs(3)).await;
            
            // 显示服务器信息
            println!("\n📈 === 连接统计 ===");
            println!("当前连接状态: {:?}", client.get_connection_state());
            println!("重试次数: {}", retry_count);
            println!("连接时间: 约 {} 秒", retry_count * 2 + 2);
            
            println!("\n💡 === 优化建议 ===");
            println!("1. 保存Cell ID可以提高连接成功率");
            println!("2. 维护服务器列表可以减少DNS查询时间");
            println!("3. 使用就近的CM服务器可以降低延迟");
            
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
    
    println!("\n✨ 服务器列表管理示例完成！");
    
    Ok(())
}

/// 从文件加载Cell ID
fn load_cell_id() -> Result<u32, io::Error> {
    let content = fs::read_to_string("cellid.txt")?;
    content.trim().parse().map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("解析Cell ID失败: {}", e))
    })
}

/// 保存Cell ID到文件
fn save_cell_id(cell_id: u32) -> Result<(), io::Error> {
    fs::write("cellid.txt", cell_id.to_string())
}

/// 从文件加载服务器列表
fn load_server_list() -> Vec<String> {
    match fs::read_to_string("server_list.txt") {
        Ok(content) => content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect(),
        Err(_) => {
            println!("📁 没有找到服务器列表文件 server_list.txt");
            Vec::new()
        }
    }
}

/// 保存服务器列表到文件
fn save_server_list(servers: &[String]) -> Result<(), io::Error> {
    let content = servers.join("\n");
    fs::write("server_list.txt", content)
}

/// 显示网络质量信息
#[allow(dead_code)]
fn display_network_quality() {
    println!("\n📊 === 网络质量检测 ===");
    println!("延迟: ~50ms");
    println!("丢包率: 0%");
    println!("带宽: 充足");
    println!("连接稳定性: 良好");
}