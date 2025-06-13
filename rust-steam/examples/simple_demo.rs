use rust_steam::types::*;
use rust_steam::callbacks::*;
use rust_steam::utils::*;
use rust_steam::authentication::*;
use rust_steam::protocol::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust Steam 客户端演示 ===\n");

    // 测试基本类型系统
    println!("🔧 测试基本类型系统:");
    let steam_id = SteamID::new(76561198000000000);
    println!("   Steam ID: {}", steam_id.render());
    println!("   Account ID: {}", steam_id.get_account_id());
    println!("   Account Type: {:?}", steam_id.get_account_type());
    println!("   Account Universe: {:?}", steam_id.get_account_universe());
    println!("   Account Instance: {:?}", steam_id.get_account_instance());

    // 测试回调系统
    println!("\n📞 测试回调系统:");
    let callback_manager = CallbackManager::new();
    
    // 订阅连接回调
    let connected_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let connected_clone = connected_called.clone();
    let _connected_sub = callback_manager.subscribe(move |callback: &ConnectedCallback| {
        connected_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        println!("   ✅ 连接成功! 服务器时间: {:?}", callback.server_time);
    });

    // 订阅断开连接回调
    let disconnected_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let disconnected_clone = disconnected_called.clone();
    let _disconnected_sub = callback_manager.subscribe(move |callback: &DisconnectedCallback| {
        disconnected_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        println!("   ❌ 连接断开. 用户主动: {}", callback.user_initiated);
        if let Some(reason) = &callback.reason {
            println!("   原因: {}", reason);
        }
    });

    // 触发测试事件
    let connected_callback = ConnectedCallback {
        server_time: std::time::SystemTime::now(),
    };
    callback_manager.trigger_callback(connected_callback);

    let disconnected_callback = DisconnectedCallback {
        user_initiated: false,
        reason: Some("测试断开".to_string()),
    };
    callback_manager.trigger_callback(disconnected_callback);

    // 等待回调处理
    sleep(Duration::from_millis(100)).await;
    
    println!("   连接回调是否被调用: {}", connected_called.load(std::sync::atomic::Ordering::SeqCst));
    println!("   断开回调是否被调用: {}", disconnected_called.load(std::sync::atomic::Ordering::SeqCst));

    // 测试工具函数
    println!("\n🛠️ 测试工具函数:");
    let timestamp = get_unix_timestamp();
    println!("   当前Unix时间戳: {}", timestamp);
    
    let hex_data = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
    let hex_string = to_hex_string(&hex_data);
    println!("   十六进制编码: {}", hex_string);
    
    if let Ok(decoded) = from_hex_string(&hex_string) {
        println!("   十六进制解码: {:?}", decoded);
        println!("   解码正确: {}", decoded == hex_data);
    }

    // 测试协议系统
    println!("\n📡 测试协议系统:");
    
    // 创建标准消息头
    let header = MsgHdr {
        msg: EMsg::Invalid as u32,
        target_job_id: 12345,
        source_job_id: 67890,
    };
    
    println!("   消息头 - 消息类型: {}, 目标任务ID: {}, 源任务ID: {}", 
             header.msg, header.target_job_id, header.source_job_id);

    // 创建扩展消息头
    let ext_header = ExtendedMsgHdr {
        msg: EMsg::ClientLogOn as u32,
        header_size: std::mem::size_of::<ExtendedMsgHdr>() as u8,
        header_version: 2,
        target_job_id: 11111,
        source_job_id: 22222,
        header_canary: 0x239,
        steam_id: steam_id.id,
        session_id: 0,
    };
    
    println!("   扩展消息头 - 消息类型: {}, Steam ID: {}, 会话ID: {}", 
             ext_header.msg, ext_header.steam_id, ext_header.session_id);

    // 创建Steam数据包
    let packet = SteamPacket::new(EMsg::ClientLogOn, vec![1, 2, 3, 4, 5]);
    println!("   数据包 - 消息类型: {:?}, 数据长度: {}", packet.msg_type, packet.data.len());

    // 测试认证相关类型
    println!("\n🔐 测试认证系统类型:");
    
    let auth_details = AuthSessionDetails {
        username: "test_user".to_string(),
        password: "test_password".to_string(),
        guard_code: None,
        authenticator: None,
        device_friendly_name: "Rust Steam Demo".to_string(),
        platform_type: EAuthTokenPlatformType::MobileApp,
        language: "chinese".to_string(),
        persist_login: false,
        website_id: "Client".to_string(),
        client_id: "DE45CD61".to_string(),
    };
    
    println!("   认证详情 - 用户名: {}, 设备名: {}", 
             auth_details.username, auth_details.device_friendly_name);
    println!("   平台类型: {:?}, 语言: {}", 
             auth_details.platform_type, auth_details.language);

    let auth_result = AuthenticationResult {
        success: true,
        requires_2fa: false,
        requires_email_verification: false,
        account_name: Some("test_account".to_string()),
        access_token: Some("access_token_123".to_string()),
        refresh_token: Some("refresh_token_456".to_string()),
        error_message: None,
    };
    
    println!("   认证结果 - 成功: {}, 需要2FA: {}, 需要邮箱验证: {}", 
             auth_result.success, auth_result.requires_2fa, auth_result.requires_email_verification);

    if let Some(ref account) = auth_result.account_name {
        println!("   账户名: {}", account);
    }

    // 测试消息类型转换
    println!("\n🔄 测试消息类型转换:");
    let msg_types = vec![
        EMsg::Invalid,
        EMsg::Multi,
        EMsg::BaseGeneral,
        EMsg::ClientLogOn,
        EMsg::ClientLogOff,
    ];
    
    for msg_type in msg_types {
        let as_u32 = msg_type as u32;
        if let Some(converted_back) = EMsg::from_u32(as_u32) {
            println!("   {:?} -> {} -> {:?} ✓", msg_type, as_u32, converted_back);
        } else {
            println!("   {:?} -> {} -> 转换失败 ✗", msg_type, as_u32);
        }
    }

    // 测试结果类型
    println!("\n📋 测试结果类型:");
    let results = vec![
        EResult::OK,
        EResult::Fail,
        EResult::NoConnection,
        EResult::InvalidPassword,
        EResult::LoggedInElsewhere,
    ];
    
    for result in results {
        println!("   {:?} ({})", result, result as u32);
    }

    println!("\n✨ 演示完成!");
    println!("💡 这个演示展示了Rust Steam客户端库的基础功能:");
    println!("   - Steam ID 处理和渲染");
    println!("   - 类型安全的回调系统");
    println!("   - 协议消息序列化/反序列化");
    println!("   - 认证系统数据结构");
    println!("   - 工具函数(时间戳、十六进制编码等)");
    println!("   - Steam协议相关的枚举和类型转换");
    
    Ok(())
}