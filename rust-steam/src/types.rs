//! Core types and enums used throughout the Steam protocol

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Steam result codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EResult {
    OK = 1,
    Fail = 2,
    NoConnection = 3,
    InvalidPassword = 5,
    LoggedInElsewhere = 6,
    InvalidProtocolVer = 7,
    InvalidParam = 8,
    FileNotFound = 9,
    Busy = 10,
    InvalidState = 11,
    InvalidName = 12,
    InvalidEmail = 13,
    DuplicateName = 14,
    AccessDenied = 15,
    Timeout = 16,
    Banned = 17,
    AccountNotFound = 18,
    InvalidSteamID = 19,
    ServiceUnavailable = 20,
    NotLoggedOn = 21,
    Pending = 22,
    EncryptionFailure = 23,
    InsufficientPrivilege = 24,
    LimitExceeded = 25,
    Revoked = 26,
    Expired = 27,
    AlreadyRedeemed = 28,
    DuplicateRequest = 29,
    AlreadyOwned = 30,
    IPNotFound = 31,
    PersistFailed = 32,
    LockingFailed = 33,
    LogonSessionReplaced = 34,
    ConnectFailed = 35,
    HandshakeFailed = 36,
    IOFailure = 37,
    RemoteDisconnect = 38,
    ShoppingCartNotFound = 39,
    Blocked = 40,
    Ignored = 41,
    NoMatch = 42,
    AccountDisabled = 43,
    ServiceReadOnly = 44,
    AccountNotFeatured = 45,
    AdministratorOK = 46,
    ContentVersion = 47,
    TryAnotherCM = 48,
    PasswordRequiredToKickSession = 49,
    AlreadyLoggedInElsewhere = 50,
    Cancelled = 51,
    DataCorruption = 52,
    DiskFull = 53,
    RemoteCallFailed = 54,
    PasswordUnset = 55,
    ExternalAccountUnlinked = 56,
    PSNTicketInvalid = 57,
    ExternalAccountAlreadyLinked = 58,
    RemoteFileConflict = 59,
    IllegalPassword = 60,
    SameAsPreviousValue = 61,
    AccountLogonDenied = 62,
    CannotUseOldPassword = 63,
    InvalidLoginAuthCode = 64,
    AccountLogonDeniedNoMail = 65,
    HardwareNotCapableOfIPT = 66,
    InsufficientBatteryCharge = 67,
    CachedCredentialInvalid = 68,
    PhoneNumberIsVOIP = 69,
    NotSupported = 70,
    // Add more as needed
}

impl Default for EResult {
    fn default() -> Self {
        EResult::OK
    }
}

impl fmt::Display for EResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Steam ID representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SteamID {
    pub id: u64,
}

impl SteamID {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn from_account_id(account_id: u32) -> Self {
        // Universe = 1 (Public), Account Type = 1 (Individual), Instance = 1
        let id = ((1u64) << 56) | ((1u64) << 52) | ((1u64) << 32) | (account_id as u64);
        Self { id }
    }

    pub fn account_id(&self) -> u32 {
        (self.id & 0xFFFFFFFF) as u32
    }

    pub fn is_valid(&self) -> bool {
        self.id != 0
    }
}

impl fmt::Display for SteamID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Protocol types for connections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    TCP,
    UDP,
    WebSocket,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// 未连接
    Disconnected,
    /// 正在连接
    Connecting,
    /// 已连接
    Connected,
    /// 正在断开连接
    Disconnecting,
}

/// Steam error types
#[derive(Error, Debug)]
pub enum SteamError {
    /// 网络相关错误
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    /// 认证错误
    #[error("Authentication failed: {message}")]
    Authentication { message: String },
    
    /// 无效状态错误
    #[error("Invalid state: {message}")]
    InvalidState { message: String },
    
    /// 加密相关错误
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    /// 未知错误
    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

/// Steam message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EMsg {
    ChannelEncryptRequest = 1303,
    ChannelEncryptResponse = 1304,
    ChannelEncryptResult = 1305,
    
    ClientLogon = 701,
    ClientLogOff = 716,
    ClientLogOnResponse = 751,
    
    // Add more message types as needed
}

/// Steam message header
#[derive(Debug, Clone)]
pub struct MsgHdr {
    pub msg: EMsg,
    pub target_job_id: u64,
    pub source_job_id: u64,
}

/// Steam extended message header
#[derive(Debug, Clone)]
pub struct ExtendedMsgHdr {
    pub msg: EMsg,
    pub header_size: u8,
    pub header_version: u16,
    pub target_job_id: u64,
    pub source_job_id: u64,
    pub header_canary: u8,
    pub steam_id: SteamID,
    pub session_id: i32,
}

/// Steam universe types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EUniverse {
    Invalid = 0,
    Public = 1,
    Beta = 2,
    Internal = 3,
    Dev = 4,
}

/// Steam account types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EAccountType {
    Invalid = 0,
    Individual = 1,
    Multiseat = 2,
    GameServer = 3,
    AnonGameServer = 4,
    Pending = 5,
    ContentServer = 6,
    Clan = 7,
    Chat = 8,
    ConsoleUser = 9,
    AnonUser = 10,
}