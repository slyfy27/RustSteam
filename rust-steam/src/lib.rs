//! # Rust Steam
//! 
//! A Rust implementation of the Steam protocol client, ported from JavaSteam.
//! 
//! ## 功能特性
//! 
//! - ✅ 核心类型系统 (SteamID, EResult, 等)
//! - ✅ 认证系统 (支持2FA和现代JWT认证)
//! - ✅ 回调系统 (类型安全的事件驱动架构) 
//! - ✅ Steam客户端 (连接、登录、消息处理)
//! - ✅ 网络层 (TCP/UDP/WebSocket连接)
//! - ✅ 协议消息 (Steam协议消息处理)
//! - ✅ 工具函数
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use rust_steam::prelude::*;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = SteamClient::new(ClientSettings::default());
//!     client.connect().await?;
//!     
//!     let auth_details = AuthSessionDetails {
//!         username: "your_username".to_string(),
//!         password: "your_password".to_string(),
//!         authenticator: Some(Arc::new(ConsoleAuthenticator)),
//!         ..Default::default()
//!     };
//!     
//!     let auth_result = client.authenticate(auth_details).await?;
//!     // ... 处理登录逻辑
//!     
//!     Ok(())
//! }
//! ```

// 核心类型系统 - 稳定工作
pub mod types;

// 工具函数 - 稳定工作
pub mod utils;

// 认证系统 - 已修复
pub mod authentication;

// 回调系统 - 已修复
pub mod callbacks;

// 客户端 - 已修复
pub mod client;

// 网络层 - 已修复
pub mod networking;

// 协议消息 - 新增
pub mod protocol;

// 处理器 - 已修复
pub mod handlers;

// 重新导出核心类型，方便使用
pub use types::*;

// 重新导出认证相关
pub use authentication::{
    AuthSessionDetails, AuthPollResult, LogOnDetails, 
    ConsoleAuthenticator, EAuthSessionGuardType, 
    EAuthTokenPlatformType, EPlatformType
};

// 重新导出回调相关
pub use callbacks::{
    CallbackManager, ConnectedCallback, DisconnectedCallback,
    LoggedOnCallback, LoggedOffCallback, CallbackSubscription
};

// 重新导出客户端相关
pub use client::{SteamClient, ClientSettings, SteamConfiguration};

// 重新导出网络相关
pub use networking::{ConnectionManager, ConnectionConfig};

// 重新导出协议相关
pub use protocol::{SteamPacket, ClientLogon, ClientLogonResponse, ChannelEncryptRequest, ChannelEncryptResponse};

// 重新导出工具函数（只导出存在的函数）
pub use utils::{
    community_id_to_steam_id, account_id_to_steam_id,
    base64_encode, base64_decode,
    bytes_to_hex, hex_to_bytes,
    sha1_hash, sha256_hash,
    get_unix_timestamp, generate_machine_id,
    is_valid_steam_username, format_file_size,
    sanitize_filename, parse_steam_store_url,
};

/// 预导入模块，包含常用类型和函数
pub mod prelude {
    pub use crate::types::*;
    pub use crate::authentication::{
        AuthSessionDetails, AuthPollResult, LogOnDetails, 
        ConsoleAuthenticator, EAuthSessionGuardType, 
        EAuthTokenPlatformType, EPlatformType
    };
    pub use crate::callbacks::{
        CallbackManager, ConnectedCallback, DisconnectedCallback,
        LoggedOnCallback, LoggedOffCallback, CallbackSubscription
    };
    pub use crate::client::{SteamClient, ClientSettings, SteamConfiguration};
    pub use crate::networking::{ConnectionManager, ConnectionConfig};
    pub use crate::protocol::{SteamPacket, ClientLogon, ClientLogonResponse, ChannelEncryptRequest, ChannelEncryptResponse};
    pub use crate::utils::{
        community_id_to_steam_id, account_id_to_steam_id,
        base64_encode, base64_decode,
        bytes_to_hex, hex_to_bytes,
        sha1_hash, sha256_hash,
        get_unix_timestamp, generate_machine_id,
        is_valid_steam_username, format_file_size,
        sanitize_filename, parse_steam_store_url,
    };
    pub use std::sync::Arc;
}

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LIB_NAME: &str = env!("CARGO_PKG_NAME");

/// 便捷的结果类型
pub type Result<T> = std::result::Result<T, SteamError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        let steam_id = SteamID::new(76561198000000000);
        assert!(steam_id.is_valid());
        assert_eq!(steam_id.id, 76561198000000000);
    }

    #[test]
    fn test_result_types() {
        let result = EResult::OK;
        assert_eq!(format!("{:?}", result), "OK");
    }

    #[test]
    fn test_utils() {
        let data = b"hello world";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = SteamClient::new(ClientSettings::default());
        assert_eq!(client.get_connection_state(), ConnectionState::Disconnected);
    }

    #[test]
    fn test_callback_manager() {
        let manager = CallbackManager::new();
        let _subscription = manager.subscribe(|_: &ConnectedCallback| {
            println!("Connection callback received");
        });
        // 测试创建成功
        assert!(!manager.should_shutdown());
    }

    #[test]
    fn test_protocol_packet() {
        let packet = SteamPacket::new(EMsg::ClientLogon, vec![1, 2, 3, 4]);
        let serialized = packet.serialize().unwrap();
        assert!(!serialized.is_empty());
    }
}
