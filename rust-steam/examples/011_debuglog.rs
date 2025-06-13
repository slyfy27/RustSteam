// 示例011: 调试日志
// 
// 此示例演示如何启用和使用调试日志功能
// 可以帮助诊断连接和认证问题

use rust_steam::prelude::*;
use tokio::time::{sleep, Duration};
use std::env;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// 自定义调试日志监听器
#[derive(Debug)]
struct DebugLogListener {
    messages: Arc<Mutex<VecDeque<String>>>,
    max_messages: usize,
}

impl DebugLogListener {
    fn new(max_messages: usize) -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new())),
            max_messages,
        }
    }
    
    fn log(&self, level: &str, module: &str, message: &str) {
        let formatted_message = format!("[{}] {}: {}", level, module, message);
        println!("🐛 {}", formatted_message);
        
        let mut messages = self.messages.lock().unwrap();
        messages.push_back(formatted_message);
        
        // 保持最大消息数量
        while messages.len() > self.max_messages {
            messages.pop_front();
        }
    }
    
    fn get_recent_messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().iter().cloned().collect()
    }
    
    fn clear(&self) {
        self.messages.lock().unwrap().clear();
    }
}

#[tokio::main]
async fn main() -> Result<(), SteamError> {
    println!("=== JavaSteam样例011: 调试日志 ===\n");
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("使用方法: {} <用户名> <密码>", args[0]);
        return Ok(());
    }
    
    let username = &args[1];
    let password = &args[2];
    
    println!("🐛 启用调试日志模式...");
    println!("   注意：调试模式会产生大量输出信息");
    
    // 创建调试日志监听器
    let debug_listener = Arc::new(DebugLogListener::new(100));
    
    // 启用详细日志记录
    debug_listener.log("INFO", "MAIN", "开始Steam调试日志示例");
    debug_listener.log("DEBUG", "CONFIG", "初始化客户端配置");
    
    // 创建Steam客户端
    let mut client = SteamClient::new(ClientSettings::default());
    
    debug_listener.log("DEBUG", "CLIENT", "Steam客户端已创建");
    
    // 获取回调管理器
    let callback_manager = client.get_callback_manager();
    
    // 订阅连接事件
    let debug_listener_connected = Arc::clone(&debug_listener);
    let _connected_sub = callback_manager.subscribe(move |callback: &ConnectedCallback| {
        debug_listener_connected.log("INFO", "CONNECTION", "已连接到Steam服务器");
        debug_listener_connected.log("DEBUG", "CONNECTION", &format!("服务器时间: {:?}", callback.server_time));
        
        println!("✅ 已连接到Steam服务器!");
        println!("   服务器时间: {:?}", callback.server_time);
    });
    
    // 订阅断开连接事件
    let debug_listener_disconnected = Arc::clone(&debug_listener);
    let _disconnected_sub = callback_manager.subscribe(move |callback: &DisconnectedCallback| {
        debug_listener_disconnected.log("WARN", "CONNECTION", "与Steam服务器的连接已断开");
        if let Some(reason) = &callback.reason {
            debug_listener_disconnected.log("DEBUG", "CONNECTION", &format!("断开原因: {}", reason));
        }
        debug_listener_disconnected.log("DEBUG", "CONNECTION", &format!("用户主动断开: {}", callback.user_initiated));
        
        println!("❌ 与Steam服务器的连接已断开");
        if let Some(reason) = &callback.reason {
            println!("   原因: {}", reason);
        }
        println!("   用户主动断开: {}", callback.user_initiated);
    });
    
    // 订阅登录成功事件
    let debug_listener_logged_on = Arc::clone(&debug_listener);
    let _logged_on_sub = callback_manager.subscribe(move |callback: &LoggedOnCallback| {
        debug_listener_logged_on.log("INFO", "AUTH", &format!("登录结果: {:?}", callback.result));
        debug_listener_logged_on.log("DEBUG", "AUTH", &format!("Steam ID: {}", callback.steam_id.render()));
        debug_listener_logged_on.log("DEBUG", "AUTH", &format!("账户名: {}", callback.account_name));
        debug_listener_logged_on.log("DEBUG", "AUTH", &format!("Cell ID: {}", callback.cell_id));
        debug_listener_logged_on.log("DEBUG", "AUTH", &format!("VAC封禁状态: {}", callback.vac_banned));
        
        match callback.result {
            EResult::OK => {
                println!("🎉 登录成功！");
                println!("   Steam ID: {}", callback.steam_id.render());
                println!("   账户名: {}", callback.account_name);
                println!("   Cell ID: {}", callback.cell_id);
                if let Some(domain) = &callback.email_domain {
                    println!("   邮箱域名: {}", domain);
                }
                println!("   VAC封禁状态: {}", callback.vac_banned);
            }
            _ => {
                println!("❌ 登录失败: {:?}", callback.result);
                if callback.extended_result != EResult::OK {
                    println!("   扩展错误: {:?}", callback.extended_result);
                }
            }
        }
    });
    
    // 订阅登出事件
    let debug_listener_logged_off = Arc::clone(&debug_listener);
    let _logged_off_sub = callback_manager.subscribe(move |callback: &LoggedOffCallback| {
        debug_listener_logged_off.log("INFO", "AUTH", &format!("登出结果: {:?}", callback.result));
        println!("👋 已从Steam登出: {:?}", callback.result);
    });
    
    // 显示调试信息
    println!("\n📊 === 调试配置 ===");
    println!("调试日志: 已启用");
    println!("最大日志缓存: 100条");
    println!("日志级别: DEBUG, INFO, WARN, ERROR");
    
    // 连接到Steam
    println!("\n🌐 正在连接Steam服务器...");
    debug_listener.log("DEBUG", "CONNECTION", "开始连接流程");
    
    client.connect().await?;
    
    debug_listener.log("DEBUG", "CONNECTION", "连接请求已发送");
    
    // 等待连接完成
    sleep(Duration::from_secs(1)).await;
    
    // 创建认证详情
    debug_listener.log("DEBUG", "AUTH", "准备认证详情");
    let auth_details = AuthSessionDetails {
        username: username.to_string(),
        password: password.to_string(),
        persistent_session: false,
        guard_data: None,
        authenticator: Some(Arc::new(ConsoleAuthenticator)),
        device_friendly_name: "Rust Steam Debug Client".to_string(),
    };
    
    debug_listener.log("INFO", "AUTH", &format!("开始认证用户: {}", username));
    println!("🔑 执行认证会话...");
    
    // 执行认证
    match client.authenticate(auth_details).await {
        Ok(auth_result) => {
            debug_listener.log("INFO", "AUTH", "认证成功");
            debug_listener.log("DEBUG", "AUTH", &format!("获得账户名: {}", auth_result.account_name));
            debug_listener.log("DEBUG", "AUTH", &format!("需要2FA: {}", auth_result.requires_2fa));
            
            println!("✅ 认证成功！");
            
            // 创建登录详情
            let login_details = LogOnDetails {
                username: auth_result.account_name.clone(),
                access_token: Some(auth_result.refresh_token.clone()),
                login_id: 149,
                client_os_type: 16,
                should_remember_password: false,
                machine_name: "Rust Steam Debug Client".to_string(),
                ..Default::default()
            };
            
            debug_listener.log("DEBUG", "AUTH", "准备登录详情");
            debug_listener.log("DEBUG", "AUTH", &format!("登录ID: {}", login_details.login_id));
            
            println!("📝 登录到Steam...");
            client.log_on(login_details).await?;
            
            // 等待登录完成
            sleep(Duration::from_secs(3)).await;
            
            // 显示调试统计
            println!("\n📈 === 调试统计 ===");
            let recent_messages = debug_listener.get_recent_messages();
            println!("总日志条数: {}", recent_messages.len());
            
            // 按日志级别分类统计
            let mut debug_count = 0;
            let mut info_count = 0;
            let mut warn_count = 0;
            let mut error_count = 0;
            
            for message in &recent_messages {
                if message.contains("[DEBUG]") {
                    debug_count += 1;
                } else if message.contains("[INFO]") {
                    info_count += 1;
                } else if message.contains("[WARN]") {
                    warn_count += 1;
                } else if message.contains("[ERROR]") {
                    error_count += 1;
                }
            }
            
            println!("DEBUG级别: {} 条", debug_count);
            println!("INFO级别: {} 条", info_count);
            println!("WARN级别: {} 条", warn_count);
            println!("ERROR级别: {} 条", error_count);
            
            // 显示最近的日志消息
            println!("\n📋 === 最近的调试日志 (最后5条) ===");
            for message in recent_messages.iter().rev().take(5).rev() {
                println!("  {}", message);
            }
            
            println!("\n💡 === 调试技巧 ===");
            println!("1. 使用DEBUG级别查看详细的内部状态");
            println!("2. INFO级别显示重要的操作流程");
            println!("3. WARN级别显示可能的问题");
            println!("4. ERROR级别显示严重错误");
            println!("5. 可以将日志保存到文件以便后续分析");
            
            println!("\n   为了演示，我们将在3秒后登出...");
            sleep(Duration::from_secs(3)).await;
            
            debug_listener.log("INFO", "AUTH", "开始登出流程");
            
            // 登出
            client.log_off().await?;
            
            debug_listener.log("DEBUG", "AUTH", "登出请求已发送");
            
            // 等待登出完成
            sleep(Duration::from_secs(1)).await;
        }
        Err(e) => {
            debug_listener.log("ERROR", "AUTH", &format!("认证失败: {}", e));
            println!("❌ 认证失败: {}", e);
        }
    }
    
    // 断开连接
    println!("🔌 断开与Steam服务器的连接...");
    debug_listener.log("INFO", "CONNECTION", "开始断开连接");
    
    client.disconnect().await?;
    
    debug_listener.log("DEBUG", "CONNECTION", "断开连接完成");
    debug_listener.log("INFO", "MAIN", "调试日志示例结束");
    
    // 最终统计
    println!("\n📊 === 最终调试统计 ===");
    let final_messages = debug_listener.get_recent_messages();
    println!("会话期间总日志条数: {}", final_messages.len());
    
    println!("\n✨ 调试日志示例完成！");
    
    Ok(())
}