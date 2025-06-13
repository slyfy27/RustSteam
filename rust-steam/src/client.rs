//! Steam 客户端
//! 
//! 主要的Steam客户端实现，处理连接、认证和消息处理

use crate::authentication::{AuthSessionDetails, AuthPollResult, LogOnDetails, SteamAuthSession};
use crate::callbacks::{CallbackManager, ConnectedCallback, DisconnectedCallback, LoggedOnCallback, LoggedOffCallback};
use crate::networking::{ConnectionConfig, ConnectionManager};
use crate::protocol::{SteamPacket, ClientLogon, ChannelEncryptRequest, EMsg};
use crate::types::{ConnectionState, ProtocolType, SteamError, SteamID};
use crate::utils::get_unix_timestamp;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Steam客户端设置
#[derive(Debug, Clone)]
pub struct ClientSettings {
    /// 网络连接超时时间
    pub connection_timeout: Duration,
    /// 自动重连
    pub auto_reconnect: bool,
    /// 重连延迟
    pub reconnect_delay: Duration,
    /// 最大重连次数
    pub max_reconnect_attempts: u32,
    /// 心跳间隔
    pub heartbeat_interval: Duration,
    /// 日志级别
    pub log_level: String,
}

impl ClientSettings {
    pub fn new() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            auto_reconnect: true,
            reconnect_delay: Duration::from_secs(5),
            max_reconnect_attempts: 5,
            heartbeat_interval: Duration::from_secs(10),
            log_level: "info".to_string(),
        }
    }
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Steam配置
#[derive(Debug, Clone)]
pub struct SteamConfiguration {
    /// 支持的协议类型
    pub protocol_types: Vec<ProtocolType>,
    /// 连接超时时间
    pub connection_timeout: Duration,
    /// 服务器列表
    pub server_list: Vec<String>,
    /// 是否允许直接连接
    pub allow_direct_connection: bool,
    /// Cell ID
    pub cell_id: Option<u32>,
    /// 客户端版本
    pub client_version: String,
}

impl SteamConfiguration {
    pub fn new() -> Self {
        Self {
            protocol_types: vec![ProtocolType::TCP, ProtocolType::WebSocket],
            connection_timeout: Duration::from_secs(30),
            server_list: Vec::new(), // 使用默认服务器列表
            allow_direct_connection: true,
            cell_id: None,
            client_version: "1.0.0".to_string(),
        }
    }
}

impl Default for SteamConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

/// Steam客户端
pub struct SteamClient {
    settings: ClientSettings,
    config: SteamConfiguration,
    callback_manager: Arc<CallbackManager>,
    connection_manager: Arc<RwLock<Option<ConnectionManager>>>,
    connection_state: Arc<RwLock<ConnectionState>>,
    steam_id: Arc<RwLock<Option<SteamID>>>,
    session_id: Arc<RwLock<Option<i32>>>,
    reconnect_attempts: Arc<RwLock<u32>>,
    is_logged_on: Arc<RwLock<bool>>,
    should_shutdown: Arc<RwLock<bool>>,
}

impl SteamClient {
    /// 创建新的Steam客户端
    pub fn new(settings: ClientSettings) -> Self {
        Self {
            settings,
            config: SteamConfiguration::default(),
            callback_manager: Arc::new(CallbackManager::new()),
            connection_manager: Arc::new(RwLock::new(None)),
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            steam_id: Arc::new(RwLock::new(None)),
            session_id: Arc::new(RwLock::new(None)),
            reconnect_attempts: Arc::new(RwLock::new(0)),
            is_logged_on: Arc::new(RwLock::new(false)),
            should_shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// 连接到Steam
    pub async fn connect(&mut self) -> Result<(), SteamError> {
        log::info!("🔗 开始连接到Steam...");

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Connecting;
        }

        // 创建连接配置
        let connection_config = ConnectionConfig::from_steam_config(&self.config);
        
        // 创建连接管理器
        let mut connection_manager = ConnectionManager::new(
            connection_config,
            self.callback_manager.clone(),
        ).await?;

        // 尝试连接
        match connection_manager.connect().await {
            Ok(_) => {
                {
                    let mut state = self.connection_state.write().await;
                    *state = ConnectionState::Connected;
                }

                // 存储连接管理器
                {
                    let mut manager = self.connection_manager.write().await;
                    *manager = Some(connection_manager);
                }

                // 重置重连计数
                {
                    let mut attempts = self.reconnect_attempts.write().await;
                    *attempts = 0;
                }

                // 触发连接回调
                let callback = ConnectedCallback {
                    timestamp: get_unix_timestamp(),
                };
                self.callback_manager.trigger(&callback);

                log::info!("✅ 已成功连接到Steam");
                
                // 开始加密握手
                self.begin_encryption_handshake().await?;
                
                Ok(())
            }
            Err(e) => {
                {
                    let mut state = self.connection_state.write().await;
                    *state = ConnectionState::Disconnected;
                }

                log::error!("❌ 连接失败: {}", e);

                // 如果启用了自动重连，尝试重连
                if self.settings.auto_reconnect {
                    self.schedule_reconnect().await?;
                }

                Err(e)
            }
        }
    }

    /// 开始加密握手
    async fn begin_encryption_handshake(&self) -> Result<(), SteamError> {
        log::info!("🔐 开始加密握手...");

        let encrypt_request = ChannelEncryptRequest::new();
        let request_data = encrypt_request.serialize()?;
        let packet = SteamPacket::new(EMsg::ChannelEncryptRequest, request_data);
        let packet_data = packet.serialize()?;

        if let Some(ref manager) = *self.connection_manager.read().await {
            manager.send_message(&packet_data).await?;
            log::info!("✅ 加密请求已发送");
        }

        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<(), SteamError> {
        log::info!("🔌 断开Steam连接...");

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Disconnecting;
        }

        // 如果已登录，先登出
        if *self.is_logged_on.read().await {
            let _ = self.log_off().await;
        }

        // 断开网络连接
        if let Some(ref mut manager) = *self.connection_manager.write().await {
            manager.disconnect().await?;
        }

        {
            let mut manager = self.connection_manager.write().await;
            *manager = None;
        }

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Disconnected;
        }

        // 触发断开连接回调
        let callback = DisconnectedCallback {
            timestamp: get_unix_timestamp(),
            reason: "用户请求断开连接".to_string(),
            was_logged_on: *self.is_logged_on.read().await,
        };
        self.callback_manager.trigger(&callback);

        log::info!("✅ 已断开Steam连接");
        Ok(())
    }

    /// 使用认证会话进行登录
    pub async fn authenticate(&mut self, details: AuthSessionDetails) -> Result<AuthPollResult, SteamError> {
        log::info!("🔐 开始Steam认证: {}", details.username);

        if self.get_connection_state() != ConnectionState::Connected {
            return Err(SteamError::InvalidState {
                message: "必须先连接到Steam".to_string(),
            });
        }

        // 创建认证会话
        let mut auth_session = SteamAuthSession::new(details).await?;
        
        // 开始认证
        auth_session.start_authentication().await?;
        
        // 轮询认证结果
        let auth_result = auth_session.poll_authentication().await?;
        
        if auth_result.success {
            // 使用认证结果进行登录
            if let Some(ref access_token) = auth_result.access_token {
                let logon_details = LogOnDetails::with_access_token(access_token.clone());
                self.log_on(logon_details).await?;
            }
        }

        Ok(auth_result)
    }

    /// 使用登录详情进行登录
    pub async fn log_on(&mut self, details: LogOnDetails) -> Result<(), SteamError> {
        log::info!("📝 开始登录Steam...");

        if self.get_connection_state() != ConnectionState::Connected {
            return Err(SteamError::InvalidState {
                message: "必须先连接到Steam".to_string(),
            });
        }

        // 创建登录消息
        let username = details.username.clone().unwrap_or_else(|| "anonymous".to_string());
        let access_token = details.access_token.clone().unwrap_or_default();
        
        let logon_msg = ClientLogon::new_with_tokens(username.clone(), access_token);
        let logon_data = logon_msg.serialize()?;
        
        // 使用Steam ID创建扩展消息包
        let steam_id = details.steam_id.unwrap_or_else(|| SteamID::new(0));
        let session_id = rand::random::<i32>();
        
        let packet = SteamPacket::new_extended(
            EMsg::ClientLogon,
            steam_id,
            session_id,
            logon_data,
        );
        
        let packet_data = packet.serialize()?;

        // 发送登录消息
        if let Some(ref manager) = *self.connection_manager.read().await {
            manager.send_message(&packet_data).await?;
            
            // 存储会话信息
            {
                let mut stored_steam_id = self.steam_id.write().await;
                *stored_steam_id = Some(steam_id);
            }
            {
                let mut stored_session_id = self.session_id.write().await;
                *stored_session_id = Some(session_id);
            }
            {
                let mut logged_on = self.is_logged_on.write().await;
                *logged_on = true;
            }

            // 触发登录成功回调
            let callback = LoggedOnCallback {
                result: crate::types::EResult::OK,
                steam_id,
                account_name: username,
                cell_id: 0,
                email_domain: String::new(),
                parental_settings: Vec::new(),
                count_loginfailures_to_migrate: 0,
                count_disconnects_to_migrate: 0,
                ogs_data_report_time_window: 0,
                client_supplied_steam_id: steam_id.id,
                ip_country_code: String::new(),
                vanity_url: String::new(),
                out_of_game_heartbeat_seconds: 30,
                in_game_heartbeat_seconds: 30,
                public_ip: Vec::new(),
                server_time: get_unix_timestamp() as u32,
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

            log::info!("✅ 登录请求已发送，Steam ID: {}", steam_id.render());
        }

        Ok(())
    }

    /// 登出
    pub async fn log_off(&mut self) -> Result<(), SteamError> {
        log::info!("👋 开始登出Steam...");

        if !*self.is_logged_on.read().await {
            return Err(SteamError::InvalidState {
                message: "当前未登录".to_string(),
            });
        }

        // 创建登出消息包
        let packet = SteamPacket::new(EMsg::ClientLogOff, Vec::new());
        let packet_data = packet.serialize()?;

        // 发送登出消息
        if let Some(ref manager) = *self.connection_manager.read().await {
            manager.send_message(&packet_data).await?;

            {
                let mut logged_on = self.is_logged_on.write().await;
                *logged_on = false;
            }

            // 触发登出回调
            let callback = LoggedOffCallback {
                result: crate::types::EResult::OK,
                timestamp: get_unix_timestamp(),
            };
            self.callback_manager.trigger(&callback);

            log::info!("✅ 已登出Steam");
        }

        Ok(())
    }

    /// 处理网络事件
    pub async fn process_events(&self) -> Result<(), SteamError> {
        if let Some(ref manager) = *self.connection_manager.read().await {
            manager.process_events().await?;
        }
        Ok(())
    }

    /// 运行客户端主循环
    pub async fn run(&mut self) -> Result<(), SteamError> {
        log::info!("🚀 启动Steam客户端主循环...");

        while !*self.should_shutdown.read().await {
            // 处理网络事件
            if let Err(e) = self.process_events().await {
                log::error!("处理网络事件时出错: {}", e);
                
                // 如果连接断开且启用了自动重连
                if self.get_connection_state() == ConnectionState::Disconnected && self.settings.auto_reconnect {
                    let _ = self.schedule_reconnect().await;
                }
            }

            // 处理回调
            self.callback_manager.process_callbacks().await;

            // 短暂休眠以避免CPU过度使用
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        log::info!("✅ Steam客户端主循环已结束");
        Ok(())
    }

    /// 安排重连
    async fn schedule_reconnect(&mut self) -> Result<(), SteamError> {
        let mut attempts = self.reconnect_attempts.write().await;
        *attempts += 1;

        if *attempts > self.settings.max_reconnect_attempts {
            log::error!("❌ 达到最大重连次数 ({}), 停止重连", self.settings.max_reconnect_attempts);
            return Ok(());
        }

        log::info!("🔄 计划在 {:?} 后进行第 {} 次重连...", self.settings.reconnect_delay, *attempts);
        
        let delay = self.settings.reconnect_delay;
        let client_ptr = self as *mut SteamClient;
        
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            
            // 安全地访问client
            unsafe {
                if let Some(client) = client_ptr.as_mut() {
                    if let Err(e) = client.connect().await {
                        log::error!("重连失败: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// 关闭客户端
    pub async fn shutdown(&mut self) -> Result<(), SteamError> {
        log::info!("🛑 关闭Steam客户端...");

        {
            let mut shutdown = self.should_shutdown.write().await;
            *shutdown = true;
        }

        // 断开连接
        self.disconnect().await?;

        // 关闭回调管理器
        self.callback_manager.shutdown().await;

        log::info!("✅ Steam客户端已关闭");
        Ok(())
    }

    /// 获取连接状态
    pub fn get_connection_state(&self) -> ConnectionState {
        match self.connection_state.try_read() {
            Ok(state) => *state,
            Err(_) => ConnectionState::Disconnected,
        }
    }

    /// 获取Steam ID
    pub async fn get_steam_id(&self) -> Option<SteamID> {
        *self.steam_id.read().await
    }

    /// 是否已登录
    pub async fn is_logged_on(&self) -> bool {
        *self.is_logged_on.read().await
    }

    /// 获取回调管理器
    pub fn get_callback_manager(&self) -> Arc<CallbackManager> {
        self.callback_manager.clone()
    }

    /// 发送消息
    pub async fn send_message(&self, message: &[u8]) -> Result<(), SteamError> {
        if let Some(ref manager) = *self.connection_manager.read().await {
            manager.send_message(message).await
        } else {
            Err(SteamError::InvalidState {
                message: "没有活动的连接".to_string(),
            })
        }
    }

    /// 设置配置
    pub fn set_configuration(&mut self, config: SteamConfiguration) {
        self.config = config;
    }

    /// 获取配置
    pub fn get_configuration(&self) -> &SteamConfiguration {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_settings() {
        let settings = ClientSettings::new();
        assert_eq!(settings.connection_timeout, Duration::from_secs(30));
        assert!(settings.auto_reconnect);
        assert_eq!(settings.max_reconnect_attempts, 5);
    }

    #[test]
    fn test_steam_configuration() {
        let config = SteamConfiguration::new();
        assert!(!config.protocol_types.is_empty());
        assert!(config.allow_direct_connection);
        assert_eq!(config.client_version, "1.0.0");
    }

    #[tokio::test]
    async fn test_client_creation() {
        let settings = ClientSettings::default();
        let client = SteamClient::new(settings);
        assert_eq!(client.get_connection_state(), ConnectionState::Disconnected);
        assert!(!client.is_logged_on().await);
    }

    #[tokio::test]
    async fn test_client_steam_id() {
        let settings = ClientSettings::default();
        let client = SteamClient::new(settings);
        assert!(client.get_steam_id().await.is_none());
    }

    #[test]
    fn test_logon_details() {
        let details = LogOnDetails::with_credentials("test".to_string(), "pass".to_string());
        assert_eq!(details.username, Some("test".to_string()));
        assert_eq!(details.password, Some("pass".to_string()));
    }
}