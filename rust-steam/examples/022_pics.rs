// 示例022: PICS (Product Info and Changes System)
// 
// 此示例演示如何使用Steam的PICS系统
// 获取应用程序和包的详细信息和变更

use rust_steam::prelude::*;
use tokio::time::{sleep, Duration};
use std::env;
use std::collections::HashMap;

/// PICS产品信息
#[derive(Debug, Clone)]
struct PICSProductInfo {
    changelist: u32,
    missing_token: bool,
    sha: String,
    size: u32,
    apps: HashMap<u32, AppInfo>,
    packages: HashMap<u32, PackageInfo>,
}

/// 应用信息
#[derive(Debug, Clone)]
struct AppInfo {
    appid: u32,
    name: String,
    type_: String,
    developer: String,
    publisher: String,
    release_date: String,
    description: String,
    genres: Vec<String>,
    categories: Vec<String>,
    supported_languages: Vec<String>,
    requirements: SystemRequirements,
    depot_ids: Vec<u32>,
}

/// 包信息
#[derive(Debug, Clone)]
struct PackageInfo {
    packageid: u32,
    name: String,
    price: Option<u32>,
    currency: String,
    appids: Vec<u32>,
    depot_ids: Vec<u32>,
    billing_type: String,
    license_type: String,
    status: String,
}

/// 系统需求
#[derive(Debug, Clone)]
struct SystemRequirements {
    minimum: String,
    recommended: String,
}

/// PICS变更信息
#[derive(Debug, Clone)]
struct PICSChanges {
    current_changelist: u32,
    since_changelist: u32,
    app_changes: HashMap<u32, PICSChangeData>,
    package_changes: HashMap<u32, PICSChangeData>,
}

/// PICS变更数据
#[derive(Debug, Clone)]
struct PICSChangeData {
    changelist: u32,
    needs_token: bool,
}

#[tokio::main]
async fn main() -> Result<(), SteamError> {
    println!("=== JavaSteam样例022: PICS系统 ===\n");
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("使用方法: {} <用户名> <密码>", args[0]);
        return Ok(());
    }
    
    let username = &args[1];
    let password = &args[2];
    
    println!("📦 开始Steam PICS系统示例...");
    println!("   PICS (Product Info and Changes System) 是Steam的产品信息系统");
    
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
                println!("🎉 登录成功！");
                println!("   Steam ID: {}", callback.steam_id.render());
                println!("   账户名: {}", callback.account_name);
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
    println!("\n🌐 正在连接Steam服务器...");
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
        device_friendly_name: "Rust Steam PICS Client".to_string(),
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
                machine_name: "Rust Steam PICS Client".to_string(),
                ..Default::default()
            };
            
            println!("📝 登录到Steam...");
            client.log_on(login_details).await?;
            
            // 等待登录完成
            sleep(Duration::from_secs(2)).await;
            
            // 演示PICS功能
            println!("\n📦 === PICS系统功能演示 ===");
            
            // 1. 获取产品信息
            demonstrate_product_info().await?;
            
            // 2. 监听产品变更
            demonstrate_change_monitoring().await?;
            
            // 3. 解析产品数据
            demonstrate_data_parsing().await?;
            
            // 4. 批量查询
            demonstrate_batch_queries().await?;
            
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
    
    println!("\n✨ PICS系统示例完成！");
    
    Ok(())
}

/// 演示产品信息获取
async fn demonstrate_product_info() -> Result<(), SteamError> {
    println!("\n1️⃣ 获取产品信息");
    
    // 获取知名游戏的信息 (Helldivers 2)
    let app_id = 553850;
    println!("📦 正在获取应用 {} 的详细信息...", app_id);
    
    sleep(Duration::from_millis(800)).await;
    
    // 模拟获取的产品信息
    let app_info = AppInfo {
        appid: app_id,
        name: "HELLDIVERS™ 2".to_string(),
        type_: "game".to_string(),
        developer: "Arrowhead Game Studios".to_string(),
        publisher: "Sony Interactive Entertainment".to_string(),
        release_date: "Feb 8, 2024".to_string(),
        description: "The Galaxy's Last Line of Offence. Enlist in the Helldivers and join the fight for freedom across a hostile galaxy in a fast, frantic, and ferocious third-person shooter.".to_string(),
        genres: vec!["Action".to_string(), "Indie".to_string(), "Strategy".to_string()],
        categories: vec!["Multi-player".to_string(), "Online Co-op".to_string(), "Cross-Platform Multiplayer".to_string()],
        supported_languages: vec!["English".to_string(), "French".to_string(), "German".to_string(), "Spanish".to_string(), "Japanese".to_string(), "Korean".to_string(), "Simplified Chinese".to_string()],
        requirements: SystemRequirements {
            minimum: "OS: Windows 10. Processor: Intel Core i7-4790K or AMD Ryzen 5 1500X. Memory: 8 GB RAM. Graphics: NVIDIA GeForce GTX 1050 Ti or AMD Radeon RX 560. DirectX: Version 12. Network: Broadband Internet connection. Storage: 100 GB available space.".to_string(),
            recommended: "OS: Windows 10. Processor: Intel Core i7-9700K or AMD Ryzen 7 2700X. Memory: 16 GB RAM. Graphics: NVIDIA GeForce RTX 2060 or AMD Radeon RX 6600XT. DirectX: Version 12. Network: Broadband Internet connection. Storage: 100 GB available space.".to_string(),
        },
        depot_ids: vec![553851, 553852, 553853],
    };
    
    display_app_info(&app_info);
    
    Ok(())
}

/// 演示变更监听
async fn demonstrate_change_monitoring() -> Result<(), SteamError> {
    println!("\n2️⃣ 监听产品变更");
    
    println!("📡 正在检查产品变更...");
    sleep(Duration::from_millis(600)).await;
    
    // 模拟PICS变更数据
    let changes = PICSChanges {
        current_changelist: 12345678,
        since_changelist: 12345000,
        app_changes: {
            let mut map = HashMap::new();
            map.insert(553850, PICSChangeData { changelist: 12345100, needs_token: false });
            map.insert(440, PICSChangeData { changelist: 12345200, needs_token: false });
            map.insert(730, PICSChangeData { changelist: 12345300, needs_token: true });
            map
        },
        package_changes: {
            let mut map = HashMap::new();
            map.insert(12345, PICSChangeData { changelist: 12345150, needs_token: false });
            map.insert(67890, PICSChangeData { changelist: 12345250, needs_token: true });
            map
        },
    };
    
    display_changes(&changes);
    
    Ok(())
}

/// 演示数据解析
async fn demonstrate_data_parsing() -> Result<(), SteamError> {
    println!("\n3️⃣ 解析产品数据");
    
    println!("🔍 正在解析VDF数据格式...");
    sleep(Duration::from_millis(400)).await;
    
    // 模拟解析VDF数据
    println!("✅ 成功解析产品配置");
    println!("   - 游戏元数据已提取");
    println!("   - DLC信息已识别");
    println!("   - 系统需求已解析");
    println!("   - 成就列表已获取");
    
    // 演示一些解析出的键值对
    println!("\n📋 解析出的关键数据:");
    println!("  common/name: \"HELLDIVERS™ 2\"");
    println!("  common/type: \"Game\"");
    println!("  common/oslist: \"windows\"");
    println!("  common/steam_release_date: \"1707350400\"");
    println!("  common/controller_support: \"full\"");
    println!("  config/launch/0/executable: \"helldivers2.exe\"");
    println!("  config/launch/0/type: \"default\"");
    
    Ok(())
}

/// 演示批量查询
async fn demonstrate_batch_queries() -> Result<(), SteamError> {
    println!("\n4️⃣ 批量查询功能");
    
    let app_ids = vec![553850, 440, 730, 570, 578080]; // Helldivers 2, TF2, CS:GO, Dota 2, PUBG
    
    println!("📦 正在批量获取 {} 个应用的信息...", app_ids.len());
    sleep(Duration::from_millis(1000)).await;
    
    // 模拟批量查询结果
    for (i, &app_id) in app_ids.iter().enumerate() {
        let game_name = match app_id {
            553850 => "HELLDIVERS™ 2",
            440 => "Team Fortress 2",
            730 => "Counter-Strike: Global Offensive",
            570 => "Dota 2",
            578080 => "PLAYERUNKNOWN'S BATTLEGROUNDS",
            _ => "Unknown Game",
        };
        
        println!("  ✅ App {}: {} (已获取)", app_id, game_name);
        sleep(Duration::from_millis(200)).await;
    }
    
    println!("\n📊 批量查询统计:");
    println!("  总应用数: {}", app_ids.len());
    println!("  成功获取: {}", app_ids.len());
    println!("  失败数量: 0");
    println!("  平均响应时间: 200ms");
    
    println!("\n💡 === PICS系统优势 ===");
    println!("✅ 高效的产品信息获取");
    println!("✅ 实时变更通知");
    println!("✅ 支持批量查询");
    println!("✅ 完整的元数据访问");
    println!("✅ VDF格式数据解析");
    println!("✅ 缓存友好的设计");
    
    Ok(())
}

/// 显示应用信息
fn display_app_info(app_info: &AppInfo) {
    println!("📦 === 应用详细信息 ===");
    println!("ID: {}", app_info.appid);
    println!("名称: {}", app_info.name);
    println!("类型: {}", app_info.type_);
    println!("开发商: {}", app_info.developer);
    println!("发行商: {}", app_info.publisher);
    println!("发布日期: {}", app_info.release_date);
    
    println!("\n📝 描述:");
    // 限制描述长度以便显示
    let desc = if app_info.description.len() > 100 {
        format!("{}...", &app_info.description[..100])
    } else {
        app_info.description.clone()
    };
    println!("  {}", desc);
    
    println!("\n🎮 类型:");
    for genre in &app_info.genres {
        println!("  - {}", genre);
    }
    
    println!("\n🏷️  分类:");
    for category in &app_info.categories {
        println!("  - {}", category);
    }
    
    println!("\n🌍 支持语言:");
    for (i, lang) in app_info.supported_languages.iter().enumerate() {
        if i < 5 { // 只显示前5种语言
            println!("  - {}", lang);
        } else if i == 5 {
            println!("  - ... 还有 {} 种语言", app_info.supported_languages.len() - 5);
            break;
        }
    }
    
    println!("\n💾 系统需求:");
    println!("  最低配置: {}", &app_info.requirements.minimum[..100.min(app_info.requirements.minimum.len())]);
    if app_info.requirements.minimum.len() > 100 {
        println!("    ...");
    }
    
    println!("\n📦 Depot IDs:");
    for depot_id in &app_info.depot_ids {
        println!("  - {}", depot_id);
    }
}

/// 显示变更信息
fn display_changes(changes: &PICSChanges) {
    println!("📊 === PICS变更信息 ===");
    println!("当前变更列表: {}", changes.current_changelist);
    println!("起始变更列表: {}", changes.since_changelist);
    
    if !changes.app_changes.is_empty() {
        println!("\n🎮 应用变更 ({} 个):", changes.app_changes.len());
        for (app_id, change_data) in &changes.app_changes {
            let token_status = if change_data.needs_token { "🔒 需要令牌" } else { "🔓 公开" };
            println!("  App {}: 变更列表 {} - {}", app_id, change_data.changelist, token_status);
        }
    }
    
    if !changes.package_changes.is_empty() {
        println!("\n📦 包变更 ({} 个):", changes.package_changes.len());
        for (package_id, change_data) in &changes.package_changes {
            let token_status = if change_data.needs_token { "🔒 需要令牌" } else { "🔓 公开" };
            println!("  Package {}: 变更列表 {} - {}", package_id, change_data.changelist, token_status);
        }
    }
    
    println!("\n💡 提示:");
    println!("  - 🔓 公开变更可以直接获取");
    println!("  - 🔒 需要令牌的变更需要特殊权限");
    println!("  - 变更列表号码越大表示越新的变更");
}