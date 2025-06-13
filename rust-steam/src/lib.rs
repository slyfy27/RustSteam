//! # Rust Steam
//! 
//! A Rust implementation of the Steam protocol client, ported from JavaSteam.
//! 
//! ## 功能特性
//! 
//! - ✅ 核心类型系统 (SteamID, EResult, 等)
//! - ⚠️ 认证系统 (部分实现，有编译问题)
//! - ⚠️ 回调系统 (部分实现，有编译问题) 
//! - ⚠️ 网络层 (部分实现，有编译问题)
//! - ✅ 工具函数
//! 
//! ## 当前状态
//! 
//! 项目正在开发中，某些模块由于 Rust 特有的编译问题暂时被注释掉。
//! 详情请查看 `EXAMPLES_ISSUES.md`。
//!

// 核心类型系统 - 稳定工作
pub mod types;

// 工具函数 - 部分工作
pub mod utils;

// 以下模块暂时注释掉，因为有编译错误
// TODO: 修复这些模块的编译问题

// 认证系统 - 有 async trait 对象安全性问题
// pub mod authentication;

// 回调系统 - 有 Clone trait 问题
// pub mod callbacks;

// 客户端 - 依赖于上面的模块
// pub mod client;

// 网络层 - 有错误类型匹配问题
// pub mod networking;

// 处理器 - 依赖于其他模块
// pub mod handlers;

// 重新导出核心类型，方便使用
pub use types::*;

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
    pub use crate::utils::{
        community_id_to_steam_id, account_id_to_steam_id,
        base64_encode, base64_decode,
        bytes_to_hex, hex_to_bytes,
        sha1_hash, sha256_hash,
        get_unix_timestamp, generate_machine_id,
        is_valid_steam_username, format_file_size,
        sanitize_filename, parse_steam_store_url,
    };
}

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LIB_NAME: &str = env!("CARGO_PKG_NAME");

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
}
