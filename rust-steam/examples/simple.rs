use rust_steam::types::{SteamID, EResult, ConnectionState, ProtocolType};

fn main() {
    println!("🦀 Rust Steam 基础示例");
    println!("======================");
    
    // 演示SteamID
    let steam_id = SteamID::new(12345);
    println!("📋 SteamID: {:?}", steam_id);
    println!("   ID: {}", steam_id.id);
    println!("   是否有效: {}", steam_id.is_valid());
    
    // 演示结果枚举
    let result = EResult::OK;
    println!("✅ 结果状态: {:?}", result);
    
    // 演示连接状态
    let states = vec![
        ConnectionState::Disconnected,
        ConnectionState::Connecting, 
        ConnectionState::Connected,
        ConnectionState::Disconnecting,
    ];
    
    println!("🔗 连接状态:");
    for state in states {
        println!("   {:?}", state);
    }
    
    // 演示协议类型
    let protocols = vec![
        ProtocolType::TCP,
        ProtocolType::UDP,
        ProtocolType::WebSocket,
    ];
    
    println!("🌐 协议类型:");
    for protocol in protocols {
        println!("   {:?}", protocol);
    }
    
    println!("🏁 示例完成");
}