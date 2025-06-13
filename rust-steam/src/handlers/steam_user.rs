//! Steam User handler
//! 
//! Handles user authentication, login, logout, and user-related operations

use crate::callbacks::{CallbackManager, LoggedOnCallback, LoggedOffCallback};
use crate::types::{EResult, SteamError, SteamID};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Login details for Steam authentication
#[derive(Debug, Clone, Default)]
pub struct LogOnDetails {
    pub username: Option<String>,
    pub password: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub login_id: Option<u32>,
    pub login_key: Option<String>,
    pub two_factor_code: Option<String>,
    pub auth_code: Option<String>,
    pub sentinel_file_hash: Option<Vec<u8>>,
    pub machine_id: Option<Vec<u8>>,
    pub account_instance: u32,
    pub account_id: u32,
    pub client_language: String,
    pub client_os_type: u32,
    pub anon_server_list: Vec<String>,
    pub should_remember_password: bool,
    pub obfuscated_private_ip: u32,
    pub protocol_version: u32,
    pub cell_id: u32,
}

impl LogOnDetails {
    pub fn new() -> Self {
        Self {
            account_instance: 1,
            client_language: "english".to_string(),
            client_os_type: 16, // Windows
            protocol_version: 65580,
            ..Default::default()
        }
    }

    pub fn set_username(&mut self, username: String) {
        self.username = Some(username);
    }

    pub fn set_password(&mut self, password: String) {
        self.password = Some(password);
    }

    pub fn set_access_token(&mut self, token: String) {
        self.access_token = Some(token);
    }

    pub fn set_refresh_token(&mut self, token: String) {
        self.refresh_token = Some(token);
    }

    pub fn set_login_id(&mut self, login_id: u32) {
        self.login_id = Some(login_id);
    }

    pub fn set_two_factor_code(&mut self, code: String) {
        self.two_factor_code = Some(code);
    }

    pub fn set_auth_code(&mut self, code: String) {
        self.auth_code = Some(code);
    }
}

/// Steam User handler for managing user authentication and session
#[derive(Debug)]
pub struct SteamUser {
    callback_manager: Arc<CallbackManager>,
    steam_id: Option<SteamID>,
    is_logged_in: bool,
    session_token: Option<String>,
    login_details: Option<LogOnDetails>,
}

impl SteamUser {
    /// Create a new SteamUser handler
    pub fn new(callback_manager: Arc<CallbackManager>) -> Self {
        Self {
            callback_manager,
            steam_id: None,
            is_logged_in: false,
            session_token: None,
            login_details: None,
        }
    }

    /// Log on to Steam with the provided details
    pub async fn log_on(&self, details: LogOnDetails) -> Result<(), SteamError> {
        log::info!("Attempting to log on to Steam...");

        // Validate login details
        if details.access_token.is_none() && details.username.is_none() {
            return Err(SteamError::Authentication {
                message: "Either access token or username must be provided".to_string(),
            });
        }

        // Store login details
        // In a real implementation, we would send the login message to Steam
        // For now, we'll simulate a successful login
        
        // Simulate login processing delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Create a mock successful login response
        let mut logged_on_callback = LoggedOnCallback::new(EResult::OK);
        logged_on_callback.steam_id = SteamID::new(76561198000000000); // Mock Steam ID
        logged_on_callback.account_name = details.username.clone().unwrap_or_else(|| "mock_user".to_string());
        logged_on_callback.cell_id = details.cell_id;

        // Fire the logged on callback
        let callback = Box::new(logged_on_callback);
        self.callback_manager.fire_callback(callback).await?;

        log::info!("Successfully logged on to Steam");
        Ok(())
    }

    /// Log off from Steam
    pub async fn log_off(&self) -> Result<(), SteamError> {
        log::info!("Logging off from Steam...");

        // In a real implementation, we would send a logoff message to Steam
        // For now, we'll simulate a successful logoff

        // Simulate logoff processing delay
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Create logoff callback
        let logged_off_callback = LoggedOffCallback::new(EResult::OK);
        let callback = Box::new(logged_off_callback);
        self.callback_manager.fire_callback(callback).await?;

        log::info!("Successfully logged off from Steam");
        Ok(())
    }

    /// Get the current Steam ID
    pub fn get_steam_id(&self) -> Option<SteamID> {
        self.steam_id
    }

    /// Check if currently logged in
    pub fn is_logged_in(&self) -> bool {
        self.is_logged_in
    }

    /// Get the current session token
    pub fn get_session_token(&self) -> Option<&String> {
        self.session_token.as_ref()
    }

    /// Request a new login key
    pub async fn request_new_login_key(&self) -> Result<(), SteamError> {
        log::info!("Requesting new login key...");
        
        // In a real implementation, this would send a request to Steam
        // and handle the response with a callback
        
        Ok(())
    }

    /// Accept a new login key
    pub async fn accept_new_login_key(&self, _unique_id: u32, _login_key: &str) -> Result<(), SteamError> {
        log::info!("Accepting new login key...");
        
        // In a real implementation, this would send an accept message to Steam
        
        Ok(())
    }

    /// Send machine auth response
    pub async fn send_machine_auth_response(&self, details: MachineAuthDetails) -> Result<(), SteamError> {
        log::info!("Sending machine auth response...");
        
        // In a real implementation, this would send the machine auth response to Steam
        
        Ok(())
    }
}

/// Machine authentication details
#[derive(Debug, Clone)]
pub struct MachineAuthDetails {
    pub job_id: u64,
    pub filename: String,
    pub bytes_written: u32,
    pub file_size: u32,
    pub offset: u32,
    pub result: EResult,
    pub last_error: u32,
    pub one_time_password: u32,
    pub sentinel_file_hash: Vec<u8>,
}

impl MachineAuthDetails {
    pub fn new(job_id: u64, filename: String) -> Self {
        Self {
            job_id,
            filename,
            bytes_written: 0,
            file_size: 0,
            offset: 0,
            result: EResult::OK,
            last_error: 0,
            one_time_password: 0,
            sentinel_file_hash: Vec::new(),
        }
    }
}

/// User status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    pub steam_id: SteamID,
    pub persona_name: String,
    pub persona_state: u32,
    pub persona_state_flags: u32,
    pub avatar_hash: Vec<u8>,
    pub last_logoff: u32,
    pub last_logon: u32,
    pub last_seen_online: u32,
    pub clan_id: SteamID,
    pub game_server_id: SteamID,
    pub game_server_ip: u32,
    pub game_server_port: u16,
    pub query_port: u16,
    pub source_tv_port: u16,
    pub game_data_blob: Vec<u8>,
    pub game_name: String,
    pub game_id: u64,
    pub rich_presence: Vec<u8>,
    pub broadcast_id: u64,
    pub game_lobby_id: SteamID,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self {
            steam_id: SteamID::new(0),
            persona_name: String::new(),
            persona_state: 0,
            persona_state_flags: 0,
            avatar_hash: Vec::new(),
            last_logoff: 0,
            last_logon: 0,
            last_seen_online: 0,
            clan_id: SteamID::new(0),
            game_server_id: SteamID::new(0),
            game_server_ip: 0,
            game_server_port: 0,
            query_port: 0,
            source_tv_port: 0,
            game_data_blob: Vec::new(),
            game_name: String::new(),
            game_id: 0,
            rich_presence: Vec::new(),
            broadcast_id: 0,
            game_lobby_id: SteamID::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callbacks::CallbackManager;

    #[tokio::test]
    async fn test_steam_user_creation() {
        let callback_manager = Arc::new(CallbackManager::new());
        let steam_user = SteamUser::new(callback_manager);
        
        assert!(!steam_user.is_logged_in());
        assert!(steam_user.get_steam_id().is_none());
    }

    #[tokio::test]
    async fn test_login_details() {
        let mut details = LogOnDetails::new();
        details.set_username("test_user".to_string());
        details.set_password("test_pass".to_string());
        details.set_login_id(123);
        
        assert_eq!(details.username, Some("test_user".to_string()));
        assert_eq!(details.password, Some("test_pass".to_string()));
        assert_eq!(details.login_id, Some(123));
    }
}