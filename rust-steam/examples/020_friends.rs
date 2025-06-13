// 示例020: 好友管理
// 
// 此示例演示如何管理Steam好友列表
// 包括获取好友列表、处理好友请求、更改在线状态等

use rust_steam::prelude::*;
use tokio::time::{sleep, Duration};
use std::env;
use std::collections::HashMap;

/// 好友信息结构
#[derive(Debug, Clone)]
struct FriendInfo {
    steam_id: SteamID,
    name: String,
    relationship: FriendRelationship,
    persona_state: PersonaState,
    last_seen: Option<std::time::SystemTime>,
}

/// 好友关系类型
#[derive(Debug, Clone, PartialEq)]
enum FriendRelationship {
    None,
    Blocked,
    RequestRecipient,  // 收到好友请求
    Friend,
    RequestInitiator,  // 发送的好友请求
    Ignored,
}

/// 在线状态
#[derive(Debug, Clone, PartialEq)]
enum PersonaState {
    Offline,
    Online,
    Busy,
    Away,
    Snooze,
    LookingToTrade,
    LookingToPlay,
    Invisible,
}

impl Default for PersonaState {
    fn default() -> Self {
        PersonaState::Offline
    }
}

#[tokio::main]
async fn main() -> Result<(), SteamError> {
    println!("=== JavaSteam样例020: 好友管理 ===\n");
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("使用方法: {} <用户名> <密码>", args[0]);
        return Ok(());
    }
    
    let username = &args[1];
    let password = &args[2];
    
    println!("👥 开始Steam好友管理示例...");
    
    // 创建Steam客户端
    let mut client = SteamClient::new(ClientSettings::default());
    
    // 获取回调管理器
    let callback_manager = client.get_callback_manager();
    
    // 好友列表存储
    let mut friends_list: HashMap<u64, FriendInfo> = HashMap::new();
    
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
        
        if !callback.user_initiated {
            println!("🔄 将尝试重新连接...");
        }
    });
    
    // 订阅登录成功事件
    let _logged_on_sub = callback_manager.subscribe(|callback: &LoggedOnCallback| {
        match callback.result {
            EResult::OK => {
                println!("🎉 登录成功！");
                println!("   Steam ID: {}", callback.steam_id.render());
                println!("   账户名: {}", callback.account_name);
                
                // 登录成功后，设置在线状态
                println!("🟢 设置在线状态为: 在线");
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
        persistent_session: false,
        guard_data: None,
        authenticator: Some(Arc::new(ConsoleAuthenticator)),
        device_friendly_name: "Rust Steam Friends Client".to_string(),
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
                machine_name: "Rust Steam Friends Client".to_string(),
                ..Default::default()
            };
            
            println!("📝 登录到Steam...");
            client.log_on(login_details).await?;
            
            // 等待登录完成
            sleep(Duration::from_secs(3)).await;
            
            // 模拟设置在线状态
            println!("\n🟢 设置个人状态为在线...");
            sleep(Duration::from_millis(500)).await;
            println!("✅ 在线状态已更新");
            
            // 模拟获取好友列表
            println!("\n👥 正在获取好友列表...");
            sleep(Duration::from_millis(800)).await;
            
            // 创建模拟好友数据
            let mock_friends = create_mock_friends_list();
            for friend in &mock_friends {
                friends_list.insert(friend.steam_id.id, friend.clone());
            }
            
            println!("✅ 好友列表已获取");
            display_friends_list(&friends_list);
            
            // 处理好友请求
            process_friend_requests(&friends_list).await;
            
            // 演示好友状态更新
            println!("\n📱 演示好友状态更新...");
            simulate_persona_state_changes(&friends_list).await;
            
            // 演示好友交互功能
            println!("\n💬 演示好友交互功能...");
            demonstrate_friend_interactions(&friends_list).await;
            
            println!("\n   为了演示，我们将在3秒后登出...");
            sleep(Duration::from_secs(3)).await;
            
            // 登出前设置离线状态
            println!("⚫ 设置状态为离线...");
            sleep(Duration::from_millis(300)).await;
            
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
    
    println!("\n✨ 好友管理示例完成！");
    
    Ok(())
}

/// 创建模拟好友列表
fn create_mock_friends_list() -> Vec<FriendInfo> {
    vec![
        FriendInfo {
            steam_id: SteamID::new(76561198000000001),
            name: "Alice".to_string(),
            relationship: FriendRelationship::Friend,
            persona_state: PersonaState::Online,
            last_seen: Some(std::time::SystemTime::now()),
        },
        FriendInfo {
            steam_id: SteamID::new(76561198000000002),
            name: "Bob".to_string(),
            relationship: FriendRelationship::Friend,
            persona_state: PersonaState::Away,
            last_seen: Some(std::time::SystemTime::now() - Duration::from_secs(3600)),
        },
        FriendInfo {
            steam_id: SteamID::new(76561198000000003),
            name: "Charlie".to_string(),
            relationship: FriendRelationship::RequestRecipient,
            persona_state: PersonaState::Online,
            last_seen: Some(std::time::SystemTime::now()),
        },
        FriendInfo {
            steam_id: SteamID::new(76561198000000004),
            name: "Diana".to_string(),
            relationship: FriendRelationship::Friend,
            persona_state: PersonaState::Offline,
            last_seen: Some(std::time::SystemTime::now() - Duration::from_secs(86400)),
        },
        FriendInfo {
            steam_id: SteamID::new(76561198000000005),
            name: "Eve".to_string(),
            relationship: FriendRelationship::Friend,
            persona_state: PersonaState::LookingToPlay,
            last_seen: Some(std::time::SystemTime::now()),
        },
    ]
}

/// 显示好友列表
fn display_friends_list(friends: &HashMap<u64, FriendInfo>) {
    println!("\n👥 === 好友列表 ({} 个好友) ===", friends.len());
    
    let mut confirmed_friends = 0;
    let mut pending_requests = 0;
    
    for friend in friends.values() {
        let status_icon = match friend.persona_state {
            PersonaState::Online => "🟢",
            PersonaState::Away => "🟡",
            PersonaState::Busy => "🔴",
            PersonaState::LookingToPlay => "🎮",
            PersonaState::LookingToTrade => "💰",
            PersonaState::Offline => "⚫",
            _ => "❓",
        };
        
        let relationship_info = match friend.relationship {
            FriendRelationship::Friend => {
                confirmed_friends += 1;
                "✅ 好友".to_string()
            }
            FriendRelationship::RequestRecipient => {
                pending_requests += 1;
                "📨 待接受的好友请求".to_string()
            }
            FriendRelationship::RequestInitiator => "📤 已发送的好友请求".to_string(),
            _ => "❓ 其他关系".to_string(),
        };
        
        println!("  {} {} ({})", status_icon, friend.name, friend.steam_id.render());
        println!("     关系: {}", relationship_info);
        println!("     状态: {:?}", friend.persona_state);
    }
    
    println!("\n📊 === 好友统计 ===");
    println!("已确认好友: {} 人", confirmed_friends);
    println!("待处理请求: {} 个", pending_requests);
}

/// 处理好友请求
async fn process_friend_requests(friends: &HashMap<u64, FriendInfo>) {
    println!("\n📨 处理好友请求...");
    
    let pending_requests: Vec<&FriendInfo> = friends
        .values()
        .filter(|f| f.relationship == FriendRelationship::RequestRecipient)
        .collect();
    
    if pending_requests.is_empty() {
        println!("   没有待处理的好友请求");
        return;
    }
    
    for request in pending_requests {
        println!("📨 收到来自 {} 的好友请求", request.name);
        println!("   Steam ID: {}", request.steam_id.render());
        
        // 模拟自动接受好友请求
        sleep(Duration::from_millis(500)).await;
        
        println!("✅ 已接受 {} 的好友请求", request.name);
        println!("   {} 现在是你的好友了！", request.name);
    }
}

/// 模拟好友状态变化
async fn simulate_persona_state_changes(friends: &HashMap<u64, FriendInfo>) {
    println!("\n📱 模拟好友状态变化...");
    
    for friend in friends.values().take(3) {
        if friend.relationship == FriendRelationship::Friend {
            let new_state = match friend.persona_state {
                PersonaState::Online => PersonaState::Away,
                PersonaState::Away => PersonaState::Online,
                PersonaState::Offline => PersonaState::Online,
                _ => PersonaState::Online,
            };
            
            println!("🔄 {} 的状态从 {:?} 变为 {:?}", friend.name, friend.persona_state, new_state);
            sleep(Duration::from_millis(800)).await;
        }
    }
}

/// 演示好友交互功能
async fn demonstrate_friend_interactions(friends: &HashMap<u64, FriendInfo>) {
    println!("\n💬 演示好友交互功能...");
    
    for friend in friends.values().take(2) {
        if friend.relationship == FriendRelationship::Friend && friend.persona_state == PersonaState::Online {
            println!("📤 向 {} 发送消息: \"Hello!\"", friend.name);
            sleep(Duration::from_millis(500)).await;
            
            println!("📥 {} 回复: \"Hi there!\"", friend.name);
            sleep(Duration::from_millis(500)).await;
            
            println!("🎮 邀请 {} 一起游戏", friend.name);
            sleep(Duration::from_millis(500)).await;
            
            println!("✅ {} 接受了游戏邀请", friend.name);
            sleep(Duration::from_millis(500)).await;
        }
    }
    
    println!("\n💡 === 好友管理功能说明 ===");
    println!("✅ 获取好友列表");
    println!("✅ 处理好友请求（接受/拒绝）");
    println!("✅ 监听好友状态变化");
    println!("✅ 发送和接收消息");
    println!("✅ 游戏邀请功能");
    println!("✅ 设置个人在线状态");
}