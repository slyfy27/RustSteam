//! # RustSteam
//! 
//! A Rust implementation of the Steam protocol client, ported from JavaSteam.
//! This library allows you to connect to and interact with Valve's Steam network.
//! 
//! ## Features
//! 
//! - Modern authentication with JWT tokens
//! - Two-factor authentication support (TOTP, SMS, Email)
//! - WebSocket and TCP connection support
//! - Async/await support throughout
//! - Type-safe Steam protocol messages
//! - Content downloading capabilities
//! - Friends management
//! - Store and community features
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! use rust_steam::prelude::*;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = SteamClient::new().await?;
//!     
//!     // Connect to Steam
//!     client.connect().await?;
//!     
//!     // Authenticate
//!     let auth_details = AuthSessionDetails {
//!         username: "your_username".to_string(),
//!         password: "your_password".to_string(),
//!         ..Default::default()
//!     };
//!     
//!     client.authenticate(auth_details).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod authentication;
pub mod callbacks;
pub mod client;
pub mod handlers;
pub mod networking;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub mod prelude {
    pub use crate::authentication::*;
    pub use crate::callbacks::{Callback, CallbackManager};
    pub use crate::client::{SteamClient, SteamConfiguration};
    pub use crate::handlers::steam_user::*;
    pub use crate::types::*;
}

// Re-export for backward compatibility
pub use client::SteamClient;
pub use authentication::{AuthSessionDetails, AuthPollResult};
pub use types::{EResult, SteamID};

/// Current version of the rust-steam library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library result type
pub type Result<T> = std::result::Result<T, crate::types::SteamError>;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
