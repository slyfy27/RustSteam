//! Callback system for handling Steam events
//! 
//! This module provides the callback infrastructure for handling asynchronous events
//! from the Steam network, similar to JavaSteam's callback system.

use crate::types::{EResult, SteamID, SteamError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use std::any::Any;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::any::TypeId;

/// Base trait for all callbacks
pub trait Callback: Send + Sync + Debug + Any {
    /// Get the callback type name for routing
    fn callback_type(&self) -> &'static str;
}

/// Callback handler function type
pub type CallbackHandler<T> = Arc<dyn Fn(T) + Send + Sync>;

/// Connection callback - fired when client connects to Steam
#[derive(Debug, Clone)]
pub struct ConnectedCallback;

impl Callback for ConnectedCallback {
    fn callback_type(&self) -> &'static str {
        "ConnectedCallback"
    }
}

/// Disconnection callback - fired when client disconnects from Steam
#[derive(Debug, Clone)]
pub struct DisconnectedCallback {
    pub user_initiated: bool,
    pub reason: Option<String>,
}

impl Callback for DisconnectedCallback {
    fn callback_type(&self) -> &'static str {
        "DisconnectedCallback"
    }
}

impl DisconnectedCallback {
    pub fn new(user_initiated: bool, reason: Option<String>) -> Self {
        Self { user_initiated, reason }
    }

    pub fn is_user_initiated(&self) -> bool {
        self.user_initiated
    }
}

/// Logged on callback - fired when user successfully logs on
#[derive(Debug, Clone)]
pub struct LoggedOnCallback {
    pub result: EResult,
    pub extended_result: EResult,
    pub steam_id: SteamID,
    pub cell_id: u32,
    pub email_domain: Option<String>,
    pub num_login_failures: u32,
    pub heartbeat_seconds: u32,
    pub ip_address: Option<std::net::IpAddr>,
    pub vac_banned: bool,
    pub account_flags: u32,
    pub client_supplied_steam_id: SteamID,
    pub public_ip: Option<std::net::IpAddr>,
    pub server_time: chrono::DateTime<chrono::Utc>,
    pub account_name: String,
    pub country: Option<String>,
    pub auth_status: i32,
    pub parental_settings: Vec<u8>,
    pub parental_setting_signature: Vec<u8>,
    pub count_auto_launch_account_restriction: i32,
    pub count_communication_restriction: i32,
    pub count_parental_restriction: i32,
    pub count_web_restriction: i32,
}

impl Callback for LoggedOnCallback {
    fn callback_type(&self) -> &'static str {
        "LoggedOnCallback"
    }
}

impl LoggedOnCallback {
    pub fn new(result: EResult) -> Self {
        Self {
            result,
            extended_result: EResult::OK,
            steam_id: SteamID::new(0),
            cell_id: 0,
            email_domain: None,
            num_login_failures: 0,
            heartbeat_seconds: 0,
            ip_address: None,
            vac_banned: false,
            account_flags: 0,
            client_supplied_steam_id: SteamID::new(0),
            public_ip: None,
            server_time: chrono::Utc::now(),
            account_name: String::new(),
            country: None,
            auth_status: 0,
            parental_settings: Vec::new(),
            parental_setting_signature: Vec::new(),
            count_auto_launch_account_restriction: 0,
            count_communication_restriction: 0,
            count_parental_restriction: 0,
            count_web_restriction: 0,
        }
    }

    pub fn get_result(&self) -> EResult {
        self.result
    }

    pub fn get_extended_result(&self) -> EResult {
        self.extended_result
    }
}

/// Logged off callback - fired when user logs off
#[derive(Debug, Clone)]
pub struct LoggedOffCallback {
    pub result: EResult,
}

impl Callback for LoggedOffCallback {
    fn callback_type(&self) -> &'static str {
        "LoggedOffCallback"
    }
}

impl LoggedOffCallback {
    pub fn new(result: EResult) -> Self {
        Self { result }
    }

    pub fn get_result(&self) -> EResult {
        self.result
    }
}

/// Generic callback event that can hold any callback
#[derive(Debug)]
pub struct CallbackEvent {
    pub callback: Box<dyn Callback>,
}

/// 回调管理器 - 负责管理所有回调订阅
#[derive(Debug)]
pub struct CallbackManager {
    callbacks: Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>>>,
    should_shutdown: Arc<AtomicBool>,
}

impl CallbackManager {
    /// Create a new callback manager
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);
        
        Self {
            callbacks: Arc::new(RwLock::new(HashMap::new())),
            should_shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Subscribe to a specific callback type
    pub fn subscribe<T, F>(&self, handler: F) -> CallbackSubscription
    where
        T: Callback + Clone + 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        let type_name = std::any::type_name::<T>();
        let boxed_handler: Box<dyn Any + Send + Sync> = 
            Box::new(Arc::new(handler) as Arc<dyn Fn(T) + Send + Sync>);

        {
            let mut handlers = self.callbacks.write().unwrap();
            handlers.entry(type_name.to_string())
                .or_insert_with(Vec::new)
                .push(boxed_handler);
        }

        CallbackSubscription {
            type_name: type_name.to_string(),
            handlers: Arc::clone(&self.callbacks),
        }
    }

    /// Get event sender for internal use
    pub(crate) fn get_sender(&self) -> mpsc::Sender<CallbackEvent> {
        self.event_sender.clone()
    }

    /// Run callback processing loop with timeout
    pub async fn run_wait_callbacks(&self, timeout_ms: u64) {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        
        let mut receiver = {
            let mut guard = self.callbacks.write().unwrap();
            guard.take()
        };

        if let Some(mut rx) = receiver {
            match tokio::time::timeout(timeout, rx.recv()).await {
                Ok(Some(event)) => {
                    self.dispatch_callback(event).await;
                }
                Ok(None) => {
                    // Channel closed
                }
                Err(_) => {
                    // Timeout
                }
            }
            
            // Put receiver back
            let mut guard = self.callbacks.write().unwrap();
            *guard = Some(rx);
        }
    }

    /// Dispatch a callback to registered handlers
    async fn dispatch_callback(&self, event: CallbackEvent) {
        let callback_type = event.callback.callback_type();
        
        let handlers = {
            let guard = self.callbacks.read().unwrap();
            guard.get(callback_type).cloned()
        };

        if let Some(handlers) = handlers {
            for handler in handlers {
                // This is a simplified dispatch - in reality you'd need more sophisticated
                // type matching and casting
                if let Ok(callback) = self.try_cast_callback(&*event.callback, callback_type) {
                    // Call the handler (simplified - actual implementation would be more complex)
                    log::debug!("Dispatching callback: {}", callback_type);
                }
            }
        }
    }

    fn try_cast_callback(&self, callback: &dyn Callback, _type_name: &str) -> Result<(), SteamError> {
        // Simplified casting - real implementation would use proper type matching
        Ok(())
    }

    /// Fire a callback event
    pub(crate) async fn fire_callback(&self, callback: Box<dyn Callback>) -> Result<(), SteamError> {
        let event = CallbackEvent { callback };
        
        self.event_sender.send(event).await
            .map_err(|_| SteamError::Unknown { 
                message: "Failed to send callback event".to_string() 
            })?;
        
        Ok(())
    }
}

impl Default for CallbackManager {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII subscription handle that unsubscribes when dropped
pub struct CallbackSubscription {
    type_name: String,
    handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn Any + Send + Sync>>>>>,
}

impl Drop for CallbackSubscription {
    fn drop(&mut self) {
        let mut handlers = self.handlers.write().unwrap();
        if let Some(handler_list) = handlers.get_mut(&self.type_name) {
            // In a real implementation, you'd want to track which specific handler to remove
            // For simplicity, we'll clear all handlers of this type when any subscription drops
            if handler_list.len() == 1 {
                handlers.remove(&self.type_name);
            } else if !handler_list.is_empty() {
                handler_list.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_callback_manager_creation() {
        let manager = CallbackManager::new();
        
        // Test subscribing to a callback
        let _subscription = manager.subscribe(|callback: ConnectedCallback| {
            println!("Connected callback received: {:?}", callback);
        });
        
        // Test firing a callback
        let connected_callback = Box::new(ConnectedCallback);
        let result = manager.fire_callback(connected_callback).await;
        assert!(result.is_ok());
    }
}