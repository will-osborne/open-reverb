use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Channel, Server, User, UserStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Authentication
    LoginRequest { username: String, password: String },
    LoginResponse { success: bool, user_id: Option<Uuid>, error: Option<String> },
    
    // User status
    StatusUpdate { user_id: Uuid, status: UserStatus },
    UserJoined { user: User },
    UserLeft { user_id: Uuid },
    
    // Channels
    JoinChannel { channel_id: Uuid },
    LeaveChannel { channel_id: Uuid },
    ChannelUpdate { channel: Channel },
    
    // Voice
    VoiceData { user_id: Uuid, channel_id: Uuid, data: Vec<u8> },
    VoiceStarted { user_id: Uuid },
    VoiceStopped { user_id: Uuid },
    
    // Video
    VideoData { user_id: Uuid, channel_id: Uuid, data: Vec<u8> },
    VideoStarted { user_id: Uuid },
    VideoStopped { user_id: Uuid },
    
    // Screen sharing
    ScreenShareData { user_id: Uuid, channel_id: Uuid, data: Vec<u8> },
    ScreenShareStarted { user_id: Uuid },
    ScreenShareStopped { user_id: Uuid },
    
    // Server info
    ServerInfo { server: Server },
    
    // Ping/pong for keeping connection alive
    Ping,
    Pong,
    
    // Error messages
    Error { code: u32, message: String },
}