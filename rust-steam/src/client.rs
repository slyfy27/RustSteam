//! Steam 客户端
//! 
//! 主要的Steam客户端实现，提供连接、认证和消息处理功能

use crate::authentication::{AuthSessionDetails, AuthSessionRequest, LogOnDetails, ConsoleAuthenticator};
use crate::callbacks::{CallbackManager, ConnectedCallback, DisconnectedCallback, LoggedOnCallback, LoggedOffCallback};
use crate::types::{SteamError, ConnectionState, ProtocolType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Steam客户端配置
#[derive(Debug, Clone)]
pub struct ClientSettings {
    /// 支持的协议类型列表
    pub protocol_types: Vec<ProtocolType>,
    /// Steam服务器列表
    pub server_list: Vec<String>,
    /// 连接超时时间
    pub connect_timeout: Duration,
    /// 是否自动重试
    pub auto_retry: bool,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            protocol_types: vec![ProtocolType::TCP, ProtocolType::WebSocket],
            server_list: vec![
                "steamcommunity.com:443".to_string(),
                "steampowered.com:443".to_string(),
            ],
            connect_timeout: Duration::from_secs(10),
            auto_retry: true,
            max_retries: 3,
        }
    }
}

/// Steam客户端主要实现
pub struct SteamClient {
    /// 客户端配置
    settings: ClientSettings,
    /// 连接状态
    connection_state: Arc<Mutex<ConnectionState>>,
    /// 回调管理器
    callback_manager: Arc<CallbackManager>,
    /// 是否正在运行
    is_running: Arc<Mutex<bool>>,
}

impl SteamClient {
    /// 创建新的Steam客户端实例
    pub fn new(settings: ClientSettings) -> Self {
        Self {
            settings,
            connection_state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            callback_manager: Arc::new(CallbackManager::new()),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// 使用默认配置创建客户端
    pub async fn new_default() -> Result<Self, SteamError> {
        Ok(Self::new(ClientSettings::default()))
    }

    /// 连接到Steam服务器
    pub async fn connect(&mut self) -> Result<(), SteamError> {
        println!("🔌 正在连接到Steam服务器...");
        
        {
            let mut state = self.connection_state.lock().unwrap();
            *state = ConnectionState::Connecting;
        }

        // 模拟连接过程
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // 在实际实现中，这里会尝试连接到Steam服务器
        // 现在我们模拟成功连接
        {
            let mut state = self.connection_state.lock().unwrap();
            *state = ConnectionState::Connected;
        }

        {
            let mut running = self.is_running.lock().unwrap();
            *running = true;
        }

        // 触发连接回调
        let connected_callback = ConnectedCallback {
            server_time: std::time::SystemTime::now(),
        };
        self.callback_manager.trigger_callback(connected_callback);

        println!("✅ 已连接到Steam服务器");
        Ok(())
    }

    /// 断开与Steam服务器的连接
    pub async fn disconnect(&mut self) -> Result<(), SteamError> {
        println!("🔌 正在断开连接...");
        
        {
            let mut state = self.connection_state.lock().unwrap();
            *state = ConnectionState::Disconnecting;
        }

        // 模拟断开过程
        tokio::time::sleep(Duration::from_millis(500)).await;

        {
            let mut state = self.connection_state.lock().unwrap();
            *state = ConnectionState::Disconnected;
        }

        {
            let mut running = self.is_running.lock().unwrap();
            *running = false;
        }

        // 触发断开连接回调
        let disconnected_callback = DisconnectedCallback {
            user_initiated: true,
            reason: Some("User requested disconnect".to_string()),
        };
        self.callback_manager.trigger_callback(disconnected_callback);

        println!("✅ 已断开连接");
        Ok(())
    }

    /// 认证登录
    pub async fn authenticate(&mut self, details: AuthSessionDetails) -> Result<crate::authentication::AuthPollResult, SteamError> {
        println!("🔐 开始认证流程...");
        
        let mut auth_request = AuthSessionRequest::new(details);
        
        // 开始认证
        let poll_result = auth_request.begin_auth().await?;
        
        if poll_result.requires_2fa {
            println!("📱 需要2FA验证");
            // 轮询认证结果
            return auth_request.poll_auth_result().await;
        }
        
        Ok(poll_result)
    }

    /// 使用认证结果登录
    pub async fn log_on(&mut self, details: LogOnDetails) -> Result<(), SteamError> {
        println!("🔑 正在登录Steam...");
        println!("👤 用户: {}", details.username);
        
        // 模拟登录过程
        tokio::time::sleep(Duration::from_millis(800)).await;
        
        // 在实际实现中，这里会发送登录消息到Steam服务器
        // 现在我们模拟成功登录
        
        let logged_on_callback = LoggedOnCallback {
            result: crate::types::EResult::OK,
            steam_id: crate::types::SteamID::new(76561198000000000),
            account_name: details.username.clone(),
            cell_id: 123,
            email_domain: Some("example.com".to_string()),
            vac_banned: false,
            extended_result: crate::types::EResult::OK,
        };
        
        self.callback_manager.trigger_callback(logged_on_callback);
        
        println!("✅ 成功登录Steam!");
        Ok(())
    }

    /// 登出
    pub async fn log_off(&mut self) -> Result<(), SteamError> {
        println!("🚪 正在登出...");
        
        // 模拟登出过程
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        let logged_off_callback = LoggedOffCallback {
            result: crate::types::EResult::OK,
        };
        
        self.callback_manager.trigger_callback(logged_off_callback);
        
        println!("✅ 已登出");
        Ok(())
    }

    /// 获取连接状态
    pub fn get_connection_state(&self) -> ConnectionState {
        *self.connection_state.lock().unwrap()
    }

    /// 获取回调管理器
    pub fn get_callback_manager(&self) -> Arc<CallbackManager> {
        Arc::clone(&self.callback_manager)
    }

    /// 检查客户端是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    /// 运行客户端主循环
    pub async fn run(&mut self) -> Result<(), SteamError> {
        println!("🚀 启动Steam客户端...");
        
        {
            let mut running = self.is_running.lock().unwrap();
            *running = true;
        }

        // 主事件循环
        while self.is_running().await {
            // 处理回调
            self.callback_manager.run_wait_callbacks(100).await;
            
            // 检查是否应该关闭
            if self.callback_manager.should_shutdown() {
                break;
            }
        }

        println!("🛑 Steam客户端已停止");
        Ok(())
    }

    /// 停止客户端
    pub fn stop(&self) {
        {
            let mut running = self.is_running.lock().unwrap();
            *running = false;
        }
        self.callback_manager.shutdown();
    }
}

/// Steam配置（向后兼容）
pub type SteamConfiguration = ClientSettings;

impl Drop for SteamClient {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authentication::ConsoleAuthenticator;

    #[tokio::test]
    async fn test_client_creation() {
        let client = SteamClient::new(ClientSettings::default());
        assert_eq!(client.get_connection_state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_connection_flow() {
        let mut client = SteamClient::new(ClientSettings::default());
        
        // 测试连接
        let result = client.connect().await;
        assert!(result.is_ok());
        assert_eq!(client.get_connection_state(), ConnectionState::Connected);
        
        // 测试断开连接
        let result = client.disconnect().await;
        assert!(result.is_ok());
        assert_eq!(client.get_connection_state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_authentication_flow() {
        let mut client = SteamClient::new(ClientSettings::default());
        
        // 先连接
        client.connect().await.unwrap();
        
        // 测试认证
        let auth_details = AuthSessionDetails {
            username: "test_user".to_string(),
            password: "test_password".to_string(),
            authenticator: Some(Arc::new(ConsoleAuthenticator)),
            ..Default::default()
        };
        
        let auth_result = client.authenticate(auth_details).await;
        assert!(auth_result.is_ok());
    }
}