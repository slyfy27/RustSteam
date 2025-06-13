//! Steam认证系统
//! 
//! 处理Steam账户认证，包括用户名/密码认证、2FA认证和JWT令牌认证

use crate::types::{SteamError, SteamID, EResult};
use crate::utils::{get_unix_timestamp, generate_machine_id, base64_encode, base64_decode};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt;

/// 认证会话详情
#[derive(Debug, Clone)]
pub struct AuthSessionDetails {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// Steam Guard认证码
    pub guard_code: Option<String>,
    /// 2FA验证器
    pub authenticator: Option<Arc<dyn Authenticator>>,
    /// 设备名称
    pub device_friendly_name: String,
    /// 平台类型
    pub platform_type: EAuthTokenPlatformType,
    /// 语言
    pub language: String,
    /// 是否持久化会话
    pub persist_login: bool,
    /// 网站ID
    pub website_id: String,
    /// 客户端ID
    pub client_id: String,
}

impl Default for AuthSessionDetails {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            guard_code: None,
            authenticator: None,
            device_friendly_name: "Rust Steam Client".to_string(),
            platform_type: EAuthTokenPlatformType::MobileApp,
            language: "chinese".to_string(),
            persist_login: true,
            website_id: "Client".to_string(),
            client_id: "DE45CD61".to_string(),
        }
    }
}

/// 认证轮询结果
#[derive(Debug, Clone)]
pub struct AuthPollResult {
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
    /// 访问令牌
    pub access_token: Option<String>,
    /// 刷新令牌
    pub refresh_token: Option<String>,
    /// 新的guard数据
    pub new_guard_data: Option<String>,
    /// 会话ID
    pub session_id: Option<String>,
    /// Steam ID
    pub steam_id: Option<SteamID>,
    /// 账户名
    pub account_name: Option<String>,
    /// 需要2FA
    pub requires_2fa: bool,
    /// 需要Email验证
    pub requires_email_verification: bool,
    /// 需要验证码
    pub requires_captcha: bool,
    /// 验证码GID
    pub captcha_gid: Option<String>,
}

impl Default for AuthPollResult {
    fn default() -> Self {
        Self {
            success: false,
            error_message: None,
            access_token: None,
            refresh_token: None,
            new_guard_data: None,
            session_id: None,
            steam_id: None,
            account_name: None,
            requires_2fa: false,
            requires_email_verification: false,
            requires_captcha: false,
            captcha_gid: None,
        }
    }
}

/// 2FA认证器接口
#[async_trait]
pub trait Authenticator: Send + Sync + fmt::Debug {
    /// 获取2FA认证码
    async fn get_device_code(&self, previous_code: Option<String>) -> Result<String, SteamError>;
    
    /// 获取Email验证码
    async fn get_email_code(&self, email_domain: String, previous_code: Option<String>) -> Result<String, SteamError>;
    
    /// 处理验证码
    async fn get_captcha_code(&self, captcha_gid: String) -> Result<String, SteamError>;
}

/// 控制台认证器
#[derive(Debug)]
pub struct ConsoleAuthenticator;

#[async_trait]
impl Authenticator for ConsoleAuthenticator {
    async fn get_device_code(&self, previous_code: Option<String>) -> Result<String, SteamError> {
        if let Some(prev) = previous_code {
            println!("❌ 之前的验证码 '{}' 不正确", prev);
        }
        
        println!("📱 请输入Steam Guard手机验证码:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        
        Ok(input.trim().to_string())
    }
    
    async fn get_email_code(&self, email_domain: String, previous_code: Option<String>) -> Result<String, SteamError> {
        if let Some(prev) = previous_code {
            println!("❌ 之前的验证码 '{}' 不正确", prev);
        }
        
        println!("📧 请检查您的邮箱 (*@{}) 并输入验证码:", email_domain);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        
        Ok(input.trim().to_string())
    }
    
    async fn get_captcha_code(&self, captcha_gid: String) -> Result<String, SteamError> {
        println!("🔐 验证码GID: {}", captcha_gid);
        println!("📷 请访问 https://steamcommunity.com/public/captcha.php?gid={} 查看验证码", captcha_gid);
        println!("请输入验证码:");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        
        Ok(input.trim().to_string())
    }
}

/// Steam认证会话
pub struct SteamAuthSession {
    client: Client,
    client_id: String,
    request_id: String,
    session_id: String,
    details: AuthSessionDetails,
    poll_interval: u64,
    max_poll_attempts: u32,
}

impl SteamAuthSession {
    /// 创建新的认证会话
    pub async fn new(details: AuthSessionDetails) -> Result<Self, SteamError> {
        Ok(Self {
            client: Client::new(),
            client_id: details.client_id.clone(),
            request_id: uuid::Uuid::new_v4().to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            details,
            poll_interval: 1000, // 1秒
            max_poll_attempts: 60, // 最多60次轮询
        })
    }

    /// 开始认证过程
    pub async fn start_authentication(&mut self) -> Result<(), SteamError> {
        log::info!("🔐 开始认证用户: {}", self.details.username);
        
        // 1. 获取RSA密钥
        let rsa_key = self.get_rsa_key().await?;
        
        // 2. 加密密码
        let encrypted_password = self.encrypt_password(&self.details.password, &rsa_key)?;
        
        // 3. 开始认证
        self.begin_auth_session(&encrypted_password, &rsa_key).await?;
        
        log::info!("✅ 认证会话已开始，请求ID: {}", self.request_id);
        Ok(())
    }

    /// 轮询认证结果
    pub async fn poll_authentication(&mut self) -> Result<AuthPollResult, SteamError> {
        let mut attempts = 0;
        let mut previous_2fa_code: Option<String> = None;
        let mut previous_email_code: Option<String> = None;
        
        while attempts < self.max_poll_attempts {
            attempts += 1;
            log::debug!("🔄 认证轮询尝试 {}/{}", attempts, self.max_poll_attempts);
            
            let poll_result = self.poll_auth_session().await?;
            
            match poll_result.status.as_str() {
                "success" => {
                    log::info!("✅ 认证成功");
                    return Ok(AuthPollResult {
                        success: true,
                        access_token: poll_result.access_token,
                        refresh_token: poll_result.refresh_token,
                        steam_id: poll_result.steam_id.map(|id| SteamID::new(id)),
                        account_name: Some(self.details.username.clone()),
                        ..Default::default()
                    });
                }
                "need_2fa" => {
                    log::info!("📱 需要2FA验证");
                    if let Some(ref authenticator) = self.details.authenticator {
                        let code = authenticator.get_device_code(previous_2fa_code.clone()).await?;
                        previous_2fa_code = Some(code.clone());
                        self.submit_2fa_code(&code).await?;
                    } else {
                        return Ok(AuthPollResult {
                            success: false,
                            error_message: Some("需要2FA验证但未提供认证器".to_string()),
                            requires_2fa: true,
                            ..Default::default()
                        });
                    }
                }
                "need_email" => {
                    log::info!("📧 需要邮箱验证");
                    if let Some(ref authenticator) = self.details.authenticator {
                        let email_domain = poll_result.email_domain.unwrap_or_else(|| "unknown".to_string());
                        let code = authenticator.get_email_code(email_domain, previous_email_code.clone()).await?;
                        previous_email_code = Some(code.clone());
                        self.submit_email_code(&code).await?;
                    } else {
                        return Ok(AuthPollResult {
                            success: false,
                            error_message: Some("需要邮箱验证但未提供认证器".to_string()),
                            requires_email_verification: true,
                            ..Default::default()
                        });
                    }
                }
                "need_captcha" => {
                    log::info!("🔐 需要验证码");
                    if let Some(ref authenticator) = self.details.authenticator {
                        if let Some(captcha_gid) = &poll_result.captcha_gid {
                            let code = authenticator.get_captcha_code(captcha_gid.clone()).await?;
                            self.submit_captcha_code(&code, captcha_gid).await?;
                        }
                    } else {
                        return Ok(AuthPollResult {
                            success: false,
                            error_message: Some("需要验证码但未提供认证器".to_string()),
                            requires_captcha: true,
                            captcha_gid: poll_result.captcha_gid,
                            ..Default::default()
                        });
                    }
                }
                "failed" => {
                    log::error!("❌ 认证失败: {}", poll_result.error_message.unwrap_or_else(|| "未知错误".to_string()));
                    return Ok(AuthPollResult {
                        success: false,
                        error_message: poll_result.error_message,
                        ..Default::default()
                    });
                }
                _ => {
                    log::debug!("⏳ 认证状态: {}, 继续轮询...", poll_result.status);
                }
            }
            
            // 等待一段时间再次轮询
            tokio::time::sleep(std::time::Duration::from_millis(self.poll_interval)).await;
        }
        
        Err(SteamError::Unknown {
            message: "认证超时".to_string(),
        })
    }

    /// 获取RSA密钥
    async fn get_rsa_key(&self) -> Result<RSAKey, SteamError> {
        let url = "https://steamcommunity.com/login/getrsakey/";
        let mut params = HashMap::new();
        params.insert("username", self.details.username.as_str());
        params.insert("donotcache", &get_unix_timestamp().to_string());

        let response = self.client
            .post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| SteamError::Network(e.to_string()))?;

        let rsa_response: RSAKeyResponse = response
            .json()
            .await
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;

        if !rsa_response.success {
            return Err(SteamError::Unknown {
                message: "无法获取RSA密钥".to_string(),
            });
        }

        Ok(RSAKey {
            modulus: rsa_response.publickey_mod,
            exponent: rsa_response.publickey_exp,
            timestamp: rsa_response.timestamp,
        })
    }

    /// 加密密码
    fn encrypt_password(&self, password: &str, rsa_key: &RSAKey) -> Result<String, SteamError> {
        // 简化的RSA加密实现
        // 在实际应用中，这里应该使用真正的RSA加密
        let combined = format!("{}:{}:{}", password, rsa_key.modulus, rsa_key.exponent);
        Ok(base64_encode(combined.as_bytes()))
    }

    /// 开始认证会话
    async fn begin_auth_session(&mut self, encrypted_password: &str, rsa_key: &RSAKey) -> Result<(), SteamError> {
        let url = "https://steamcommunity.com/login/dologin/";
        let mut params = HashMap::new();
        params.insert("username", self.details.username.as_str());
        params.insert("password", encrypted_password);
        params.insert("twofactorcode", self.details.guard_code.as_deref().unwrap_or(""));
        params.insert("captchagid", "-1");
        params.insert("captcha_text", "");
        params.insert("emailauth", "");
        params.insert("loginfriendlyname", &self.details.device_friendly_name);
        params.insert("oauth_client_id", &self.details.client_id);
        params.insert("oauth_scope", "read_limited write_limited");
        params.insert("rsatimestamp", &rsa_key.timestamp);
        params.insert("remember_login", if self.details.persist_login { "true" } else { "false" });
        params.insert("donotcache", &get_unix_timestamp().to_string());

        let response = self.client
            .post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| SteamError::Network(e.to_string()))?;

        let login_response: LoginResponse = response
            .json()
            .await
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;

        if !login_response.success {
            return Err(SteamError::Unknown {
                message: login_response.message.unwrap_or_else(|| "登录失败".to_string()),
            });
        }

        // 更新会话状态
        if let Some(oauth) = login_response.oauth {
            self.request_id = oauth.oauth_token.unwrap_or_else(|| self.request_id.clone());
        }

        Ok(())
    }

    /// 轮询认证状态
    async fn poll_auth_session(&self) -> Result<PollAuthResponse, SteamError> {
        let url = "https://steamcommunity.com/login/gettoken/";
        let mut params = HashMap::new();
        params.insert("oauth_token", self.request_id.as_str());

        let response = self.client
            .post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| SteamError::Network(e.to_string()))?;

        let poll_response: PollAuthResponse = response
            .json()
            .await
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;

        Ok(poll_response)
    }

    /// 提交2FA代码
    async fn submit_2fa_code(&self, code: &str) -> Result<(), SteamError> {
        let url = "https://steamcommunity.com/login/dologin/";
        let mut params = HashMap::new();
        params.insert("twofactorcode", code);
        params.insert("oauth_token", &self.request_id);

        let _response = self.client
            .post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| SteamError::Network(e.to_string()))?;

        Ok(())
    }

    /// 提交邮箱验证码
    async fn submit_email_code(&self, code: &str) -> Result<(), SteamError> {
        let url = "https://steamcommunity.com/login/dologin/";
        let mut params = HashMap::new();
        params.insert("emailauth", code);
        params.insert("oauth_token", &self.request_id);

        let _response = self.client
            .post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| SteamError::Network(e.to_string()))?;

        Ok(())
    }

    /// 提交验证码
    async fn submit_captcha_code(&self, code: &str, captcha_gid: &str) -> Result<(), SteamError> {
        let url = "https://steamcommunity.com/login/dologin/";
        let mut params = HashMap::new();
        params.insert("captcha_text", code);
        params.insert("captchagid", captcha_gid);
        params.insert("oauth_token", &self.request_id);

        let _response = self.client
            .post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| SteamError::Network(e.to_string()))?;

        Ok(())
    }
}

/// 登录详情
#[derive(Debug, Clone)]
pub struct LogOnDetails {
    pub username: Option<String>,
    pub password: Option<String>,
    pub login_key: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub account_name: Option<String>,
    pub steam_id: Option<SteamID>,
    pub client_instance_id: u64,
    pub machine_id: Vec<u8>,
    pub should_remember_password: bool,
    pub supports_rate_limit_response: bool,
    pub login_id: u32,
    pub two_factor_code: Option<String>,
    pub auth_code: Option<String>,
    pub sentinel_file_hash: Option<Vec<u8>>,
    pub machine_name: Option<String>,
    pub client_language: String,
    pub client_os_type: u32,
    pub anon_user_target_account_name: Option<String>,
    pub request_steam2_ticket: bool,
}

impl LogOnDetails {
    pub fn new() -> Self {
        Self {
            username: None,
            password: None,
            login_key: None,
            access_token: None,
            refresh_token: None,
            account_name: None,
            steam_id: None,
            client_instance_id: get_unix_timestamp() as u64,
            machine_id: generate_machine_id(),
            should_remember_password: true,
            supports_rate_limit_response: true,
            login_id: rand::random(),
            two_factor_code: None,
            auth_code: None,
            sentinel_file_hash: None,
            machine_name: Some("Rust Steam Client".to_string()),
            client_language: "chinese".to_string(),
            client_os_type: 16, // Windows
            anon_user_target_account_name: None,
            request_steam2_ticket: false,
        }
    }

    pub fn with_credentials(username: String, password: String) -> Self {
        let mut details = Self::new();
        details.username = Some(username);
        details.password = Some(password);
        details
    }

    pub fn with_access_token(access_token: String) -> Self {
        let mut details = Self::new();
        details.access_token = Some(access_token);
        details
    }
}

impl Default for LogOnDetails {
    fn default() -> Self {
        Self::new()
    }
}

// Steam认证会话Guard类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EAuthSessionGuardType {
    None = 0,
    EmailCode = 1,
    DeviceCode = 2,
    DeviceConfirmation = 3,
    EmailConfirmation = 4,
    MachineToken = 5,
}

// 认证令牌平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EAuthTokenPlatformType {
    Unknown = 0,
    SteamClient = 1,
    WebBrowser = 2,
    MobileApp = 3,
}

// 平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EPlatformType {
    Unknown = 0,
    Win32 = 1,
    Win64 = 2,
    Linux32 = 3,
    Linux64 = 4,
    MacOSX = 5,
    PS3 = 6,
    PS4 = 7,
    PS5 = 8,
    XBox360 = 9,
    XBoxOne = 10,
    AndroidPhone = 11,
    AndroidTablet = 12,
    IPhone = 13,
    IPad = 14,
    AndroidTV = 15,
    VirtualReality = 16,
    Steam = 17,
}

// 内部数据结构
#[derive(Debug, Deserialize)]
struct RSAKeyResponse {
    success: bool,
    publickey_mod: String,
    publickey_exp: String,
    timestamp: String,
}

#[derive(Debug)]
struct RSAKey {
    modulus: String,
    exponent: String,
    timestamp: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    success: bool,
    message: Option<String>,
    oauth: Option<OAuthResponse>,
}

#[derive(Debug, Deserialize)]
struct OAuthResponse {
    oauth_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PollAuthResponse {
    status: String,
    error_message: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    steam_id: Option<u64>,
    email_domain: Option<String>,
    captcha_gid: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_session_details() {
        let details = AuthSessionDetails::default();
        assert_eq!(details.device_friendly_name, "Rust Steam Client");
        assert_eq!(details.language, "chinese");
        assert!(details.persist_login);
    }

    #[test]
    fn test_logon_details() {
        let details = LogOnDetails::with_credentials("testuser".to_string(), "testpass".to_string());
        assert_eq!(details.username, Some("testuser".to_string()));
        assert_eq!(details.password, Some("testpass".to_string()));
        assert!(details.should_remember_password);
    }

    #[test]
    fn test_console_authenticator() {
        let auth = ConsoleAuthenticator;
        assert_eq!(format!("{:?}", auth), "ConsoleAuthenticator");
    }

    #[tokio::test]
    async fn test_auth_session_creation() {
        let details = AuthSessionDetails::default();
        let session = SteamAuthSession::new(details).await;
        assert!(session.is_ok());
    }
}