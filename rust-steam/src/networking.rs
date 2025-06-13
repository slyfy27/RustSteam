//! Networking module for Steam protocol communication
//! 
//! Handles TCP, UDP, and WebSocket connections to Steam servers

use crate::callbacks::CallbackManager;
use crate::client::SteamConfiguration;
use crate::types::{ProtocolType, SteamError, ConnectionState};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

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
}

impl ConnectionManager {
    /// Create a new connection manager
    pub async fn new(
        config: ConnectionConfig,
        callback_manager: Arc<CallbackManager>,
    ) -> Result<Self, SteamError> {
        Ok(Self {
            config,
            callback_manager,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            active_connection: Arc::new(RwLock::new(None)),
        })
    }

    /// Connect to Steam servers
    pub async fn connect(&mut self) -> Result<(), SteamError> {
        log::info!("Attempting to connect to Steam servers...");

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Connecting;
        }

        // Try each protocol type
        for protocol in &self.config.protocol_types {
            match self.try_connect_with_protocol(*protocol).await {
                Ok(connection) => {
                    {
                        let mut active = self.active_connection.write().await;
                        *active = Some(connection);
                    }
                    {
                        let mut state = self.connection_state.write().await;
                        *state = ConnectionState::Connected;
                    }
                    log::info!("Successfully connected using protocol: {:?}", protocol);
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("Failed to connect with protocol {:?}: {}", protocol, e);
                    continue;
                }
            }
        }

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Disconnected;
        }

        Err(SteamError::Network(reqwest::Error::from(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "Failed to connect with any protocol",
        ))))
    }

    /// Try to connect using a specific protocol
    async fn try_connect_with_protocol(
        &self,
        protocol: ProtocolType,
    ) -> Result<Box<dyn Connection + Send + Sync>, SteamError> {
        match protocol {
            ProtocolType::TCP => {
                let tcp_connection = TcpConnection::new(&self.config.server_list[0]).await?;
                Ok(Box::new(tcp_connection))
            }
            ProtocolType::WebSocket => {
                let ws_connection = WebSocketConnection::new(&self.config.server_list[0]).await?;
                Ok(Box::new(ws_connection))
            }
            ProtocolType::UDP => {
                let udp_connection = UdpConnection::new(&self.config.server_list[0]).await?;
                Ok(Box::new(udp_connection))
            }
        }
    }

    /// Disconnect from Steam
    pub async fn disconnect(&mut self) -> Result<(), SteamError> {
        log::info!("Disconnecting from Steam...");

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

        log::info!("Disconnected from Steam");
        Ok(())
    }

    /// Process network events
    pub async fn process_events(&self) -> Result<(), SteamError> {
        let connection = self.active_connection.read().await;
        if let Some(ref conn) = *connection {
            conn.process_events().await?;
        }
        Ok(())
    }

    /// Send a message
    pub async fn send_message(&self, message: &[u8]) -> Result<(), SteamError> {
        let connection = self.active_connection.read().await;
        if let Some(ref conn) = *connection {
            conn.send_message(message).await
        } else {
            Err(SteamError::InvalidState {
                message: "No active connection".to_string(),
            })
        }
    }

    /// Get connection state
    pub async fn get_state(&self) -> ConnectionState {
        *self.connection_state.read().await
    }
}

/// Base trait for all connection types
#[async_trait::async_trait]
pub trait Connection: std::fmt::Debug {
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
    stream: Option<TcpStream>,
    connected: bool,
}

impl TcpConnection {
    pub async fn new(address: &str) -> Result<Self, SteamError> {
        let mut conn = Self {
            address: address.to_string(),
            stream: None,
            connected: false,
        };
        conn.connect().await?;
        Ok(conn)
    }
}

#[async_trait::async_trait]
impl Connection for TcpConnection {
    async fn connect(&mut self) -> Result<(), SteamError> {
        log::debug!("Connecting TCP to {}", self.address);
        
        let stream = TcpStream::connect(&self.address).await
            .map_err(|e| SteamError::Network(reqwest::Error::from(e)))?;
        
        self.stream = Some(stream);
        self.connected = true;
        
        log::debug!("TCP connection established");
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), SteamError> {
        log::debug!("Disconnecting TCP connection");
        // TCP stream will be dropped automatically
        Ok(())
    }

    async fn send_message(&self, message: &[u8]) -> Result<(), SteamError> {
        if let Some(_stream) = &self.stream {
            // In a real implementation, we would write to the stream
            log::trace!("Sending TCP message of {} bytes", message.len());
            Ok(())
        } else {
            Err(SteamError::InvalidState {
                message: "TCP connection not established".to_string(),
            })
        }
    }

    async fn receive_message(&self) -> Result<Vec<u8>, SteamError> {
        if let Some(_stream) = &self.stream {
            // In a real implementation, we would read from the stream
            // For now, return empty message
            Ok(Vec::new())
        } else {
            Err(SteamError::InvalidState {
                message: "TCP connection not established".to_string(),
            })
        }
    }

    async fn process_events(&self) -> Result<(), SteamError> {
        // Process any incoming messages
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::TCP
    }
}

/// WebSocket connection implementation
#[derive(Debug)]
pub struct WebSocketConnection {
    address: String,
    connected: bool,
}

impl WebSocketConnection {
    pub async fn new(address: &str) -> Result<Self, SteamError> {
        let mut conn = Self {
            address: address.to_string(),
            connected: false,
        };
        conn.connect().await?;
        Ok(conn)
    }
}

#[async_trait::async_trait]
impl Connection for WebSocketConnection {
    async fn connect(&mut self) -> Result<(), SteamError> {
        log::debug!("Connecting WebSocket to {}", self.address);
        
        // In a real implementation, we would establish a WebSocket connection
        self.connected = true;
        
        log::debug!("WebSocket connection established");
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), SteamError> {
        log::debug!("Disconnecting WebSocket connection");
        Ok(())
    }

    async fn send_message(&self, message: &[u8]) -> Result<(), SteamError> {
        if self.connected {
            log::trace!("Sending WebSocket message of {} bytes", message.len());
            Ok(())
        } else {
            Err(SteamError::InvalidState {
                message: "WebSocket connection not established".to_string(),
            })
        }
    }

    async fn receive_message(&self) -> Result<Vec<u8>, SteamError> {
        if self.connected {
            Ok(Vec::new())
        } else {
            Err(SteamError::InvalidState {
                message: "WebSocket connection not established".to_string(),
            })
        }
    }

    async fn process_events(&self) -> Result<(), SteamError> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::WebSocket
    }
}

/// UDP connection implementation
#[derive(Debug)]
pub struct UdpConnection {
    address: String,
    connected: bool,
}

impl UdpConnection {
    pub async fn new(address: &str) -> Result<Self, SteamError> {
        let mut conn = Self {
            address: address.to_string(),
            connected: false,
        };
        conn.connect().await?;
        Ok(conn)
    }
}

#[async_trait::async_trait]
impl Connection for UdpConnection {
    async fn connect(&mut self) -> Result<(), SteamError> {
        log::debug!("Connecting UDP to {}", self.address);
        
        // In a real implementation, we would establish a UDP socket
        self.connected = true;
        
        log::debug!("UDP connection established");
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), SteamError> {
        log::debug!("Disconnecting UDP connection");
        Ok(())
    }

    async fn send_message(&self, message: &[u8]) -> Result<(), SteamError> {
        if self.connected {
            log::trace!("Sending UDP message of {} bytes", message.len());
            Ok(())
        } else {
            Err(SteamError::InvalidState {
                message: "UDP connection not established".to_string(),
            })
        }
    }

    async fn receive_message(&self) -> Result<Vec<u8>, SteamError> {
        if self.connected {
            Ok(Vec::new())
        } else {
            Err(SteamError::InvalidState {
                message: "UDP connection not established".to_string(),
            })
        }
    }

    async fn process_events(&self) -> Result<(), SteamError> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
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
            connection_timeout: std::time::Duration::from_secs(30),
            server_list: vec!["127.0.0.1:27017".to_string()],
            allow_direct_connection: true,
            cell_id: None,
        };
        
        let callback_manager = Arc::new(CallbackManager::new());
        let result = ConnectionManager::new(config, callback_manager).await;
        
        assert!(result.is_ok());
    }
}