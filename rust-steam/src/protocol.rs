//! Steam Protocol Messages
//! 
//! Defines Steam protocol message types, serialization and deserialization

use crate::types::{SteamError, SteamID, EResult, EMsg, MsgHdr, ExtendedMsgHdr};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// Magic constant for Steam protocol
pub const STEAM_MAGIC: u32 = 0x31305456; // "VT01"

/// Steam message packet
#[derive(Debug, Clone)]
pub struct SteamPacket {
    pub header: PacketHeader,
    pub body: Vec<u8>,
}

/// Packet header types
#[derive(Debug, Clone)]
pub enum PacketHeader {
    /// Standard message header
    Standard(MsgHdr),
    /// Extended message header (with Steam ID)
    Extended(ExtendedMsgHdr),
}

impl SteamPacket {
    /// Create a new Steam packet
    pub fn new(msg: EMsg, body: Vec<u8>) -> Self {
        Self {
            header: PacketHeader::Standard(MsgHdr {
                msg,
                target_job_id: 0,
                source_job_id: 0,
            }),
            body,
        }
    }

    /// Create a new extended Steam packet with Steam ID
    pub fn new_extended(msg: EMsg, steam_id: SteamID, session_id: i32, body: Vec<u8>) -> Self {
        Self {
            header: PacketHeader::Extended(ExtendedMsgHdr {
                msg,
                header_size: 36, // Standard extended header size
                header_version: 2,
                target_job_id: 0,
                source_job_id: 0,
                header_canary: 239, // Standard canary value
                steam_id,
                session_id,
            }),
            body,
        }
    }

    /// Serialize packet to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, SteamError> {
        let mut buffer = Vec::new();

        match &self.header {
            PacketHeader::Standard(hdr) => {
                buffer.write_u32::<LittleEndian>(hdr.msg as u32)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
                buffer.write_u64::<LittleEndian>(hdr.target_job_id)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
                buffer.write_u64::<LittleEndian>(hdr.source_job_id)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
            }
            PacketHeader::Extended(hdr) => {
                buffer.write_u32::<LittleEndian>(hdr.msg as u32 | 0x80000000) // Set extended flag
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
                buffer.write_u8(hdr.header_size)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
                buffer.write_u16::<LittleEndian>(hdr.header_version)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
                buffer.write_u64::<LittleEndian>(hdr.target_job_id)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
                buffer.write_u64::<LittleEndian>(hdr.source_job_id)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
                buffer.write_u8(hdr.header_canary)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
                buffer.write_u64::<LittleEndian>(hdr.steam_id.id)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
                buffer.write_i32::<LittleEndian>(hdr.session_id)
                    .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
            }
        }

        buffer.extend_from_slice(&self.body);
        Ok(buffer)
    }

    /// Deserialize packet from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, SteamError> {
        let mut cursor = Cursor::new(data);
        
        let msg_raw = cursor.read_u32::<LittleEndian>()
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        
        let is_extended = (msg_raw & 0x80000000) != 0;
        let msg_id = msg_raw & 0x7FFFFFFF;
        
        // Convert to EMsg enum (simplified)
        let msg = match msg_id {
            701 => EMsg::ClientLogon,
            716 => EMsg::ClientLogOff,
            751 => EMsg::ClientLogOnResponse,
            1303 => EMsg::ChannelEncryptRequest,
            1304 => EMsg::ChannelEncryptResponse,
            1305 => EMsg::ChannelEncryptResult,
            _ => return Err(SteamError::Unknown { 
                message: format!("Unknown message type: {}", msg_id) 
            }),
        };

        let header = if is_extended {
            let header_size = cursor.read_u8()
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
            let header_version = cursor.read_u16::<LittleEndian>()
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
            let target_job_id = cursor.read_u64::<LittleEndian>()
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
            let source_job_id = cursor.read_u64::<LittleEndian>()
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
            let header_canary = cursor.read_u8()
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
            let steam_id = SteamID::new(cursor.read_u64::<LittleEndian>()
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?);
            let session_id = cursor.read_i32::<LittleEndian>()
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?;

            PacketHeader::Extended(ExtendedMsgHdr {
                msg,
                header_size,
                header_version,
                target_job_id,
                source_job_id,
                header_canary,
                steam_id,
                session_id,
            })
        } else {
            let target_job_id = cursor.read_u64::<LittleEndian>()
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
            let source_job_id = cursor.read_u64::<LittleEndian>()
                .map_err(|e| SteamError::Unknown { message: e.to_string() })?;

            PacketHeader::Standard(MsgHdr {
                msg,
                target_job_id,
                source_job_id,
            })
        };

        // Read remaining data as body
        let mut body = Vec::new();
        cursor.read_to_end(&mut body)
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;

        Ok(SteamPacket { header, body })
    }

    /// Get message type
    pub fn get_message_type(&self) -> EMsg {
        match &self.header {
            PacketHeader::Standard(hdr) => hdr.msg,
            PacketHeader::Extended(hdr) => hdr.msg,
        }
    }
}

/// Channel encryption request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEncryptRequest {
    pub protocol_version: u32,
    pub universe: u32,
}

impl ChannelEncryptRequest {
    pub fn new() -> Self {
        Self {
            protocol_version: 1,
            universe: 1, // Public universe
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, SteamError> {
        let mut buffer = Vec::new();
        buffer.write_u32::<LittleEndian>(self.protocol_version)
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        buffer.write_u32::<LittleEndian>(self.universe)
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        Ok(buffer)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, SteamError> {
        let mut cursor = Cursor::new(data);
        let protocol_version = cursor.read_u32::<LittleEndian>()
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        let universe = cursor.read_u32::<LittleEndian>()
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        
        Ok(Self {
            protocol_version,
            universe,
        })
    }
}

/// Channel encryption response
#[derive(Debug, Clone)]
pub struct ChannelEncryptResponse {
    pub key: Vec<u8>,
    pub crc: u32,
}

impl ChannelEncryptResponse {
    pub fn deserialize(data: &[u8]) -> Result<Self, SteamError> {
        if data.len() < 132 { // 128 bytes key + 4 bytes CRC
            return Err(SteamError::Unknown { 
                message: "Invalid encryption response length".to_string() 
            });
        }

        let key = data[0..128].to_vec();
        let mut cursor = Cursor::new(&data[128..]);
        let crc = cursor.read_u32::<LittleEndian>()
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;

        Ok(Self { key, crc })
    }
}

/// Client logon message
#[derive(Debug, Clone)]
pub struct ClientLogon {
    pub protocol_version: u32,
    pub deprecated_obfuscation_mask: u32,
    pub current_directory: String,
    pub username: String,
    pub password: String,
    pub anonymous_user_target_account_name: String,
    pub real_name: String,
    pub phone_number: String,
    pub email_address: String,
    pub rtime32_account_creation: u32,
    pub account_name: String,
    pub friends_group_id: i32,
    pub friends_group_name: String,
    pub steam2_auth_ticket: Vec<u8>,
    pub email_verification: String,
    pub game_server_token: String,
    pub login_key: String,
    pub was_converted_deprecated_msg: bool,
    pub anon_user_target_account_name: String,
    pub resolved_user_steam_id: u64,
    pub eresult_sentry_file: i32,
    pub sha_sentry_file: Vec<u8>,
    pub auth_code: String,
    pub otp_type: i32,
    pub otp_value: u32,
    pub otp_identifier: String,
    pub steam2_unknown_value: bool,
    pub supports_rate_limit_response: bool,
    pub web_logon_nonce: String,
    pub priority_reason: i32,
    pub embedded_client_secret: Vec<u8>,
    pub disable_partner_autogrants: bool,
}

impl ClientLogon {
    pub fn new_with_tokens(username: String, access_token: String) -> Self {
        Self {
            protocol_version: 65580, // Current Steam protocol version
            deprecated_obfuscation_mask: 0,
            current_directory: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            username: username.clone(),
            password: String::new(), // Empty when using access token
            anonymous_user_target_account_name: String::new(),
            real_name: String::new(),
            phone_number: String::new(),
            email_address: String::new(),
            rtime32_account_creation: 0,
            account_name: username,
            friends_group_id: 0,
            friends_group_name: String::new(),
            steam2_auth_ticket: Vec::new(),
            email_verification: String::new(),
            game_server_token: String::new(),
            login_key: access_token, // Use access token as login key
            was_converted_deprecated_msg: false,
            anon_user_target_account_name: String::new(),
            resolved_user_steam_id: 0,
            eresult_sentry_file: 0,
            sha_sentry_file: Vec::new(),
            auth_code: String::new(),
            otp_type: 0,
            otp_value: 0,
            otp_identifier: String::new(),
            steam2_unknown_value: false,
            supports_rate_limit_response: true,
            web_logon_nonce: String::new(),
            priority_reason: 0,
            embedded_client_secret: Vec::new(),
            disable_partner_autogrants: false,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, SteamError> {
        // This is a simplified serialization
        // In a real implementation, this would use protobuf or VDF
        let mut buffer = Vec::new();
        
        buffer.write_u32::<LittleEndian>(self.protocol_version)
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        
        // Write strings as length-prefixed
        self.write_string(&mut buffer, &self.username)?;
        self.write_string(&mut buffer, &self.password)?;
        self.write_string(&mut buffer, &self.login_key)?;
        
        Ok(buffer)
    }

    fn write_string(&self, buffer: &mut Vec<u8>, s: &str) -> Result<(), SteamError> {
        let bytes = s.as_bytes();
        buffer.write_u32::<LittleEndian>(bytes.len() as u32)
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        buffer.extend_from_slice(bytes);
        Ok(())
    }
}

/// Client logon response
#[derive(Debug, Clone)]
pub struct ClientLogonResponse {
    pub result: EResult,
    pub out_of_game_heartbeat_seconds: i32,
    pub in_game_heartbeat_seconds: i32,
    pub deprecated_public_ip: u32,
    pub rtime32_server_time: u32,
    pub account_flags: u32,
    pub cell_id: u32,
    pub email_domain: String,
    pub steam2_ticket: Vec<u8>,
    pub eresult_extended: EResult,
    pub webapi_authenticate_user_nonce: String,
    pub cell_id_ping_threshold: u32,
    pub deprecated_use_pics: bool,
    pub vanity_url: String,
    pub public_ip: Vec<u8>,
    pub server_time: u32,
    pub account_name: String,
    pub count_loginfailures_to_migrate: u32,
    pub count_disconnects_to_migrate: u32,
    pub ogs_data_report_time_window: u32,
    pub client_supplied_steam_id: u64,
    pub ip_country_code: String,
    pub parental_settings: Vec<u8>,
    pub parental_setting_signature: Vec<u8>,
    pub count_loginfailures_password_to_migrate: u32,
    pub count_disconnects_password_to_migrate: u32,
    pub country_code: String,
    pub steam_id: u64,
    pub facebook_id: u64,
    pub facebook_name: String,
    pub steam_guard_notify_newmachines: bool,
    pub steam_guard_machine_name_user_chosen: String,
    pub is_steam_guard_machine_name_user_chosen: bool,
    pub request_id_verify_password: u32,
    pub is_phone_verified: bool,
    pub two_factor_state: u32,
    pub is_phone_identifying: bool,
    pub is_phone_needing_reverify: bool,
}

impl ClientLogonResponse {
    pub fn deserialize(data: &[u8]) -> Result<Self, SteamError> {
        let mut cursor = Cursor::new(data);
        
        // This is a simplified deserialization
        // In a real implementation, this would parse protobuf
        let result_raw = cursor.read_u32::<LittleEndian>()
            .map_err(|e| SteamError::Unknown { message: e.to_string() })?;
        
        let result = match result_raw {
            1 => EResult::OK,
            2 => EResult::Fail,
            5 => EResult::InvalidPassword,
            62 => EResult::AccountLogonDenied,
            _ => EResult::Fail,
        };

        // Create a basic response with default values
        Ok(Self {
            result,
            out_of_game_heartbeat_seconds: 500,
            in_game_heartbeat_seconds: 500,
            deprecated_public_ip: 0,
            rtime32_server_time: 0,
            account_flags: 0,
            cell_id: 0,
            email_domain: String::new(),
            steam2_ticket: Vec::new(),
            eresult_extended: EResult::OK,
            webapi_authenticate_user_nonce: String::new(),
            cell_id_ping_threshold: 0,
            deprecated_use_pics: false,
            vanity_url: String::new(),
            public_ip: Vec::new(),
            server_time: 0,
            account_name: String::new(),
            count_loginfailures_to_migrate: 0,
            count_disconnects_to_migrate: 0,
            ogs_data_report_time_window: 0,
            client_supplied_steam_id: 0,
            ip_country_code: String::new(),
            parental_settings: Vec::new(),
            parental_setting_signature: Vec::new(),
            count_loginfailures_password_to_migrate: 0,
            count_disconnects_password_to_migrate: 0,
            country_code: String::new(),
            steam_id: 0,
            facebook_id: 0,
            facebook_name: String::new(),
            steam_guard_notify_newmachines: false,
            steam_guard_machine_name_user_chosen: String::new(),
            is_steam_guard_machine_name_user_chosen: false,
            request_id_verify_password: 0,
            is_phone_verified: false,
            two_factor_state: 0,
            is_phone_identifying: false,
            is_phone_needing_reverify: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let packet = SteamPacket::new(EMsg::ClientLogon, vec![1, 2, 3, 4]);
        let serialized = packet.serialize().unwrap();
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_channel_encrypt_request() {
        let request = ChannelEncryptRequest::new();
        let serialized = request.serialize().unwrap();
        assert_eq!(serialized.len(), 8); // 2 u32 values
        
        let deserialized = ChannelEncryptRequest::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.protocol_version, request.protocol_version);
        assert_eq!(deserialized.universe, request.universe);
    }
}