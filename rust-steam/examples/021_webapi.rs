// 示例021: Steam Web API
// 
// 此示例演示如何使用Steam Web API获取游戏信息
// 包括新闻、统计数据、成就等信息

use rust_steam::prelude::*;
use tokio::time::{sleep, Duration};
use std::env;
use std::collections::HashMap;

/// Steam Web API 客户端
struct SteamWebAPI {
    api_key: Option<String>,
    base_url: String,
}

impl SteamWebAPI {
    fn new() -> Self {
        Self {
            api_key: None,
            base_url: "https://api.steampowered.com".to_string(),
        }
    }
    
    fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
    
    /// 获取游戏新闻
    async fn get_news_for_app(&self, app_id: u32) -> Result<GameNews, SteamError> {
        println!("📰 正在获取游戏 {} 的新闻...", app_id);
        
        // 模拟API调用延迟
        sleep(Duration::from_millis(800)).await;
        
        // 模拟返回新闻数据
        Ok(GameNews {
            app_id,
            newsitems: vec![
                NewsItem {
                    gid: "123456789".to_string(),
                    title: "重大更新：新增多人模式".to_string(),
                    url: format!("https://steamcommunity.com/games/{}/announcements/detail/123456789", app_id),
                    is_external_url: false,
                    author: "开发团队".to_string(),
                    contents: "我们很高兴地宣布新的多人模式现已上线！".to_string(),
                    feedlabel: "产品更新".to_string(),
                    date: 1640995200, // 2022-01-01
                    feedname: "steam_community_announcements".to_string(),
                },
                NewsItem {
                    gid: "123456790".to_string(),
                    title: "节日活动开始".to_string(),
                    url: format!("https://steamcommunity.com/games/{}/announcements/detail/123456790", app_id),
                    is_external_url: false,
                    author: "社区管理员".to_string(),
                    contents: "特殊节日活动现已开始，快来参与吧！".to_string(),
                    feedlabel: "社区活动".to_string(),
                    date: 1640908800, // 2021-12-31
                    feedname: "steam_community_announcements".to_string(),
                },
            ],
        })
    }
    
    /// 获取玩家统计
    async fn get_player_stats(&self, steam_id: u64, app_id: u32) -> Result<PlayerStats, SteamError> {
        println!("📊 正在获取玩家 {} 在游戏 {} 的统计数据...", steam_id, app_id);
        
        // 模拟API调用延迟
        sleep(Duration::from_millis(600)).await;
        
        // 模拟返回统计数据
        Ok(PlayerStats {
            steam_id,
            game_name: match app_id {
                440 => "Team Fortress 2".to_string(),
                730 => "Counter-Strike: Global Offensive".to_string(),
                570 => "Dota 2".to_string(),
                _ => format!("Game {}", app_id),
            },
            achievements: vec![
                Achievement {
                    name: "first_kill".to_string(),
                    display_name: "首次击杀".to_string(),
                    description: "获得你的第一次击杀".to_string(),
                    achieved: true,
                    unlock_time: Some(1640995200),
                },
                Achievement {
                    name: "veteran_player".to_string(),
                    display_name: "资深玩家".to_string(),
                    description: "游戏时间达到100小时".to_string(),
                    achieved: false,
                    unlock_time: None,
                },
            ],
            stats: vec![
                Stat {
                    name: "total_kills".to_string(),
                    display_name: "总击杀数".to_string(),
                    value: 1337,
                },
                Stat {
                    name: "total_deaths".to_string(),
                    display_name: "总死亡数".to_string(),
                    value: 420,
                },
                Stat {
                    name: "total_playtime".to_string(),
                    display_name: "总游戏时间(分钟)".to_string(),
                    value: 4567,
                },
            ],
        })
    }
    
    /// 获取游戏模式列表
    async fn get_servers_at_address(&self, addr: &str) -> Result<Vec<GameServer>, SteamError> {
        println!("🖥️  正在查询地址 {} 的游戏服务器...", addr);
        
        // 模拟API调用延迟
        sleep(Duration::from_millis(400)).await;
        
        // 模拟返回服务器列表
        Ok(vec![
            GameServer {
                addr: addr.to_string(),
                gmsindex: 1,
                appid: 440,
                gamedir: "tf".to_string(),
                region: 3, // Europe
                secure: true,
                lan: false,
                gameport: 27015,
                specport: 27016,
            },
            GameServer {
                addr: addr.to_string(),
                gmsindex: 2,
                appid: 730,
                gamedir: "csgo".to_string(),
                region: 3,
                secure: true,
                lan: false,
                gameport: 27015,
                specport: 27016,
            },
        ])
    }
}

/// 游戏新闻结构
#[derive(Debug)]
struct GameNews {
    app_id: u32,
    newsitems: Vec<NewsItem>,
}

/// 新闻条目
#[derive(Debug)]
struct NewsItem {
    gid: String,
    title: String,
    url: String,
    is_external_url: bool,
    author: String,
    contents: String,
    feedlabel: String,
    date: i64,
    feedname: String,
}

/// 玩家统计
#[derive(Debug)]
struct PlayerStats {
    steam_id: u64,
    game_name: String,
    achievements: Vec<Achievement>,
    stats: Vec<Stat>,
}

/// 成就
#[derive(Debug)]
struct Achievement {
    name: String,
    display_name: String,
    description: String,
    achieved: bool,
    unlock_time: Option<i64>,
}

/// 统计数据
#[derive(Debug)]
struct Stat {
    name: String,
    display_name: String,
    value: i64,
}

/// 游戏服务器
#[derive(Debug)]
struct GameServer {
    addr: String,
    gmsindex: u32,
    appid: u32,
    gamedir: String,
    region: u32,
    secure: bool,
    lan: bool,
    gameport: u16,
    specport: u16,
}

#[tokio::main]
async fn main() -> Result<(), SteamError> {
    println!("=== JavaSteam样例021: Steam Web API ===\n");
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("使用方法: {} <用户名> <密码> [API密钥]", args[0]);
        return Ok(());
    }
    
    let username = &args[1];
    let password = &args[2];
    let api_key = args.get(3).cloned();
    
    println!("🌐 开始Steam Web API示例...");
    
    // 创建Steam客户端
    let mut client = SteamClient::new(ClientSettings::default());
    
    // 获取回调管理器
    let callback_manager = client.get_callback_manager();
    
    // 创建Web API客户端
    let web_api = if let Some(key) = api_key {
        println!("🔑 使用提供的API密钥");
        SteamWebAPI::new().with_api_key(key)
    } else {
        println!("ℹ️  未提供API密钥，将使用公共API");
        SteamWebAPI::new()
    };
    
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
        device_friendly_name: "Rust Steam WebAPI Client".to_string(),
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
                machine_name: "Rust Steam WebAPI Client".to_string(),
                ..Default::default()
            };
            
            println!("📝 登录到Steam...");
            client.log_on(login_details).await?;
            
            // 等待登录完成
            sleep(Duration::from_secs(2)).await;
            
            // 演示Web API功能
            println!("\n📡 === Steam Web API 功能演示 ===");
            
            // 1. 获取游戏新闻
            println!("\n1️⃣ 获取Team Fortress 2新闻");
            match web_api.get_news_for_app(440).await {
                Ok(news) => {
                    display_game_news(&news);
                }
                Err(e) => {
                    println!("❌ 获取新闻失败: {}", e);
                }
            }
            
            // 2. 获取玩家统计
            println!("\n2️⃣ 获取玩家统计数据");
            let mock_steam_id = 76561198000000000;
            match web_api.get_player_stats(mock_steam_id, 440).await {
                Ok(stats) => {
                    display_player_stats(&stats);
                }
                Err(e) => {
                    println!("❌ 获取统计数据失败: {}", e);
                }
            }
            
            // 3. 查询游戏服务器
            println!("\n3️⃣ 查询游戏服务器");
            match web_api.get_servers_at_address("192.168.1.100").await {
                Ok(servers) => {
                    display_game_servers(&servers);
                }
                Err(e) => {
                    println!("❌ 查询服务器失败: {}", e);
                }
            }
            
            // 4. 演示其他API功能
            demonstrate_additional_apis().await;
            
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
    
    println!("\n✨ Steam Web API示例完成！");
    
    Ok(())
}

/// 显示游戏新闻
fn display_game_news(news: &GameNews) {
    println!("📰 === 游戏新闻 (App ID: {}) ===", news.app_id);
    
    for (i, item) in news.newsitems.iter().enumerate() {
        println!("\n📄 新闻 #{}", i + 1);
        println!("  标题: {}", item.title);
        println!("  作者: {}", item.author);
        println!("  分类: {}", item.feedlabel);
        println!("  日期: {}", format_timestamp(item.date));
        println!("  链接: {}", item.url);
        println!("  内容: {}", item.contents);
    }
}

/// 显示玩家统计
fn display_player_stats(stats: &PlayerStats) {
    println!("📊 === 玩家统计 ({}) ===", stats.game_name);
    println!("Steam ID: {}", stats.steam_id);
    
    println!("\n🏆 成就进度:");
    let achieved_count = stats.achievements.iter().filter(|a| a.achieved).count();
    println!("  已解锁: {} / {} ({:.1}%)", 
             achieved_count, 
             stats.achievements.len(),
             (achieved_count as f32 / stats.achievements.len() as f32) * 100.0);
    
    for achievement in &stats.achievements {
        let status = if achievement.achieved { "✅" } else { "❌" };
        println!("  {} {} - {}", status, achievement.display_name, achievement.description);
        if let Some(unlock_time) = achievement.unlock_time {
            println!("     解锁时间: {}", format_timestamp(unlock_time));
        }
    }
    
    println!("\n📈 游戏统计:");
    for stat in &stats.stats {
        println!("  {}: {}", stat.display_name, stat.value);
    }
}

/// 显示游戏服务器
fn display_game_servers(servers: &[GameServer]) {
    println!("🖥️  === 游戏服务器列表 ===");
    
    for (i, server) in servers.iter().enumerate() {
        println!("\n🖥️  服务器 #{}", i + 1);
        println!("  地址: {}", server.addr);
        println!("  游戏: App ID {}", server.appid);
        println!("  游戏目录: {}", server.gamedir);
        println!("  地区: {}", format_region(server.region));
        println!("  游戏端口: {}", server.gameport);
        println!("  观战端口: {}", server.specport);
        println!("  安全模式: {}", if server.secure { "是" } else { "否" });
        println!("  局域网: {}", if server.lan { "是" } else { "否" });
    }
}

/// 演示其他API功能
async fn demonstrate_additional_apis() {
    println!("\n4️⃣ 其他API功能演示");
    
    // 模拟获取用户库存
    println!("📦 获取用户库存...");
    sleep(Duration::from_millis(300)).await;
    println!("✅ 找到 23 个物品");
    
    // 模拟获取市场价格
    println!("💰 查询市场价格...");
    sleep(Duration::from_millis(400)).await;
    println!("✅ AK-47 | 红线 (磨损) - ¥89.00");
    
    // 模拟获取用户成就
    println!("🎯 检查全局成就统计...");
    sleep(Duration::from_millis(500)).await;
    println!("✅ 全球成就达成率统计已获取");
    
    println!("\n💡 === 可用的Web API接口 ===");
    println!("✅ ISteamNews.GetNewsForApp - 获取游戏新闻");
    println!("✅ ISteamUserStats.GetPlayerAchievements - 获取玩家成就");
    println!("✅ ISteamUserStats.GetUserStatsForGame - 获取玩家统计");
    println!("✅ ISteamApps.GetAppList - 获取应用列表");
    println!("✅ IPlayerService.GetOwnedGames - 获取拥有的游戏");
    println!("✅ ISteamUser.GetPlayerSummaries - 获取玩家概要");
    println!("✅ ISteamEconomy.GetAssetPrices - 获取资产价格");
}

/// 格式化时间戳
fn format_timestamp(timestamp: i64) -> String {
    // 简单的时间戳格式化
    let dt = std::time::UNIX_EPOCH + Duration::from_secs(timestamp as u64);
    format!("{:?}", dt) // 在实际应用中会使用更好的时间格式化
}

/// 格式化地区
fn format_region(region: u32) -> String {
    match region {
        0 => "美国东部".to_string(),
        1 => "美国西部".to_string(),
        2 => "韩国".to_string(),
        3 => "欧洲".to_string(),
        4 => "亚洲".to_string(),
        _ => format!("未知地区 ({})", region),
    }
}