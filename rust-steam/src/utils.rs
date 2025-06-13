//! Utility functions and helpers for the rust-steam library

use crate::types::{SteamError, SteamID};
use std::time::{SystemTime, UNIX_EPOCH};

/// Convert a Steam community ID to SteamID
pub fn community_id_to_steam_id(community_id: u64) -> Result<SteamID, SteamError> {
    if community_id < 76561197960265728 {
        return Err(SteamError::InvalidState {
            message: "Invalid community ID".to_string(),
        });
    }
    
    Ok(SteamID::new(community_id))
}

/// Convert a Steam account ID to SteamID  
pub fn account_id_to_steam_id(account_id: u32) -> SteamID {
    SteamID::from_account_id(account_id)
}

/// Get current Unix timestamp
pub fn get_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Generate a random machine ID
pub fn generate_machine_id() -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..20).map(|_| rng.gen::<u8>()).collect()
}

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert hex string to bytes
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, SteamError> {
    hex::decode(hex_str).map_err(|e| SteamError::Unknown {
        message: format!("Failed to decode hex: {}", e),
    })
}

/// Validate Steam username
pub fn is_valid_steam_username(username: &str) -> bool {
    !username.is_empty() 
        && username.len() <= 64
        && username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// Generate a random session ID
pub fn generate_session_id() -> i32 {
    use rand::Rng;
    rand::thread_rng().gen()
}

/// Create a job ID from timestamp and counter
pub fn create_job_id() -> u64 {
    static mut COUNTER: u32 = 0;
    let timestamp = get_unix_timestamp();
    
    unsafe {
        COUNTER = COUNTER.wrapping_add(1);
        (timestamp << 32) | (COUNTER as u64)
    }
}

/// Mask IP address for privacy
pub fn mask_ip_address(ip: std::net::IpAddr) -> String {
    match ip {
        std::net::IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            format!("{}.{}.XXX.XXX", octets[0], octets[1])
        }
        std::net::IpAddr::V6(ipv6) => {
            let segments = ipv6.segments();
            format!("{}:{}:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX", segments[0], segments[1])
        }
    }
}

/// Calculate SHA1 hash
pub fn sha1_hash(data: &[u8]) -> Vec<u8> {
    use sha1::{Sha1, Digest};
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Calculate SHA256 hash  
pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(feature = "crypto")]
mod crypto_utils {
    use sha1::{Sha1, Digest};
    use sha2::{Sha256, Digest as Sha2Digest};
    use hmac::{Hmac, Mac};
    use crate::types::SteamError;

    pub type HmacSha1 = Hmac<Sha1>;

    /// 计算HMAC-SHA1
    pub fn hmac_sha1(key: &[u8], data: &[u8]) -> Result<Vec<u8>, SteamError> {
        let mut mac = HmacSha1::new_from_slice(key)
            .map_err(|e| SteamError::Crypto(format!("Failed to create HMAC: {}", e)))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

/// Base64 encode
pub fn base64_encode(data: &[u8]) -> String {
    base64::encode(data)
}

/// Base64 decode
pub fn base64_decode(data: &str) -> Result<Vec<u8>, SteamError> {
    base64::decode(data).map_err(|e| SteamError::Unknown {
        message: format!("Failed to decode base64: {}", e),
    })
}

/// Format file size in human readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Sanitize filename for filesystem
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

/// Parse Steam store URL to extract app ID
pub fn parse_steam_store_url(url: &str) -> Option<u32> {
    if let Some(start) = url.find("/app/") {
        let id_part = &url[start + 5..];
        if let Some(end) = id_part.find('/').or_else(|| id_part.find('?')) {
            id_part[..end].parse().ok()
        } else {
            id_part.parse().ok()
        }
    } else {
        None
    }
}

/// Retry function with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_attempts: u32,
    initial_delay: std::time::Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut delay = initial_delay;
    
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_attempts {
                    return Err(e);
                }
                
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, std::time::Duration::from_secs(60));
            }
        }
    }
    
    unreachable!()
}

/// Create a rate limiter
pub struct RateLimiter {
    tokens: std::sync::Arc<tokio::sync::Mutex<u32>>,
    max_tokens: u32,
    refill_rate: std::time::Duration,
    last_refill: std::sync::Arc<tokio::sync::Mutex<std::time::Instant>>,
}

impl RateLimiter {
    pub fn new(max_tokens: u32, refill_rate: std::time::Duration) -> Self {
        Self {
            tokens: std::sync::Arc::new(tokio::sync::Mutex::new(max_tokens)),
            max_tokens,
            refill_rate,
            last_refill: std::sync::Arc::new(tokio::sync::Mutex::new(std::time::Instant::now())),
        }
    }
    
    pub async fn acquire(&self) -> bool {
        self.refill_tokens().await;
        
        let mut tokens = self.tokens.lock().await;
        if *tokens > 0 {
            *tokens -= 1;
            true
        } else {
            false
        }
    }
    
    async fn refill_tokens(&self) {
        let now = std::time::Instant::now();
        let mut last_refill = self.last_refill.lock().await;
        
        if now.duration_since(*last_refill) >= self.refill_rate {
            let mut tokens = self.tokens.lock().await;
            *tokens = self.max_tokens;
            *last_refill = now;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_community_id_to_steam_id() {
        let community_id = 76561198000000000;
        let steam_id = community_id_to_steam_id(community_id).unwrap();
        assert_eq!(steam_id.id, community_id);
    }

    #[test]
    fn test_account_id_to_steam_id() {
        let account_id = 123456;
        let steam_id = account_id_to_steam_id(account_id);
        assert_eq!(steam_id.account_id(), account_id);
    }

    #[test]
    fn test_hex_conversion() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let hex = bytes_to_hex(&data);
        assert_eq!(hex, "deadbeef");
        
        let decoded = hex_to_bytes(&hex).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_steam_username_validation() {
        assert!(is_valid_steam_username("valid_user123"));
        assert!(is_valid_steam_username("user-name"));
        assert!(!is_valid_steam_username(""));
        assert!(!is_valid_steam_username("user@domain"));
        assert!(!is_valid_steam_username("user with spaces"));
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1048576), "1.00 MB");
        assert_eq!(format_file_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_filename_sanitization() {
        let dirty = "file<name>with:illegal\"chars/here";
        let clean = sanitize_filename(dirty);
        assert!(!clean.contains('<'));
        assert!(!clean.contains('>'));
        assert!(!clean.contains(':'));
        assert!(!clean.contains('"'));
        assert!(!clean.contains('/'));
    }

    #[test]
    fn test_steam_store_url_parsing() {
        let url1 = "https://store.steampowered.com/app/730/Counter-Strike_Global_Offensive/";
        assert_eq!(parse_steam_store_url(url1), Some(730));
        
        let url2 = "https://store.steampowered.com/app/440/?snr=1_7_15__13";
        assert_eq!(parse_steam_store_url(url2), Some(440));
        
        let invalid_url = "https://store.steampowered.com/games/";
        assert_eq!(parse_steam_store_url(invalid_url), None);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, std::time::Duration::from_millis(100));
        
        // Should be able to acquire 2 tokens
        assert!(limiter.acquire().await);
        assert!(limiter.acquire().await);
        
        // Third attempt should fail
        assert!(!limiter.acquire().await);
        
        // Wait for refill and try again
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        assert!(limiter.acquire().await);
    }
}