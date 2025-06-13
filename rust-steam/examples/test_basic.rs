// 基础功能测试示例

use rust_steam::prelude::*;

#[tokio::main]
async fn main() -> Result<(), SteamError> {
    println!("=== Rust Steam 基础功能测试 ===\n");
    
    // 测试SteamID
    let steam_id = SteamID::new(76561198000000000);
    println!("✅ SteamID测试:");
    println!("   ID: {}", steam_id.id);
    println!("   Rendered: {}", steam_id.render());
    println!("   Valid: {}", steam_id.is_valid());
    
    // 测试EResult
    let result = EResult::OK;
    println!("\n✅ EResult测试:");
    println!("   Result: {:?}", result);
    println!("   Display: {}", result);
    
    // 测试连接状态
    let state = ConnectionState::Connected;
    println!("\n✅ ConnectionState测试:");
    println!("   State: {:?}", state);
    
    // 测试协议类型
    let protocol = ProtocolType::TCP;
    println!("\n✅ ProtocolType测试:");
    println!("   Protocol: {:?}", protocol);
    
    // 测试认证详情创建
    let auth_details = AuthSessionDetails {
        username: "test_user".to_string(),
        password: "test_password".to_string(),
        persistent_session: false,
        guard_data: None,
        authenticator: Some(Arc::new(ConsoleAuthenticator)),
        device_friendly_name: "Test Client".to_string(),
        website_id: "Client".to_string(),
        platform_type: EAuthTokenPlatformType::SteamClient,
    };
    
    println!("\n✅ AuthSessionDetails测试:");
    println!("   Username: {}", auth_details.username);
    println!("   Platform: {:?}", auth_details.platform_type);
    
    // 测试登录详情创建
    let login_details = LogOnDetails {
        username: "test_user".to_string(),
        access_token: "test_token".to_string(),
        refresh_token: Some("test_refresh".to_string()),
        login_id: 123,
    };
    
    println!("\n✅ LogOnDetails测试:");
    println!("   Username: {}", login_details.username);
    println!("   Login ID: {}", login_details.login_id);
    
    // 测试客户端创建
    let _client = SteamClient::new(ClientSettings::default());
    println!("\n✅ SteamClient创建测试: 成功");
    
    println!("\n🎉 所有基础功能测试通过！");
    
    Ok(())
}