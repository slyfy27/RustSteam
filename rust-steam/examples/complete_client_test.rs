//! 完整的Steam客户端功能测试
//! 
//! 这个示例展示了已完善的所有真实功能，不再包含模拟行为

use rust_steam::prelude::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use log::{info, error, debug};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("🚀 启动完整的Steam客户端功能测试");

    // 1. 测试基础类型系统
    test_basic_types().await?;

    // 2. 测试网络连接(仅测试结构，不实际连接)
    test_network_structures().await?;

    // 3. 测试协议消息序列化
    test_protocol_messages().await?;

    // 4. 测试认证系统结构
    test_authentication_structures().await?;

    // 5. 测试客户端结构
    test_client_structures().await?;

    // 6. 测试回调系统
    test_callback_system().await?;

    // 7. 测试工具函数
    test_utility_functions().await?;

    info!("✅ 所有功能测试完成");
    Ok(())
}

/// 测试基础类型系统
async fn test_basic_types() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 测试基础类型系统...");

    // 测试SteamID
    let steam_id = SteamID::new(76561198000000000);
    assert!(steam_id.is_valid());
    info!("✅ SteamID: {} -> {}", steam_id.id, steam_id.render());

    // 测试EResult
    let result = EResult::OK;
    info!("✅ EResult: {:?}", result);

    // 测试连接状态
    let state = ConnectionState::Disconnected;
    info!("✅ ConnectionState: {:?}", state);

    info!("✅ 基础类型系统测试完成");
    Ok(())
}

/// 测试网络结构
async fn test_network_structures() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 测试网络结构...");

    // 测试连接配置
    let steam_config = SteamConfiguration::default();
    let connection_config = ConnectionConfig::from_steam_config(&steam_config);
    
    info!("✅ 连接配置创建成功");
    info!("   - 协议类型: {:?}", connection_config.protocol_types);
    info!("   - 服务器数量: {}", connection_config.server_list.len());
    info!("   - 连接超时: {:?}", connection_config.connection_timeout);

    // 注意：不实际创建网络连接，因为这需要真实的Steam服务器
    info!("⚠️  网络连接测试已跳过(需要真实Steam服务器)");

    info!("✅ 网络结构测试完成");
    Ok(())
}

/// 测试协议消息
async fn test_protocol_messages() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 测试协议消息序列化...");

    // 测试Steam数据包
    let packet = SteamPacket::new(EMsg::ClientLogon, vec![1, 2, 3, 4]);
    let serialized = packet.serialize()?;
    info!("✅ 标准数据包序列化: {} 字节", serialized.len());

    // 测试扩展数据包
    let steam_id = SteamID::new(76561198000000000);
    let ext_packet = SteamPacket::new_extended(
        EMsg::ClientLogon,
        steam_id,
        12345,
        vec![5, 6, 7, 8]
    );
    let ext_serialized = ext_packet.serialize()?;
    info!("✅ 扩展数据包序列化: {} 字节", ext_serialized.len());

    // 测试加密请求
    let encrypt_request = ChannelEncryptRequest::new();
    let encrypt_data = encrypt_request.serialize()?;
    info!("✅ 加密请求序列化: {} 字节", encrypt_data.len());

    // 测试反序列化
    let deserialized_request = ChannelEncryptRequest::deserialize(&encrypt_data)?;
    assert_eq!(deserialized_request.protocol_version, encrypt_request.protocol_version);
    info!("✅ 加密请求反序列化成功");

    // 测试客户端登录消息
    let logon_msg = ClientLogon::new_with_tokens(
        "test_user".to_string(),
        "test_access_token".to_string()
    );
    let logon_data = logon_msg.serialize()?;
    info!("✅ 客户端登录消息序列化: {} 字节", logon_data.len());

    info!("✅ 协议消息测试完成");
    Ok(())
}

/// 测试认证系统结构
async fn test_authentication_structures() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 测试认证系统结构...");

    // 测试认证会话详情
    let auth_details = AuthSessionDetails {
        username: "test_user".to_string(),
        password: "test_password".to_string(),
        authenticator: Some(Arc::new(ConsoleAuthenticator)),
        ..Default::default()
    };
    
    info!("✅ 认证会话详情创建成功");
    info!("   - 用户名: {}", auth_details.username);
    info!("   - 设备名: {}", auth_details.device_friendly_name);
    info!("   - 平台类型: {:?}", auth_details.platform_type);

    // 测试登录详情
    let logon_details = LogOnDetails::with_credentials(
        "test_user".to_string(),
        "test_password".to_string()
    );
    
    assert_eq!(logon_details.username, Some("test_user".to_string()));
    assert_eq!(logon_details.password, Some("test_password".to_string()));
    info!("✅ 登录详情创建成功");

    // 测试令牌登录详情
    let token_details = LogOnDetails::with_access_token("test_access_token".to_string());
    assert_eq!(token_details.access_token, Some("test_access_token".to_string()));
    info!("✅ 令牌登录详情创建成功");

    // 注意：不实际创建认证会话，因为这需要网络连接
    info!("⚠️  认证会话测试已跳过(需要网络连接)");

    info!("✅ 认证系统结构测试完成");
    Ok(())
}

/// 测试客户端结构
async fn test_client_structures() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 测试客户端结构...");

    // 测试客户端设置
    let settings = ClientSettings::new();
    assert_eq!(settings.connection_timeout, Duration::from_secs(30));
    assert!(settings.auto_reconnect);
    assert_eq!(settings.max_reconnect_attempts, 5);
    info!("✅ 客户端设置创建成功");

    // 测试Steam配置
    let config = SteamConfiguration::new();
    assert!(!config.protocol_types.is_empty());
    assert!(config.allow_direct_connection);
    assert_eq!(config.client_version, "1.0.0");
    info!("✅ Steam配置创建成功");

    // 测试客户端创建
    let client = SteamClient::new(settings);
    assert_eq!(client.get_connection_state(), ConnectionState::Disconnected);
    info!("✅ Steam客户端创建成功");
    info!("   - 初始连接状态: {:?}", client.get_connection_state());

    // 测试客户端状态查询
    assert!(!client.is_logged_on().await);
    assert!(client.get_steam_id().await.is_none());
    info!("✅ 客户端状态查询正常");

    info!("✅ 客户端结构测试完成");
    Ok(())
}

/// 测试回调系统
async fn test_callback_system() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 测试回调系统...");

    let callback_manager = Arc::new(CallbackManager::new());
    
    // 测试连接回调订阅
    let connect_subscription = callback_manager.subscribe(|callback: &ConnectedCallback| {
        info!("🔗 收到连接回调: 时间戳 {}", callback.timestamp);
    });
    
    // 测试断开连接回调订阅
    let disconnect_subscription = callback_manager.subscribe(|callback: &DisconnectedCallback| {
        info!("🔌 收到断开连接回调: {}, 原因: {}", callback.timestamp, callback.reason);
    });
    
    // 测试登录回调订阅
    let logon_subscription = callback_manager.subscribe(|callback: &LoggedOnCallback| {
        info!("🔐 收到登录回调: Steam ID {}, 结果: {:?}", callback.steam_id.render(), callback.result);
    });
    
    // 测试登出回调订阅
    let logoff_subscription = callback_manager.subscribe(|callback: &LoggedOffCallback| {
        info!("👋 收到登出回调: 结果: {:?}, 时间戳: {}", callback.result, callback.timestamp);
    });
    
    info!("✅ 回调订阅创建成功");

    // 触发测试回调
    let test_connect_callback = ConnectedCallback {
        timestamp: get_unix_timestamp(),
    };
    callback_manager.trigger(&test_connect_callback);
    
    let test_disconnect_callback = DisconnectedCallback {
        timestamp: get_unix_timestamp(),
        reason: "测试断开连接".to_string(),
        was_logged_on: false,
    };
    callback_manager.trigger(&test_disconnect_callback);
    
    // 等待回调处理
    sleep(Duration::from_millis(100)).await;
    callback_manager.process_callbacks().await;
    
    info!("✅ 回调触发测试完成");

    // 确保订阅正确释放
    drop(connect_subscription);
    drop(disconnect_subscription);
    drop(logon_subscription);
    drop(logoff_subscription);
    
    info!("✅ 回调系统测试完成");
    Ok(())
}

/// 测试工具函数
async fn test_utility_functions() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 测试工具函数...");

    // 测试Base64编码/解码
    let test_data = b"Hello, Steam!";
    let encoded = base64_encode(test_data);
    let decoded = base64_decode(&encoded)?;
    assert_eq!(test_data.to_vec(), decoded);
    info!("✅ Base64编码/解码: {} -> {} -> 原始数据", 
          String::from_utf8_lossy(test_data), encoded);

    // 测试十六进制编码/解码
    let hex_encoded = bytes_to_hex(test_data);
    let hex_decoded = hex_to_bytes(&hex_encoded)?;
    assert_eq!(test_data.to_vec(), hex_decoded);
    info!("✅ 十六进制编码/解码: {} -> {} -> 原始数据", 
          String::from_utf8_lossy(test_data), hex_encoded);

    // 测试哈希函数
    let sha1_hash = sha1_hash(test_data);
    let sha256_hash = sha256_hash(test_data);
    info!("✅ SHA1哈希: {}", bytes_to_hex(&sha1_hash));
    info!("✅ SHA256哈希: {}", bytes_to_hex(&sha256_hash));

    // 测试时间戳
    let timestamp = get_unix_timestamp();
    info!("✅ Unix时间戳: {}", timestamp);

    // 测试机器ID生成
    let machine_id = generate_machine_id();
    info!("✅ 机器ID生成: {} 字节", machine_id.len());

    // 测试Steam用户名验证
    assert!(is_valid_steam_username("valid_username"));
    assert!(!is_valid_steam_username(""));
    assert!(!is_valid_steam_username("a")); // 太短
    info!("✅ Steam用户名验证功能正常");

    // 测试文件大小格式化
    let file_size = format_file_size(1024 * 1024 + 512 * 1024); // 1.5MB
    info!("✅ 文件大小格式化: {}", file_size);

    // 测试文件名清理
    let cleaned = sanitize_filename("test<>file:|name?.exe");
    info!("✅ 文件名清理: test<>file:|name?.exe -> {}", cleaned);

    // 测试Steam ID转换
    let community_id = 76561198000000000u64;
    let steam_id = community_id_to_steam_id(community_id);
    info!("✅ Community ID转换: {} -> Steam ID {}", community_id, steam_id.render());

    let account_id = 39734272u32;
    let steam_id_from_account = account_id_to_steam_id(account_id);
    info!("✅ Account ID转换: {} -> Steam ID {}", account_id, steam_id_from_account.render());

    // 测试Steam商店URL解析
    if let Some(app_id) = parse_steam_store_url("https://store.steampowered.com/app/730/") {
        info!("✅ Steam商店URL解析: app/{}", app_id);
    }

    info!("✅ 工具函数测试完成");
    Ok(())
}

/// 演示完整的客户端工作流程(结构测试)
#[allow(dead_code)]
async fn demo_complete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    info!("🎯 演示完整的客户端工作流程(结构测试)...");

    // 1. 创建客户端
    let settings = ClientSettings {
        connection_timeout: Duration::from_secs(10),
        auto_reconnect: true,
        reconnect_delay: Duration::from_secs(3),
        max_reconnect_attempts: 3,
        heartbeat_interval: Duration::from_secs(30),
        log_level: "debug".to_string(),
    };
    
    let mut client = SteamClient::new(settings);
    info!("✅ 客户端创建完成");

    // 2. 设置配置
    let config = SteamConfiguration {
        protocol_types: vec![ProtocolType::TCP, ProtocolType::WebSocket],
        connection_timeout: Duration::from_secs(15),
        server_list: vec![
            "155.133.254.133:27017".to_string(),
            "155.133.254.133:27018".to_string(),
        ],
        allow_direct_connection: true,
        cell_id: Some(123),
        client_version: "1.0.0".to_string(),
    };
    
    client.set_configuration(config);
    info!("✅ 客户端配置完成");

    // 3. 订阅回调
    let callback_manager = client.get_callback_manager();
    
    let _connect_sub = callback_manager.subscribe(|callback: &ConnectedCallback| {
        info!("📡 客户端已连接: {}", callback.timestamp);
    });
    
    let _disconnect_sub = callback_manager.subscribe(|callback: &DisconnectedCallback| {
        info!("📡 客户端已断开: {}, 原因: {}", callback.timestamp, callback.reason);
    });
    
    let _logon_sub = callback_manager.subscribe(|callback: &LoggedOnCallback| {
        info!("📡 登录完成: Steam ID {}, 结果: {:?}", 
              callback.steam_id.render(), callback.result);
    });
    
    info!("✅ 回调订阅完成");

    // 注意：实际的连接和认证需要真实的Steam凭据和网络连接
    info!("⚠️  实际连接测试已跳过(需要真实凭据)");

    // 4. 验证客户端状态
    assert_eq!(client.get_connection_state(), ConnectionState::Disconnected);
    assert!(!client.is_logged_on().await);
    assert!(client.get_steam_id().await.is_none());
    
    info!("✅ 客户端状态验证完成");
    info!("✅ 完整工作流程演示完成");

    Ok(())
}

/// 性能测试
#[allow(dead_code)]
async fn performance_tests() -> Result<(), Box<dyn std::error::Error>> {
    info!("⚡ 开始性能测试...");

    let start_time = std::time::Instant::now();

    // 测试大量SteamID创建
    let mut steam_ids = Vec::new();
    for i in 0..10000 {
        steam_ids.push(SteamID::new(76561198000000000 + i));
    }
    
    let steam_id_time = start_time.elapsed();
    info!("✅ 创建10000个SteamID耗时: {:?}", steam_id_time);

    // 测试大量消息序列化
    let packet_start = std::time::Instant::now();
    let mut packets = Vec::new();
    for i in 0..1000 {
        let packet = SteamPacket::new(EMsg::ClientLogon, vec![i as u8; 100]);
        packets.push(packet.serialize()?);
    }
    
    let packet_time = packet_start.elapsed();
    info!("✅ 序列化1000个数据包耗时: {:?}", packet_time);

    // 测试回调系统性能
    let callback_start = std::time::Instant::now();
    let callback_manager = Arc::new(CallbackManager::new());
    
    // 创建多个订阅
    let mut subscriptions = Vec::new();
    for i in 0..100 {
        let sub = callback_manager.subscribe(move |_: &ConnectedCallback| {
            // 空处理器，测试订阅开销
            debug!("回调处理器 {}", i);
        });
        subscriptions.push(sub);
    }
    
    // 触发多个回调
    for _ in 0..100 {
        let callback = ConnectedCallback {
            timestamp: get_unix_timestamp(),
        };
        callback_manager.trigger(&callback);
    }
    
    let callback_time = callback_start.elapsed();
    info!("✅ 100个订阅者处理100个回调耗时: {:?}", callback_time);

    let total_time = start_time.elapsed();
    info!("✅ 性能测试总耗时: {:?}", total_time);

    Ok(())
}