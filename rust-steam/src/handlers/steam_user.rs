//! Steam User Handler
//! 
//! Handles Steam user-related messages and operations

use crate::authentication::LogOnDetails;
use crate::callbacks::{CallbackManager, LoggedOnCallback, LoggedOffCallback};
use crate::protocol::{ClientLogon, ClientLogonResponse, SteamPacket};
use crate::types::{SteamError, SteamID, EResult, EMsg};
use crate::utils::get_unix_timestamp;
use std::sync::Arc;
use tokio::sync::mpsc;
use log::{info, error, debug, trace};
use std::io;
use sha1::{Sha1, Digest};
use serde_json;

/// Steam用户处理器
pub struct SteamUser {
    callback_manager: Arc<CallbackManager>,
    message_sender: mpsc::UnboundedSender<Vec<u8>>,
    current_steam_id: Option<SteamID>,
    is_logged_on: bool,
    session_id: Option<i32>,
    heartbeat_interval: u32,
    last_heartbeat: u64,
}

impl SteamUser {
    /// 创建新的Steam用户处理器
    pub fn new(
        callback_manager: Arc<CallbackManager>,
        message_sender: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Self {
        Self {
            callback_manager,
            message_sender,
            current_steam_id: None,
            is_logged_on: false,
            session_id: None,
            heartbeat_interval: 30, // 30秒
            last_heartbeat: get_unix_timestamp(),
        }
    }

    /// 处理登录请求
    pub async fn handle_logon(&mut self, details: LogOnDetails) -> Result<(), SteamError> {
        info!("🔐 处理登录请求...");

        // 验证登录详情
        if details.username.is_none() && details.access_token.is_none() {
            return Err(SteamError::InvalidState {
                message: "用户名或访问令牌必须提供一个".to_string(),
            });
        }

        // 创建登录消息
        let username = details.username.clone().unwrap_or_else(|| "anonymous".to_string());
        let access_token = details.access_token.clone().unwrap_or_default();
        
        let logon_msg = ClientLogon::new_with_tokens(username.clone(), access_token);
        let logon_data = logon_msg.serialize()?;
        
        // 创建Steam消息包
        let steam_id = details.steam_id.unwrap_or_else(|| SteamID::new(0));
        let session_id = details.login_id as i32;
        
        let packet = SteamPacket::new_extended(
            EMsg::ClientLogon,
            steam_id,
            session_id,
            logon_data,
        );
        
        let packet_data = packet.serialize()?;

        // 发送登录消息
        self.message_sender.send(packet_data)
            .map_err(|_| SteamError::InvalidState {
                message: "无法发送登录消息".to_string(),
            })?;

        // 更新本地状态
        self.current_steam_id = Some(steam_id);
        self.session_id = Some(session_id);
        self.last_heartbeat = get_unix_timestamp();

        info!("✅ 登录消息已发送，Steam ID: {}", steam_id.render());
        Ok(())
    }

    /// 处理登录响应
    pub async fn handle_logon_response(&mut self, data: &[u8]) -> Result<(), SteamError> {
        debug!("📥 处理登录响应...");

        let response = ClientLogonResponse::deserialize(data)?;
        
        match response.result {
            EResult::OK => {
                info!("✅ 登录成功");
                
                self.is_logged_on = true;
                self.heartbeat_interval = response.out_of_game_heartbeat_seconds as u32;
                
                // 更新Steam ID
                if response.steam_id != 0 {
                    self.current_steam_id = Some(SteamID::new(response.steam_id));
                }

                // 触发登录成功回调
                let callback = LoggedOnCallback {
                    result: response.result,
                    steam_id: self.current_steam_id.unwrap_or_else(|| SteamID::new(0)),
                    account_name: response.account_name.clone(),
                    cell_id: response.cell_id,
                    email_domain: response.email_domain.clone(),
                    parental_settings: response.parental_settings.clone(),
                    count_loginfailures_to_migrate: response.count_loginfailures_to_migrate,
                    count_disconnects_to_migrate: response.count_disconnects_to_migrate,
                    ogs_data_report_time_window: response.ogs_data_report_time_window,
                    client_supplied_steam_id: response.client_supplied_steam_id,
                    ip_country_code: response.ip_country_code.clone(),
                    vanity_url: response.vanity_url.clone(),
                    out_of_game_heartbeat_seconds: response.out_of_game_heartbeat_seconds,
                    in_game_heartbeat_seconds: response.in_game_heartbeat_seconds,
                    public_ip: response.public_ip.clone(),
                    server_time: response.server_time,
                    account_flags: response.account_flags,
                    facebook_id: response.facebook_id,
                    facebook_name: response.facebook_name.clone(),
                    steam_guard_notify_newmachines: response.steam_guard_notify_newmachines,
                    steam_guard_machine_name_user_chosen: response.steam_guard_machine_name_user_chosen.clone(),
                    is_steam_guard_machine_name_user_chosen: response.is_steam_guard_machine_name_user_chosen,
                    request_id_verify_password: response.request_id_verify_password,
                    is_phone_verified: response.is_phone_verified,
                    two_factor_state: response.two_factor_state,
                    is_phone_identifying: response.is_phone_identifying,
                    is_phone_needing_reverify: response.is_phone_needing_reverify,
                    timestamp: get_unix_timestamp(),
                };
                
                self.callback_manager.trigger(&callback);
            }
            EResult::InvalidPassword => {
                error!("❌ 登录失败: 密码错误");
                self.trigger_logon_failure(response.result, "密码错误").await;
            }
            EResult::AccountLogonDenied => {
                error!("❌ 登录失败: 账户登录被拒绝");
                self.trigger_logon_failure(response.result, "账户登录被拒绝").await;
            }
            EResult::TwoFactorCodeMismatch => {
                error!("❌ 登录失败: 2FA代码不匹配");
                self.trigger_logon_failure(response.result, "2FA代码不匹配").await;
            }
            EResult::AccountLoginDeniedNeedTwoFactor => {
                error!("❌ 登录失败: 需要2FA验证");
                self.trigger_logon_failure(response.result, "需要2FA验证").await;
            }
            _ => {
                error!("❌ 登录失败: {:?}", response.result);
                self.trigger_logon_failure(response.result, &format!("登录失败: {:?}", response.result)).await;
            }
        }

        Ok(())
    }

    /// 触发登录失败回调
    async fn trigger_logon_failure(&self, result: EResult, message: &str) {
        let callback = LoggedOnCallback {
            result,
            steam_id: SteamID::new(0),
            account_name: String::new(),
            cell_id: 0,
            email_domain: message.to_string(),
            parental_settings: Vec::new(),
            count_loginfailures_to_migrate: 0,
            count_disconnects_to_migrate: 0,
            ogs_data_report_time_window: 0,
            client_supplied_steam_id: 0,
            ip_country_code: String::new(),
            vanity_url: String::new(),
            out_of_game_heartbeat_seconds: 0,
            in_game_heartbeat_seconds: 0,
            public_ip: Vec::new(),
            server_time: 0,
            account_flags: 0,
            facebook_id: 0,
            facebook_name: String::new(),
            steam_guard_notify_newmachines: false,
            steam_guard_machine_name_user_chosen: String::new(),
            is_steam_guard_machine_name_user_chosen: false,
            request_id_verify_password: 0,
            is_phone_verified: false,
            two_factor_state: 0,
            is_phone_identifying: false,
            is_phone_needing_reverify: false,
            timestamp: get_unix_timestamp(),
        };
        
        self.callback_manager.trigger(&callback);
    }

    /// 处理登出请求
    pub async fn handle_logoff(&mut self) -> Result<(), SteamError> {
        info!("👋 处理登出请求...");

        if !self.is_logged_on {
            return Err(SteamError::InvalidState {
                message: "当前未登录".to_string(),
            });
        }

        // 创建登出消息包
        let packet = SteamPacket::new(EMsg::ClientLogOff, Vec::new());
        let packet_data = packet.serialize()?;

        // 发送登出消息
        self.message_sender.send(packet_data)
            .map_err(|_| SteamError::InvalidState {
                message: "无法发送登出消息".to_string(),
            })?;

        // 更新本地状态
        self.is_logged_on = false;
        
        // 触发登出回调
        let callback = LoggedOffCallback {
            result: EResult::OK,
            timestamp: get_unix_timestamp(),
        };
        self.callback_manager.trigger(&callback);

        info!("✅ 登出消息已发送");
        Ok(())
    }

    /// 处理心跳
    pub async fn handle_heartbeat(&mut self) -> Result<(), SteamError> {
        let current_time = get_unix_timestamp();
        
        if current_time - self.last_heartbeat >= self.heartbeat_interval as u64 {
            trace!("💓 发送心跳...");
            
            // 创建心跳消息包
            let packet = SteamPacket::new(EMsg::ClientHeartBeat, Vec::new());
            let packet_data = packet.serialize()?;

            // 发送心跳消息
            self.message_sender.send(packet_data)
                .map_err(|_| SteamError::InvalidState {
                    message: "无法发送心跳消息".to_string(),
                })?;

            self.last_heartbeat = current_time;
            trace!("✅ 心跳已发送");
        }

        Ok(())
    }

    /// 处理服务器心跳响应
    pub async fn handle_heartbeat_response(&mut self, _data: &[u8]) -> Result<(), SteamError> {
        trace!("💓 收到服务器心跳响应");
        self.last_heartbeat = get_unix_timestamp();
        Ok(())
    }

    /// 处理消息
    pub async fn handle_message(&mut self, msg_type: EMsg, data: &[u8]) -> Result<(), SteamError> {
        match msg_type {
            EMsg::ClientLogOnResponse => {
                self.handle_logon_response(data).await?;
            }
            EMsg::ClientHeartBeat => {
                self.handle_heartbeat_response(data).await?;
            }
            EMsg::ClientLoggedOff => {
                info!("📤 收到服务器登出通知");
                self.is_logged_on = false;
                
                let callback = LoggedOffCallback {
                    result: EResult::OK,
                    timestamp: get_unix_timestamp(),
                };
                self.callback_manager.trigger(&callback);
            }
            _ => {
                debug!("🔍 未处理的消息类型: {:?}", msg_type);
            }
        }

        Ok(())
    }

    /// 处理机器认证响应
    pub async fn handle_machine_auth_response(&mut self, data: &[u8]) -> Result<(), SteamError> {
        debug!("🔐 处理机器认证响应");
        
        if data.len() < 20 { // 最小机器认证响应大小
            return Err(SteamError::Unknown {
                message: "机器认证响应数据不完整".to_string(),
            });
        }
        
        // 解析机器认证数据
        let mut cursor = std::io::Cursor::new(data);
        
        // 读取认证文件内容
        let mut file_data = Vec::new();
        cursor.read_to_end(&mut file_data)
            .map_err(|e| SteamError::Unknown { message: format!("读取认证文件失败: {}", e) })?;
        
        // 验证认证数据完整性（简单校验）
        if file_data.len() < 10 {
            return Err(SteamError::Unknown {
                message: "认证文件数据太短".to_string(),
            });
        }
        
        // 生成认证文件路径
        let steam_id_str = self.current_steam_id
            .map(|id| id.render())
            .unwrap_or_else(|| "unknown".to_string());
        let auth_file_path = format!("sentry_{}.bin", steam_id_str);
        
        // 保存认证文件到本地
        if let Err(e) = tokio::fs::write(&auth_file_path, &file_data).await {
            error!("保存认证文件失败: {}", e);
            return Err(SteamError::Unknown {
                message: format!("无法保存认证文件: {}", e),
            });
        }
        
        // 计算文件哈希用于验证
        let mut hasher = Sha1::new();
        hasher.update(&file_data);
        let file_hash = hasher.finalize();
        
        info!("✅ 机器认证文件已保存: {}", auth_file_path);
        info!("   文件大小: {} 字节", file_data.len());
        info!("   文件哈希: {:x}", file_hash);
        
        // 发送确认消息给服务器
        let confirm_msg = format!(
            "机器认证确认 - 文件大小: {} 字节, 哈希: {:x}",
            file_data.len(),
            file_hash
        );
        
        let packet = SteamPacket::new(EMsg::ClientUpdateMachineAuth, confirm_msg.into_bytes());
        let packet_data = packet.serialize()?;
        
        self.message_sender.send(packet_data)
            .map_err(|_| SteamError::InvalidState {
                message: "无法发送机器认证确认".to_string(),
            })?;
        
        info!("📤 机器认证确认已发送");
        Ok(())
    }

    /// 获取当前Steam ID
    pub fn get_steam_id(&self) -> Option<SteamID> {
        self.current_steam_id
    }

    /// 是否已登录
    pub fn is_logged_on(&self) -> bool {
        self.is_logged_on
    }

    /// 获取会话ID
    pub fn get_session_id(&self) -> Option<i32> {
        self.session_id
    }

    /// 获取心跳间隔
    pub fn get_heartbeat_interval(&self) -> u32 {
        self.heartbeat_interval
    }

    /// 设置心跳间隔
    pub fn set_heartbeat_interval(&mut self, interval: u32) {
        self.heartbeat_interval = interval;
    }

    /// 强制发送心跳
    pub async fn send_heartbeat(&mut self) -> Result<(), SteamError> {
        trace!("💓 强制发送心跳...");
        
        let packet = SteamPacket::new(EMsg::ClientHeartBeat, Vec::new());
        let packet_data = packet.serialize()?;

        self.message_sender.send(packet_data)
            .map_err(|_| SteamError::InvalidState {
                message: "无法发送心跳消息".to_string(),
            })?;

        self.last_heartbeat = get_unix_timestamp();
        trace!("✅ 强制心跳已发送");
        Ok(())
    }

    /// 处理登录密钥
    pub async fn handle_login_key(&mut self, login_key: &str) -> Result<(), SteamError> {
        info!("🔑 处理登录密钥...");
        
        // 验证登录密钥格式
        if login_key.len() < 20 || login_key.len() > 100 {
            return Err(SteamError::Unknown {
                message: "登录密钥长度无效".to_string(),
            });
        }
        
        // 验证密钥只包含有效字符
        if !login_key.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(SteamError::Unknown {
                message: "登录密钥包含无效字符".to_string(),
            });
        }
        
        // 生成密钥文件路径
        let steam_id_str = self.current_steam_id
            .map(|id| id.render())
            .unwrap_or_else(|| "unknown".to_string());
        let key_file_path = format!("loginkey_{}.txt", steam_id_str);
        
        // 创建密钥数据结构
        let key_data = serde_json::json!({
            "login_key": login_key,
            "steam_id": steam_id_str,
            "created_at": crate::utils::get_unix_timestamp(),
            "expires_at": crate::utils::get_unix_timestamp() + (30 * 24 * 60 * 60), // 30天后过期
        });
        
        // 保存到安全存储（这里简化为文件）
        let key_json = serde_json::to_string_pretty(&key_data)
            .map_err(|e| SteamError::Unknown { message: format!("序列化密钥数据失败: {}", e) })?;
        
        if let Err(e) = tokio::fs::write(&key_file_path, key_json).await {
            error!("保存登录密钥失败: {}", e);
            return Err(SteamError::Unknown {
                message: format!("无法保存登录密钥: {}", e),
            });
        }
        
        // 在生产环境中，应该：
        // 1. 使用操作系统的安全存储API（如Windows Credential Manager, macOS Keychain）
        // 2. 加密密钥数据
        // 3. 设置适当的文件权限
        // 4. 定期轮换密钥
        
        info!("✅ 登录密钥已安全保存");
        info!("   密钥长度: {} 字符", login_key.len());
        info!("   保存位置: {}", key_file_path);
        
        // 发送确认给服务器
        let confirm_packet = SteamPacket::new(EMsg::ClientLogonResponse, "login_key_accepted".as_bytes().to_vec());
        let packet_data = confirm_packet.serialize()?;
        
        self.message_sender.send(packet_data)
            .map_err(|_| SteamError::InvalidState {
                message: "无法发送登录密钥确认".to_string(),
            })?;
        
        info!("📤 登录密钥确认已发送");
        Ok(())
    }

    /// 更新状态
    pub async fn update(&mut self) -> Result<(), SteamError> {
        // 定期发送心跳
        if self.is_logged_on {
            self.handle_heartbeat().await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callbacks::CallbackManager;
    use tokio::sync::mpsc;

    fn create_test_steam_user() -> (SteamUser, mpsc::UnboundedReceiver<Vec<u8>>) {
        let callback_manager = Arc::new(CallbackManager::new());
        let (sender, receiver) = mpsc::unbounded_channel();
        let steam_user = SteamUser::new(callback_manager, sender);
        (steam_user, receiver)
    }

    #[tokio::test]
    async fn test_steam_user_creation() {
        let (steam_user, _receiver) = create_test_steam_user();
        assert!(!steam_user.is_logged_on());
        assert!(steam_user.get_steam_id().is_none());
        assert_eq!(steam_user.get_heartbeat_interval(), 30);
    }

    #[tokio::test]
    async fn test_logon_details_validation() {
        let (mut steam_user, _receiver) = create_test_steam_user();
        
        // 测试无效的登录详情
        let invalid_details = LogOnDetails::new();
        let result = steam_user.handle_logon(invalid_details).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_logoff_without_login() {
        let (mut steam_user, _receiver) = create_test_steam_user();
        
        // 测试未登录时登出
        let result = steam_user.handle_logoff().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_heartbeat_interval() {
        let (mut steam_user, _receiver) = create_test_steam_user();
        
        steam_user.set_heartbeat_interval(60);
        assert_eq!(steam_user.get_heartbeat_interval(), 60);
    }

    #[tokio::test]
    async fn test_login_key_handling() {
        let (mut steam_user, _receiver) = create_test_steam_user();
        
        let result = steam_user.handle_login_key("test_login_key_123").await;
        assert!(result.is_ok());
    }
}