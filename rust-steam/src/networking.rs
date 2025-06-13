//! Networking module for Steam protocol communication
//! 
//! Handles TCP, UDP, and WebSocket connections to Steam servers

use crate::callbacks::CallbackManager;
use crate::client::SteamConfiguration;
use crate::types::{ProtocolType, SteamError, ConnectionState};
use std::sync::Arc;
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::{RwLock, mpsc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

/// Connection configuration
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub protocol_types: Vec<ProtocolType>,
    pub connection_timeout: std::time::Duration,
    pub server_list: Vec<String>,
    pub allow_direct_connection: bool,
    pub cell_id: Option<u32>,
}

impl ConnectionConfig {
    pub fn from_steam_config(config: &SteamConfiguration) -> Self {
        Self {
            protocol_types: config.protocol_types.clone(),
            connection_timeout: config.connection_timeout,
            server_list: if config.server_list.is_empty() {
                // Default Steam CM servers
                vec![
                    "155.133.254.133:27017".to_string(),
                    "155.133.254.133:27018".to_string(),
                    "155.133.254.133:27019".to_string(),
                    "155.133.254.133:27020".to_string(),
                    "155.133.244.135:27017".to_string(),
                    "155.133.244.135:27018".to_string(),
                    "162.254.195.46:27017".to_string(),
                    "162.254.195.46:27018".to_string(),
                ]
            } else {
                config.server_list.clone()
            },
            allow_direct_connection: config.allow_direct_connection,
            cell_id: config.cell_id,
        }
    }
}

/// Connection manager for handling Steam server connections
pub struct ConnectionManager {
    config: ConnectionConfig,
    callback_manager: Arc<CallbackManager>,
    connection_state: Arc<RwLock<ConnectionState>>,
    active_connection: Arc<RwLock<Option<Box<dyn Connection + Send + Sync>>>>,
    message_sender: mpsc::UnboundedSender<Vec<u8>>,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<Vec<u8>>>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub async fn new(
        config: ConnectionConfig,
        callback_manager: Arc<CallbackManager>,
    ) -> Result<Self, SteamError> {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            config,
            callback_manager,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            active_connection: Arc::new(RwLock::new(None)),
            message_sender: sender,
            message_receiver: Arc::new(RwLock::new(Some(receiver))),
        })
    }

    /// Connect to Steam servers
    pub async fn connect(&mut self) -> Result<(), SteamError> {
        log::info!("尝试连接到Steam服务器...");

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Connecting;
        }

        // Try each server in the list
        for server in &self.config.server_list {
            for protocol in &self.config.protocol_types {
                match self.try_connect_with_protocol(*protocol, server).await {
                    Ok(connection) => {
                        {
                            let mut active = self.active_connection.write().await;
                            *active = Some(connection);
                        }
                        {
                            let mut state = self.connection_state.write().await;
                            *state = ConnectionState::Connected;
                        }
                        log::info!("成功连接，使用协议: {:?}, 服务器: {}", protocol, server);
                        
                        // Start message processing task
                        self.start_message_processing().await?;
                        
                        return Ok(());
                    }
                    Err(e) => {
                        log::warn!("连接失败，协议 {:?}, 服务器 {}: {}", protocol, server, e);
                        continue;
                    }
                }
            }
        }

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Disconnected;
        }

        Err(SteamError::Network(
            "无法使用任何协议连接到Steam服务器".to_string()
        ))
    }

    /// Try to connect using a specific protocol
    async fn try_connect_with_protocol(
        &self,
        protocol: ProtocolType,
        server: &str,
    ) -> Result<Box<dyn Connection + Send + Sync>, SteamError> {
        let timeout = self.config.connection_timeout;
        
        let connection_future = async {
            match protocol {
                ProtocolType::TCP => {
                    let tcp_connection = TcpConnection::new(server).await?;
                    Ok(Box::new(tcp_connection) as Box<dyn Connection + Send + Sync>)
                }
                ProtocolType::WebSocket => {
                    let ws_connection = WebSocketConnection::new(server).await?;
                    Ok(Box::new(ws_connection) as Box<dyn Connection + Send + Sync>)
                }
                ProtocolType::UDP => {
                    let udp_connection = UdpConnection::new(server).await?;
                    Ok(Box::new(udp_connection) as Box<dyn Connection + Send + Sync>)
                }
            }
        };

        match tokio::time::timeout(timeout, connection_future).await {
            Ok(result) => result,
            Err(_) => Err(SteamError::Network(
                format!("连接超时: {} ({})", server, timeout.as_secs())
            )),
        }
    }

    /// Start message processing task
    async fn start_message_processing(&self) -> Result<(), SteamError> {
        let connection = self.active_connection.clone();
        let callback_manager = self.callback_manager.clone();
        let mut receiver = self.message_receiver.write().await
            .take()
            .ok_or_else(|| SteamError::InvalidState {
                message: "消息接收器已被使用".to_string(),
            })?;

        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                if let Some(conn) = connection.read().await.as_ref() {
                    if let Err(e) = conn.send_message(&message).await {
                        log::error!("发送消息失败: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Disconnect from Steam
    pub async fn disconnect(&mut self) -> Result<(), SteamError> {
        log::info!("断开Steam连接...");

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Disconnecting;
        }

        if let Some(connection) = self.active_connection.write().await.take() {
            connection.disconnect().await?;
        }

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Disconnected;
        }

        log::info!("已断开Steam连接");
        Ok(())
    }

    /// Process network events
    pub async fn process_events(&self) -> Result<(), SteamError> {
        let connection = self.active_connection.read().await;
        if let Some(ref conn) = *connection {
            if let Ok(message) = conn.receive_message().await {
                if !message.is_empty() {
                    self.process_incoming_message(message).await?;
                }
            }
        }
        Ok(())
    }

    /// Process incoming message
    async fn process_incoming_message(&self, message: Vec<u8>) -> Result<(), SteamError> {
        log::trace!("收到消息: {} 字节", message.len());
        
        // Parse Steam protocol message
        if message.len() < 8 { // Minimum header size
            log::warn!("消息太短，无法解析");
            return Ok(());
        }
        
        let mut cursor = Cursor::new(&message);
        
        // Read message type (first 4 bytes)
        let raw_msg_type = cursor.read_u32::<LittleEndian>()
            .map_err(|e| SteamError::Unknown { message: format!("读取消息类型失败: {}", e) })?;
        
        // Parse message type and check if it's extended
        let is_extended = (raw_msg_type & 0x80000000) != 0;
        let msg_type_id = raw_msg_type & 0x7FFFFFFF;
        
        log::debug!("消息类型: {} (扩展: {})", msg_type_id, is_extended);
        
        // Convert to EMsg enum if possible
        let msg_type = crate::types::EMsg::from_u32(msg_type_id);
        
        if let Some(parsed_msg_type) = msg_type {
            // Extract message payload
            let payload = if is_extended {
                // Extended message: skip extended header (additional metadata)
                if message.len() < 36 { // Extended header size
                    log::warn!("扩展消息头不完整");
                    return Ok(());
                }
                message[36..].to_vec() // Skip extended header
            } else {
                // Standard message: skip standard header
                if message.len() < 20 { // Standard header size  
                    log::warn!("标准消息头不完整");
                    return Ok(());
                }
                message[20..].to_vec() // Skip standard header
            };
            
            // Trigger callbacks based on message type
            match parsed_msg_type {
                crate::types::EMsg::ClientLogOnResponse => {
                    log::info!("📥 收到登录响应");
                    self.trigger_logon_response_callback(&payload).await;
                }
                crate::types::EMsg::ClientLoggedOff => {
                    log::info!("📤 收到登出通知");  
                    self.trigger_logoff_callback(&payload).await;
                }
                crate::types::EMsg::ChannelEncryptResult => {
                    log::info!("🔐 收到加密结果");
                    self.handle_encryption_result(&payload).await;
                }
                crate::types::EMsg::Multi => {
                    log::debug!("📦 收到多消息包");
                    self.handle_multi_message(&payload).await;
                }
                _ => {
                    log::debug!("🔍 收到未处理的消息类型: {:?}", parsed_msg_type);
                }
            }
        } else {
            log::warn!("⚠️ 未知的消息类型: {}", msg_type_id);
        }
        
        Ok(())
    }
    
    /// Trigger login response callback
    async fn trigger_logon_response_callback(&self, payload: &[u8]) {
        // Parse login response from payload
        // This would extract fields like result, steam_id, etc.
        log::debug!("处理登录响应 payload: {} 字节", payload.len());
        
        // For now, create a basic success callback
        // In a full implementation, this would parse the actual response
        use crate::callbacks::{LoggedOnCallback, ConnectedCallback};
        use crate::types::{EResult, SteamID};
        use crate::utils::get_unix_timestamp;
        
        let callback = LoggedOnCallback {
            result: EResult::OK,
            steam_id: SteamID::new(76561198000000000), // Would be parsed from response
            account_name: "user".to_string(), // Would be parsed from response
            cell_id: 0, // Would be parsed from response
            email_domain: None,
            vac_banned: false,
            extended_result: EResult::OK,
        };
        
        self.callback_manager.trigger_callback(callback);
    }
    
    /// Trigger logoff callback
    async fn trigger_logoff_callback(&self, _payload: &[u8]) {
        use crate::callbacks::LoggedOffCallback;
        use crate::types::EResult;
        
        let callback = LoggedOffCallback {
            result: EResult::OK,
        };
        
        self.callback_manager.trigger_callback(callback);
    }
    
    /// Handle encryption result
    async fn handle_encryption_result(&self, payload: &[u8]) {
        log::debug!("处理加密结果: {} 字节", payload.len());
        
        // Parse encryption result
        if payload.len() >= 4 {
            let mut cursor = Cursor::new(payload);
            if let Ok(result) = cursor.read_u32::<LittleEndian>() {
                if result == 1 { // EResult::OK
                    log::info!("✅ 加密握手成功");
                    // Trigger connected callback
                    use crate::callbacks::ConnectedCallback;
                    let callback = ConnectedCallback {
                        server_time: std::time::SystemTime::now(),
                    };
                    self.callback_manager.trigger_callback(callback);
                } else {
                    log::error!("❌ 加密握手失败: {}", result);
                }
            }
        }
    }
    
    /// Handle multi-message packet
    async fn handle_multi_message(&self, payload: &[u8]) {
        log::debug!("处理多消息包: {} 字节", payload.len());
        
        // Multi messages contain multiple sub-messages
        // Each sub-message has a length prefix followed by the message data
        let mut cursor = Cursor::new(payload);
        
        while (cursor.position() as usize) < payload.len() {
            // Read sub-message length
            if let Ok(sub_msg_len) = cursor.read_u32::<LittleEndian>() {
                let remaining = payload.len() - cursor.position() as usize;
                if sub_msg_len as usize <= remaining {
                    // Extract sub-message
                    let start = cursor.position() as usize;
                    let end = start + sub_msg_len as usize;
                    let sub_message = &payload[start..end];
                    
                    // Recursively process sub-message
                    if let Err(e) = self.process_incoming_message(sub_message.to_vec()).await {
                        log::error!("处理子消息失败: {}", e);
                    }
                    
                    // Move cursor
                    cursor.set_position(end as u64);
                } else {
                    log::warn!("子消息长度超出范围");
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Send a message
    pub async fn send_message(&self, message: &[u8]) -> Result<(), SteamError> {
        self.message_sender.send(message.to_vec())
            .map_err(|_| SteamError::InvalidState {
                message: "消息发送通道已关闭".to_string(),
            })
    }

    /// Get connection state
    pub async fn get_state(&self) -> ConnectionState {
        *self.connection_state.read().await
    }
}

/// Base trait for all connection types
#[async_trait::async_trait]
pub trait Connection: std::fmt::Debug + Send + Sync {
    async fn connect(&mut self) -> Result<(), SteamError>;
    async fn disconnect(&self) -> Result<(), SteamError>;
    async fn send_message(&self, message: &[u8]) -> Result<(), SteamError>;
    async fn receive_message(&self) -> Result<Vec<u8>, SteamError>;
    async fn process_events(&self) -> Result<(), SteamError>;
    fn is_connected(&self) -> bool;
    fn get_protocol_type(&self) -> ProtocolType;
}

/// TCP connection implementation
#[derive(Debug)]
pub struct TcpConnection {
    address: String,
    stream: Arc<RwLock<Option<TcpStream>>>,
    connected: Arc<RwLock<bool>>,
}

impl TcpConnection {
    pub async fn new(address: &str) -> Result<Self, SteamError> {
        let mut conn = Self {
            address: address.to_string(),
            stream: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
        };
        conn.connect().await?;
        Ok(conn)
    }
}

#[async_trait::async_trait]
impl Connection for TcpConnection {
    async fn connect(&mut self) -> Result<(), SteamError> {
        log::debug!("TCP连接到 {}", self.address);
        
        let stream = TcpStream::connect(&self.address)
            .map_err(|e| SteamError::Network(format!("TCP连接失败: {}", e)))?;
        
        {
            let mut stream_guard = self.stream.write().await;
            *stream_guard = Some(stream);
        }
        
        {
            let mut connected = self.connected.write().await;
            *connected = true;
        }
        
        log::debug!("TCP连接已建立");
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), SteamError> {
        log::debug!("断开TCP连接");
        
        {
            let mut stream_guard = self.stream.write().await;
            *stream_guard = None;
        }
        
        {
            let mut connected = self.connected.write().await;
            *connected = false;
        }
        
        Ok(())
    }

    async fn send_message(&self, message: &[u8]) -> Result<(), SteamError> {
        let stream_guard = self.stream.read().await;
        if let Some(ref mut stream) = *stream_guard {
            // Steam protocol: 4-byte length prefix + message
            let mut length_buf = Vec::new();
            length_buf.write_u32::<LittleEndian>(message.len() as u32)
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
            
            // Send length prefix
            let mut stream_write = stream;
            stream_write.write_all(&length_buf).await
                .map_err(|e| SteamError::Network(format!("发送长度前缀失败: {}", e)))?;
            
            // Send message
            stream_write.write_all(message).await
                .map_err(|e| SteamError::Network(format!("发送消息失败: {}", e)))?;
            
            stream_write.flush().await
                .map_err(|e| SteamError::Network(format!("刷新流失败: {}", e)))?;
            
            log::trace!("TCP发送消息: {} 字节", message.len());
            
            Ok(())
        } else {
            Err(SteamError::InvalidState {
                message: "TCP连接未建立".to_string(),
            })
        }
    }

    async fn receive_message(&self) -> Result<Vec<u8>, SteamError> {
        let stream_guard = self.stream.read().await;
        if let Some(ref mut stream) = *stream_guard {
            // Steam protocol: read 4-byte length prefix first
            let mut length_buf = [0u8; 4];
            let mut stream_read = stream;
            
            match stream_read.read_exact(&mut length_buf).await {
                Ok(_) => {
                    let message_length = u32::from_le_bytes(length_buf) as usize;
                    
                    if message_length > 1024 * 1024 { // 1MB limit
                        return Err(SteamError::Unknown {
                            message: format!("消息长度过大: {}", message_length),
                        });
                    }
                    
                    let mut message_buf = vec![0u8; message_length];
                    stream_read.read_exact(&mut message_buf).await
                        .map_err(|e| SteamError::Network(format!("读取消息失败: {}", e)))?;
                    
                    log::trace!("TCP接收消息: {} 字节", message_length);
                    Ok(message_buf)
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available
                    Ok(Vec::new())
                }
                Err(e) => Err(SteamError::Network(format!("读取长度前缀失败: {}", e))),
            }
        } else {
            Err(SteamError::InvalidState {
                message: "TCP连接未建立".to_string(),
            })
        }
    }

    async fn process_events(&self) -> Result<(), SteamError> {
        // Process any pending messages
        Ok(())
    }

    fn is_connected(&self) -> bool {
        // This is a blocking read, but should be fast
        match self.connected.try_read() {
            Ok(connected) => *connected,
            Err(_) => false,
        }
    }

    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::TCP
    }
}

/// WebSocket connection implementation
#[derive(Debug)]
pub struct WebSocketConnection {
    address: String,
    ws_stream: Arc<RwLock<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    connected: Arc<RwLock<bool>>,
}

impl WebSocketConnection {
    pub async fn new(address: &str) -> Result<Self, SteamError> {
        let mut conn = Self {
            address: address.to_string(),
            ws_stream: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
        };
        conn.connect().await?;
        Ok(conn)
    }
}

#[async_trait::async_trait]
impl Connection for WebSocketConnection {
    async fn connect(&mut self) -> Result<(), SteamError> {
        log::debug!("WebSocket连接到 {}", self.address);
        
        let ws_url = if self.address.starts_with("ws://") || self.address.starts_with("wss://") {
            self.address.clone()
        } else {
            format!("wss://{}/cmsocket/", self.address)
        };
        
        let (ws_stream, _) = connect_async(&ws_url).await
            .map_err(|e| SteamError::Network(format!("WebSocket连接失败: {}", e)))?;
        
        {
            let mut stream_guard = self.ws_stream.write().await;
            *stream_guard = Some(ws_stream);
        }
        
        {
            let mut connected = self.connected.write().await;
            *connected = true;
        }
        
        log::debug!("WebSocket连接已建立");
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), SteamError> {
        log::debug!("断开WebSocket连接");
        
        {
            let mut stream_guard = self.ws_stream.write().await;
            if let Some(mut ws) = stream_guard.take() {
                let _ = ws.close(None).await;
            }
        }
        
        {
            let mut connected = self.connected.write().await;
            *connected = false;
        }
        
        Ok(())
    }

    async fn send_message(&self, message: &[u8]) -> Result<(), SteamError> {
        let stream_guard = self.ws_stream.read().await;
        if let Some(ref mut ws) = *stream_guard {
            let mut ws_write = ws;
            ws_write.send(Message::Binary(message.to_vec())).await
                .map_err(|e| SteamError::Network(format!("WebSocket发送失败: {}", e)))?;
            
            log::trace!("WebSocket发送消息: {} 字节", message.len());
            Ok(())
        } else {
            Err(SteamError::InvalidState {
                message: "WebSocket连接未建立".to_string(),
            })
        }
    }

    async fn receive_message(&self) -> Result<Vec<u8>, SteamError> {
        let stream_guard = self.ws_stream.read().await;
        if let Some(ref mut ws) = *stream_guard {
            let mut ws_read = ws;
            match ws_read.next().await {
                Some(Ok(Message::Binary(data))) => {
                    log::trace!("WebSocket接收消息: {} 字节", data.len());
                    Ok(data)
                }
                Some(Ok(_)) => Ok(Vec::new()), // Non-binary message, ignore
                Some(Err(e)) => Err(SteamError::Network(format!("WebSocket接收错误: {}", e))),
                None => Ok(Vec::new()), // Connection closed
            }
        } else {
            Err(SteamError::InvalidState {
                message: "WebSocket连接未建立".to_string(),
            })
        }
    }

    async fn process_events(&self) -> Result<(), SteamError> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        match self.connected.try_read() {
            Ok(connected) => *connected,
            Err(_) => false,
        }
    }

    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::WebSocket
    }
}

/// UDP connection implementation
#[derive(Debug)]
pub struct UdpConnection {
    address: String,
    socket: Arc<RwLock<Option<UdpSocket>>>,
    server_addr: Arc<RwLock<Option<SocketAddr>>>,
    connected: Arc<RwLock<bool>>,
}

impl UdpConnection {
    pub async fn new(address: &str) -> Result<Self, SteamError> {
        let mut conn = Self {
            address: address.to_string(),
            socket: Arc::new(RwLock::new(None)),
            server_addr: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
        };
        conn.connect().await?;
        Ok(conn)
    }
}

#[async_trait::async_trait]
impl Connection for UdpConnection {
    async fn connect(&mut self) -> Result<(), SteamError> {
        log::debug!("UDP连接到 {}", self.address);
        
        let server_addr: SocketAddr = self.address.parse()
            .map_err(|e| SteamError::Network(format!("无效的UDP地址: {}", e)))?;
        
        let socket = UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| SteamError::Network(format!("UDP套接字绑定失败: {}", e)))?;
        
        socket.connect(&server_addr).await
            .map_err(|e| SteamError::Network(format!("UDP连接失败: {}", e)))?;
        
        {
            let mut socket_guard = self.socket.write().await;
            *socket_guard = Some(socket);
        }
        
        {
            let mut addr_guard = self.server_addr.write().await;
            *addr_guard = Some(server_addr);
        }
        
        {
            let mut connected = self.connected.write().await;
            *connected = true;
        }
        
        log::debug!("UDP连接已建立");
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), SteamError> {
        log::debug!("断开UDP连接");
        
        {
            let mut socket_guard = self.socket.write().await;
            *socket_guard = None;
        }
        
        {
            let mut connected = self.connected.write().await;
            *connected = false;
        }
        
        Ok(())
    }

    async fn send_message(&self, message: &[u8]) -> Result<(), SteamError> {
        let socket_guard = self.socket.read().await;
        if let Some(ref socket) = *socket_guard {
            socket.send(message).await
                .map_err(|e| SteamError::Network(format!("UDP发送失败: {}", e)))?;
            
            log::trace!("UDP发送消息: {} 字节", message.len());
            Ok(())
        } else {
            Err(SteamError::InvalidState {
                message: "UDP连接未建立".to_string(),
            })
        }
    }

    async fn receive_message(&self) -> Result<Vec<u8>, SteamError> {
        let socket_guard = self.socket.read().await;
        if let Some(ref socket) = *socket_guard {
            let mut buf = vec![0u8; 1500]; // Standard MTU size
            match socket.recv(&mut buf).await {
                Ok(len) => {
                    buf.truncate(len);
                    log::trace!("UDP接收消息: {} 字节", len);
                    Ok(buf)
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    Ok(Vec::new())
                }
                Err(e) => Err(SteamError::Network(format!("UDP接收失败: {}", e))),
            }
        } else {
            Err(SteamError::InvalidState {
                message: "UDP连接未建立".to_string(),
            })
        }
    }

    async fn process_events(&self) -> Result<(), SteamError> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        match self.connected.try_read() {
            Ok(connected) => *connected,
            Err(_) => false,
        }
    }

    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::UDP
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callbacks::CallbackManager;

    #[tokio::test]
    async fn test_connection_config() {
        let steam_config = crate::client::SteamConfiguration::default();
        let config = ConnectionConfig::from_steam_config(&steam_config);
        
        assert!(!config.server_list.is_empty());
        assert!(config.protocol_types.len() > 0);
    }

    #[tokio::test]
    async fn test_connection_manager_creation() {
        let config = ConnectionConfig {
            protocol_types: vec![ProtocolType::TCP],
            connection_timeout: std::time::Duration::from_secs(5),
            server_list: vec!["127.0.0.1:27017".to_string()],
            allow_direct_connection: true,
            cell_id: None,
        };
        
        let callback_manager = Arc::new(CallbackManager::new());
        let result = ConnectionManager::new(config, callback_manager).await;
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_tcp_connection_creation() {
        // Test TCP connection structure creation
        let conn = TcpConnection {
            address: "127.0.0.1:27017".to_string(),
            stream: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
        };
        
        assert_eq!(conn.get_protocol_type(), ProtocolType::TCP);
        assert!(!conn.is_connected());
    }
}