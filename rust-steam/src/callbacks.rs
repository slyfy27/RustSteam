//! Steam 回调系统
//! 
//! 提供类型安全的事件驱动架构来处理Steam协议消息

use crate::types::{EResult, SteamID, SteamError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::time::SystemTime;

/// 基础回调trait
pub trait Callback: Send + Sync + Debug + Any {
    /// 获取回调类型名称
    fn callback_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// 连接回调 - 当连接到Steam时触发
#[derive(Debug, Clone)]
pub struct ConnectedCallback {
    /// 服务器时间
    pub server_time: SystemTime,
}

impl Callback for ConnectedCallback {}

/// 断开连接回调 - 当与Steam断开连接时触发
#[derive(Debug, Clone)]
pub struct DisconnectedCallback {
    /// 是否是用户主动断开
    pub user_initiated: bool,
    /// 断开原因
    pub reason: Option<String>,
}

impl DisconnectedCallback {
    pub fn is_user_initiated(&self) -> bool {
        self.user_initiated
    }
}

impl Callback for DisconnectedCallback {}

/// 登录回调 - 当登录成功时触发
#[derive(Debug, Clone)]
pub struct LoggedOnCallback {
    /// 登录结果
    pub result: EResult,
    /// Steam ID
    pub steam_id: SteamID,
    /// 账户名
    pub account_name: String,
    /// Cell ID
    pub cell_id: u32,
    /// 邮箱域名
    pub email_domain: Option<String>,
    /// 是否被VAC封禁
    pub vac_banned: bool,
    /// 扩展结果
    pub extended_result: EResult,
}

impl LoggedOnCallback {
    pub fn get_result(&self) -> EResult {
        self.result
    }
    
    pub fn get_extended_result(&self) -> EResult {
        self.extended_result
    }
}

impl Callback for LoggedOnCallback {}

/// 登出回调 - 当登出时触发
#[derive(Debug, Clone)]
pub struct LoggedOffCallback {
    /// 登出结果
    pub result: EResult,
}

impl LoggedOffCallback {
    pub fn get_result(&self) -> EResult {
        self.result
    }
}

impl Callback for LoggedOffCallback {}

/// 回调事件包装器
#[derive(Debug)]
pub struct CallbackEvent {
    pub callback: Box<dyn Callback>,
}

/// 回调管理器 - 负责管理所有回调订阅
pub struct CallbackManager {
    callbacks: Arc<RwLock<HashMap<TypeId, Vec<CallbackHandler>>>>,
    should_shutdown: Arc<AtomicBool>,
    next_id: Arc<AtomicUsize>,
}

impl std::fmt::Debug for CallbackManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackManager")
            .field("should_shutdown", &self.should_shutdown)
            .field("next_id", &self.next_id)
            .field("callbacks", &"<callback handlers>")
            .finish()
    }
}

/// 回调处理器
type CallbackHandler = Box<dyn Fn(&dyn Any) + Send + Sync>;

impl CallbackManager {
    /// 创建新的回调管理器
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(RwLock::new(HashMap::new())),
            should_shutdown: Arc::new(AtomicBool::new(false)),
            next_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 订阅特定类型的回调
    pub fn subscribe<T, F>(&self, handler: F) -> CallbackSubscription
    where
        T: Callback + 'static,
        F: Fn(&T) + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let subscription_id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        let boxed_handler: CallbackHandler = Box::new(move |any_callback| {
            if let Some(typed_callback) = any_callback.downcast_ref::<T>() {
                handler(typed_callback);
            }
        });

        {
            let mut callbacks = self.callbacks.write().unwrap();
            callbacks.entry(type_id)
                .or_insert_with(Vec::new)
                .push(boxed_handler);
        }

        CallbackSubscription {
            type_id,
            subscription_id,
            callbacks: Arc::clone(&self.callbacks),
        }
    }

    /// 运行并等待回调
    pub async fn run_wait_callbacks(&self, timeout_ms: u64) {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        
        // 在实际实现中，这里会从消息队列中接收事件
        // 现在我们只是等待指定的时间
        tokio::time::sleep(timeout).await;
    }

    /// 触发回调事件
    pub fn trigger_callback<T>(&self, callback: T) 
    where
        T: Callback + 'static,
    {
        let type_id = TypeId::of::<T>();
        
        let handlers = {
            let callbacks = self.callbacks.read().unwrap();
            // 避免cloned()，直接收集引用
            callbacks.get(&type_id)
                .map(|h| h.len())
                .unwrap_or(0)
        };

        if handlers > 0 {
            let callback_any: &dyn Any = &callback;
            let callbacks_guard = self.callbacks.read().unwrap();
            if let Some(handler_list) = callbacks_guard.get(&type_id) {
                for handler in handler_list {
                    handler(callback_any);
                }
            }
        }
    }

    /// 发送回调事件（异步版本）
    pub async fn send_callback(&self, event: CallbackEvent) -> Result<(), SteamError> {
        // 在实际实现中，这里会将事件放入队列
        // 现在我们直接处理事件
        println!("📨 处理回调事件: {}", event.callback.callback_type());
        Ok(())
    }
    
    /// 停止回调管理器
    pub fn shutdown(&self) {
        self.should_shutdown.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    
    /// 检查是否应该关闭
    pub fn should_shutdown(&self) -> bool {
        self.should_shutdown.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// RAII订阅句柄，在drop时自动取消订阅
pub struct CallbackSubscription {
    type_id: TypeId,
    subscription_id: usize,
    callbacks: Arc<RwLock<HashMap<TypeId, Vec<CallbackHandler>>>>,
}

impl Drop for CallbackSubscription {
    fn drop(&mut self) {
        // 在实际实现中，这里会移除特定的处理器
        // 现在我们简化处理，清空该类型的所有处理器
        let mut callbacks = self.callbacks.write().unwrap();
        if let Some(handlers) = callbacks.get_mut(&self.type_id) {
            handlers.clear();
        }
    }
}

impl Default for CallbackManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_callback_subscription() {
        let manager = CallbackManager::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        let _subscription = manager.subscribe(move |_: &ConnectedCallback| {
            called_clone.store(true, Ordering::SeqCst);
        });

        let callback = ConnectedCallback {
            server_time: SystemTime::now(),
        };

        manager.trigger_callback(callback);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_multiple_subscriptions() {
        let manager = CallbackManager::new();
        let counter = Arc::new(AtomicUsize::new(0));
        
        let counter1 = Arc::clone(&counter);
        let _sub1 = manager.subscribe(move |_: &ConnectedCallback| {
            counter1.fetch_add(1, Ordering::SeqCst);
        });
        
        let counter2 = Arc::clone(&counter);
        let _sub2 = manager.subscribe(move |_: &ConnectedCallback| {
            counter2.fetch_add(1, Ordering::SeqCst);
        });

        let callback = ConnectedCallback {
            server_time: SystemTime::now(),
        };

        manager.trigger_callback(callback);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_async_operations() {
        let manager = CallbackManager::new();
        
        // 测试异步运行
        let start = std::time::Instant::now();
        manager.run_wait_callbacks(100).await;
        let elapsed = start.elapsed();
        
        assert!(elapsed >= std::time::Duration::from_millis(90));
    }
}