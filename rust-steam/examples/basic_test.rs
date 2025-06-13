// 基础测试程序 - 演示核心功能
use std::time::SystemTime;

// 复制一些基础类型定义，避免编译依赖问题
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EAccountType {
    Invalid = 0,
    Individual = 1,
    Multiseat = 2,
    GameServer = 3,
    AnonGameServer = 4,
    Pending = 5,
    ContentServer = 6,
    Clan = 7,
    Chat = 8,
    ConsoleUser = 9,
    AnonUser = 10,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EAccountUniverse {
    Invalid = 0,
    Public = 1,
    Beta = 2,
    Internal = 3,
    Dev = 4,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EAccountInstance {
    All = 0,
    Desktop = 1,
    Console = 2,
    Web = 4,
}

#[derive(Debug, Clone, Copy)]
pub struct SteamID {
    pub id: u64,
}

impl SteamID {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn render(&self) -> String {
        format!("STEAM_{}:{}:{}", 
                self.get_account_universe() as u32,
                self.id & 1,
                (self.id >> 1) & 0x7FFFFFFF)
    }

    pub fn get_account_id(&self) -> u32 {
        (self.id & 0xFFFFFFFF) as u32
    }

    pub fn get_account_type(&self) -> EAccountType {
        let account_type = ((self.id >> 52) & 0xF) as u32;
        match account_type {
            0 => EAccountType::Invalid,
            1 => EAccountType::Individual,
            2 => EAccountType::Multiseat,
            3 => EAccountType::GameServer,
            4 => EAccountType::AnonGameServer,
            5 => EAccountType::Pending,
            6 => EAccountType::ContentServer,
            7 => EAccountType::Clan,
            8 => EAccountType::Chat,
            9 => EAccountType::ConsoleUser,
            10 => EAccountType::AnonUser,
            _ => EAccountType::Invalid,
        }
    }

    pub fn get_account_universe(&self) -> EAccountUniverse {
        let universe = ((self.id >> 56) & 0xFF) as u32;
        match universe {
            0 => EAccountUniverse::Invalid,
            1 => EAccountUniverse::Public,
            2 => EAccountUniverse::Beta,
            3 => EAccountUniverse::Internal,
            4 => EAccountUniverse::Dev,
            _ => EAccountUniverse::Invalid,
        }
    }

    pub fn get_account_instance(&self) -> EAccountInstance {
        let instance = ((self.id >> 32) & 0xFFFFF) as u32;
        match instance {
            0 => EAccountInstance::All,
            1 => EAccountInstance::Desktop,
            2 => EAccountInstance::Console,
            4 => EAccountInstance::Web,
            _ => EAccountInstance::Desktop,
        }
    }
}

// 工具函数
pub fn get_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn to_hex_string(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<String>()
}

pub fn from_hex_string(hex: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if hex.len() % 2 != 0 {
        return Err("十六进制字符串长度必须是偶数".into());
    }
    
    let result: Result<Vec<u8>, _> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i+2], 16))
        .collect();
    
    Ok(result?)
}

fn main() {
    println!("=== Rust Steam 基础功能测试 ===\n");

    // 测试Steam ID系统
    println!("🆔 测试Steam ID系统:");
    let test_ids = vec![
        76561198000000001,
        76561198042069719,
        76561198154250288,
        76561197960265728,
    ];

    for steam_id_num in test_ids {
        let steam_id = SteamID::new(steam_id_num);
        println!("   原始ID: {}", steam_id_num);
        println!("   渲染格式: {}", steam_id.render());
        println!("   账户ID: {}", steam_id.get_account_id());
        println!("   账户类型: {:?}", steam_id.get_account_type());
        println!("   账户宇宙: {:?}", steam_id.get_account_universe());
        println!("   账户实例: {:?}", steam_id.get_account_instance());
        println!();
    }

    // 测试工具函数
    println!("🛠️ 测试工具函数:");
    let timestamp = get_unix_timestamp();
    println!("   当前Unix时间戳: {}", timestamp);
    println!("   可读时间: {:?}", 
             std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp));

    // 测试十六进制编码
    let test_data = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
    let hex_string = to_hex_string(&test_data);
    println!("   十六进制编码: {:?} -> {}", test_data, hex_string);
    
    match from_hex_string(&hex_string) {
        Ok(decoded) => {
            println!("   十六进制解码: {} -> {:?}", hex_string, decoded);
            println!("   解码正确性: {}", decoded == test_data);
        }
        Err(e) => {
            println!("   解码失败: {}", e);
        }
    }

    // 测试错误的十六进制字符串
    match from_hex_string("ABC") {
        Ok(_) => println!("   错误：应该解码失败"),
        Err(e) => println!("   正确处理错误输入: {}", e),
    }

    // 测试Steam协议相关功能
    println!("\n📡 测试协议相关功能:");
    
    // 模拟消息类型
    #[derive(Debug, Clone, Copy)]
    enum TestMsgType {
        Invalid = 0,
        ClientLogOn = 702,
        ClientLogOff = 703,
        ClientHeartBeat = 704,
    }

    let msg_types = vec![
        TestMsgType::Invalid,
        TestMsgType::ClientLogOn,
        TestMsgType::ClientLogOff,
        TestMsgType::ClientHeartBeat,
    ];

    for msg_type in msg_types {
        println!("   消息类型: {:?} ({})", msg_type, msg_type as u32);
    }

    // 模拟数据包结构
    #[derive(Debug)]
    struct TestPacket {
        msg_type: u32,
        data: Vec<u8>,
        timestamp: u64,
    }

    let packets = vec![
        TestPacket {
            msg_type: 702,
            data: vec![0x01, 0x02, 0x03, 0x04],
            timestamp: get_unix_timestamp(),
        },
        TestPacket {
            msg_type: 703,
            data: vec![0x05, 0x06, 0x07, 0x08],
            timestamp: get_unix_timestamp(),
        },
    ];

    for (i, packet) in packets.iter().enumerate() {
        println!("   数据包 {}: 类型={}, 数据长度={}, 时间戳={}", 
                 i + 1, packet.msg_type, packet.data.len(), packet.timestamp);
        println!("      数据: {}", to_hex_string(&packet.data));
    }

    // 测试认证相关数据结构
    println!("\n🔐 测试认证数据结构:");
    
    #[derive(Debug)]
    struct TestAuthDetails {
        username: String,
        device_name: String,
        language: String,
        persist_login: bool,
    }

    let auth_details = TestAuthDetails {
        username: "test_user".to_string(),
        device_name: "Rust Steam Client".to_string(),
        language: "chinese".to_string(),
        persist_login: false,
    };

    println!("   认证详情: {:?}", auth_details);

    #[derive(Debug)]
    struct TestAuthResult {
        success: bool,
        requires_2fa: bool,
        account_name: Option<String>,
        access_token: Option<String>,
    }

    let auth_results = vec![
        TestAuthResult {
            success: true,
            requires_2fa: false,
            account_name: Some("successful_user".to_string()),
            access_token: Some("token_123".to_string()),
        },
        TestAuthResult {
            success: false,
            requires_2fa: true,
            account_name: None,
            access_token: None,
        },
    ];

    for (i, result) in auth_results.iter().enumerate() {
        println!("   认证结果 {}: {:?}", i + 1, result);
    }

    // 性能测试
    println!("\n⚡ 简单性能测试:");
    let start = std::time::Instant::now();
    let iterations = 100_000u64;

    for i in 0..iterations {
        let steam_id = SteamID::new(76561198000000000 + i);
        let _rendered = steam_id.render();
        let _account_id = steam_id.get_account_id();
    }

    let duration = start.elapsed();
    println!("   {} 次Steam ID操作耗时: {:?}", iterations, duration);
    println!("   平均每次操作: {:?}", duration / iterations as u32);

    // 内存测试
    println!("\n💾 内存使用测试:");
    let mut steam_ids = Vec::new();
    for i in 0..10000 {
        steam_ids.push(SteamID::new(76561198000000000 + i));
    }
    println!("   创建了 {} 个Steam ID对象", steam_ids.len());
    println!("   每个Steam ID大小: {} 字节", std::mem::size_of::<SteamID>());
    println!("   总内存使用: {} 字节", steam_ids.len() * std::mem::size_of::<SteamID>());

    println!("\n✨ 基础功能测试完成!");
    println!("💡 这个测试展示了以下核心功能:");
    println!("   - Steam ID 解析和渲染");
    println!("   - 账户类型、宇宙、实例识别");
    println!("   - 时间戳处理");
    println!("   - 十六进制编码/解码");
    println!("   - 基础数据结构");
    println!("   - 错误处理");
    println!("   - 性能特性");
    println!("   - 内存使用情况");
}