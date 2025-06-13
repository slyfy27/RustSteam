//! Main Steam client implementation
//! 
//! This module contains the core SteamClient class that manages connections
//! to Steam servers and coordinates various handlers.

use crate::authentication::{AuthSessionDetails, AuthPollResult, begin_auth_session_via_credentials};
use crate::callbacks::{CallbackManager, ConnectedCallback, DisconnectedCallback, LoggedOnCallback, LoggedOffCallback};
use crate::handlers::steam_user::{SteamUser, LogOnDetails};
use crate::networking::{ConnectionManager, ConnectionConfig};
use crate::types::{EResult, SteamError, ConnectionState, ProtocolType};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::Any;
use tokio::sync::RwLock;

/// Steam client configuration
#[derive(Debug, Clone)]
pub struct SteamConfiguration {
    pub protocol_types: Vec<ProtocolType>,
    pub connection_timeout: std::time::Duration,
    pub server_list_provider: Option<String>,
    pub allow_direct_connection: bool,
    pub cell_id: Option<u32>,
    pub server_list: Vec<String>,
    pub web_api_key: Option<String>,
    pub http_client_name: String,
    pub universe: crate::types::EUniverse,
}

impl Default for SteamConfiguration {
    fn default() -> Self {
        Self {
            protocol_types: vec![ProtocolType::TCP, ProtocolType::WebSocket],
            connection_timeout: std::time::Duration::from_secs(30),
            server_list_provider: None,
            allow_direct_connection: true,
            cell_id: None,
            server_list: Vec::new(),
            web_api_key: None,
            http_client_name: "rust-steam".to_string(),
            universe: crate::types::EUniverse::Public,
        }
    }
}

impl SteamConfiguration {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set protocol types for connections
    pub fn with_protocol_types(mut self, protocol_types: Vec<ProtocolType>) -> Self {
        self.protocol_types = protocol_types;
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set web API key
    pub fn with_web_api_key(mut self, api_key: String) -> Self {
        self.web_api_key = Some(api_key);
        self
    }

    /// Set cell ID
    pub fn with_cell_id(mut self, cell_id: u32) -> Self {
        self.cell_id = Some(cell_id);
        self
    }
}

/// Main Steam client for connecting to and interacting with Steam
pub struct SteamClient {
    configuration: SteamConfiguration,
    callback_manager: Arc<CallbackManager>,
    connection_manager: Arc<RwLock<ConnectionManager>>,
    handlers: Arc<Mutex<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    connection_state: Arc<RwLock<ConnectionState>>,
    is_running: Arc<RwLock<bool>>,
}

impl SteamClient {
    /// Create a new Steam client with default configuration
    pub async fn new() -> Result<Self, SteamError> {
        Self::with_configuration(SteamConfiguration::default()).await
    }

    /// Create a new Steam client with custom configuration
    pub async fn with_configuration(config: SteamConfiguration) -> Result<Self, SteamError> {
        let callback_manager = Arc::new(CallbackManager::new());
        let connection_config = ConnectionConfig::from_steam_config(&config);
        let connection_manager = Arc::new(RwLock::new(
            ConnectionManager::new(connection_config, callback_manager.clone()).await?
        ));

        let mut handlers = HashMap::new();
        
        // Initialize core handlers
        let steam_user = SteamUser::new(callback_manager.clone());
        handlers.insert("SteamUser".to_string(), Box::new(steam_user) as Box<dyn Any + Send + Sync>);

        Ok(Self {
            configuration: config,
            callback_manager,
            connection_manager,
            handlers: Arc::new(Mutex::new(handlers)),
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Get a handler by type
    pub fn get_handler<T: 'static>(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        let type_name = std::any::type_name::<T>();
        let simple_name = type_name.split("::").last().unwrap_or(type_name);
        
        self.handlers.lock().unwrap().get(simple_name).cloned()
    }

    /// Get the callback manager
    pub fn get_callback_manager(&self) -> Arc<CallbackManager> {
        Arc::clone(&self.callback_manager)
    }

    /// Connect to Steam
    pub async fn connect(&self) -> Result<(), SteamError> {
        log::info!("Connecting to Steam...");
        
        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Connecting;
        }

        {
            let mut running = self.is_running.write().await;
            *running = true;
        }

        // Attempt connection
        let mut connection_manager = self.connection_manager.write().await;
        match connection_manager.connect().await {
            Ok(_) => {
                {
                    let mut state = self.connection_state.write().await;
                    *state = ConnectionState::Connected;
                }

                // Fire connected callback
                let callback = Box::new(ConnectedCallback);
                self.callback_manager.fire_callback(callback).await?;

                log::info!("Successfully connected to Steam");
                Ok(())
            }
            Err(e) => {
                {
                    let mut state = self.connection_state.write().await;
                    *state = ConnectionState::Disconnected;
                }

                // Fire disconnected callback
                let callback = Box::new(DisconnectedCallback::new(false, Some(e.to_string())));
                self.callback_manager.fire_callback(callback).await?;

                log::error!("Failed to connect to Steam: {}", e);
                Err(e)
            }
        }
    }

    /// Disconnect from Steam
    pub async fn disconnect(&self) -> Result<(), SteamError> {
        log::info!("Disconnecting from Steam...");

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Disconnecting;
        }

        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        let mut connection_manager = self.connection_manager.write().await;
        connection_manager.disconnect().await?;

        {
            let mut state = self.connection_state.write().await;
            *state = ConnectionState::Disconnected;
        }

        // Fire disconnected callback
        let callback = Box::new(DisconnectedCallback::new(true, None));
        self.callback_manager.fire_callback(callback).await?;

        log::info!("Disconnected from Steam");
        Ok(())
    }

    /// Authenticate with Steam using credentials
    pub async fn authenticate(&self, details: AuthSessionDetails) -> Result<AuthPollResult, SteamError> {
        log::info!("Starting authentication for user: {}", details.username);

        // Begin authentication session
        let mut auth_session = begin_auth_session_via_credentials(details).await?;

        // Poll for result
        let poll_result = auth_session.polling_wait_for_result().await?;

        log::info!("Authentication successful for user: {}", poll_result.account_name);
        Ok(poll_result)
    }

    /// Log on to Steam with authentication tokens
    pub async fn log_on(&self, details: LogOnDetails) -> Result<(), SteamError> {
        let handlers = self.handlers.lock().unwrap();
        if let Some(steam_user) = handlers.get("SteamUser")
            .and_then(|handler| handler.downcast_ref::<SteamUser>()) {
            steam_user.log_on(details).await
        } else {
            Err(SteamError::InvalidState {
                message: "SteamUser handler not found".to_string()
            })
        }
    }

    /// Log off from Steam
    pub async fn log_off(&self) -> Result<(), SteamError> {
        let handlers = self.handlers.lock().unwrap();
        if let Some(steam_user) = handlers.get("SteamUser")
            .and_then(|handler| handler.downcast_ref::<SteamUser>()) {
            steam_user.log_off().await
        } else {
            Err(SteamError::InvalidState {
                message: "SteamUser handler not found".to_string()
            })
        }
    }

    /// Get current connection state
    pub async fn get_connection_state(&self) -> ConnectionState {
        *self.connection_state.read().await
    }

    /// Check if client is connected
    pub async fn is_connected(&self) -> bool {
        matches!(*self.connection_state.read().await, ConnectionState::Connected)
    }

    /// Check if client is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Run the client's main processing loop
    pub async fn run(&self) -> Result<(), SteamError> {
        while self.is_running().await {
            // Process callbacks
            self.callback_manager.run_wait_callbacks(1000).await;
            
            // Process connection events
            let connection_manager = self.connection_manager.read().await;
            connection_manager.process_events().await?;
            
            // Small delay to prevent busy waiting
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        
        Ok(())
    }
}

impl Drop for SteamClient {
    fn drop(&mut self) {
        // Ensure cleanup happens
        log::debug!("SteamClient dropping");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_steam_client_creation() {
        let client = SteamClient::new().await;
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.get_connection_state().await, ConnectionState::Disconnected);
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_steam_configuration() {
        let config = SteamConfiguration::new()
            .with_connection_timeout(std::time::Duration::from_secs(60))
            .with_protocol_types(vec![ProtocolType::TCP])
            .with_web_api_key("test_key".to_string());

        assert_eq!(config.connection_timeout, std::time::Duration::from_secs(60));
        assert_eq!(config.protocol_types, vec![ProtocolType::TCP]);
        assert_eq!(config.web_api_key, Some("test_key".to_string()));
    }
}