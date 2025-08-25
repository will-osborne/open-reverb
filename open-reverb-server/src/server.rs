use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use open_reverb_common::models::{Channel, Server as ServerModel, User, UserStatus};
use open_reverb_common::protocol::Message;

pub struct Server {
    users: HashMap<Uuid, User>,
    channels: HashMap<Uuid, Channel>,
    // Maps user ID to channel ID
    user_channels: HashMap<Uuid, Uuid>,
    // Maps channel ID to active sessions in that channel
    channel_sessions: HashMap<Uuid, HashSet<Uuid>>,
    // Broadcast sender for each channel
    channel_senders: HashMap<Uuid, broadcast::Sender<Message>>,
}

impl Server {
    pub fn new() -> Self {
        let mut server = Self {
            users: HashMap::new(),
            channels: HashMap::new(),
            user_channels: HashMap::new(),
            channel_sessions: HashMap::new(),
            channel_senders: HashMap::new(),
        };
        
        // Create default channel
        let default_channel_id = Uuid::new_v4();
        let default_channel = Channel {
            id: default_channel_id,
            name: "Main".to_string(),
            description: Some("Default channel".to_string()),
            parent_id: None,
            members: Vec::new(),
        };
        
        server.channels.insert(default_channel_id, default_channel);
        server.channel_sessions.insert(default_channel_id, HashSet::new());
        
        // Create broadcast channel for the default channel
        let (sender, _) = broadcast::channel(100);
        server.channel_senders.insert(default_channel_id, sender);
        
        server
    }
    
    pub fn add_user(&mut self, username: String) -> Uuid {
        let user_id = Uuid::new_v4();
        let user = User {
            id: user_id,
            username,
            status: UserStatus::Online,
        };
        
        self.users.insert(user_id, user);
        user_id
    }
    
    pub fn remove_user(&mut self, user_id: Uuid) {
        if let Some(channel_id) = self.user_channels.remove(&user_id) {
            if let Some(sessions) = self.channel_sessions.get_mut(&channel_id) {
                sessions.remove(&user_id);
            }
        }
        
        self.users.remove(&user_id);
    }
    
    pub fn join_channel(&mut self, user_id: Uuid, channel_id: Uuid) -> bool {
        if !self.users.contains_key(&user_id) || !self.channels.contains_key(&channel_id) {
            return false;
        }
        
        // Remove from previous channel if any
        if let Some(prev_channel_id) = self.user_channels.get(&user_id) {
            if let Some(sessions) = self.channel_sessions.get_mut(prev_channel_id) {
                sessions.remove(&user_id);
            }
        }
        
        // Add to new channel
        self.user_channels.insert(user_id, channel_id);
        if let Some(sessions) = self.channel_sessions.get_mut(&channel_id) {
            sessions.insert(user_id);
        }
        
        true
    }
    
    pub fn leave_channel(&mut self, user_id: Uuid) {
        if let Some(channel_id) = self.user_channels.remove(&user_id) {
            if let Some(sessions) = self.channel_sessions.get_mut(&channel_id) {
                sessions.remove(&user_id);
            }
        }
    }
    
    pub fn get_channel_sender(&self, channel_id: &Uuid) -> Option<broadcast::Sender<Message>> {
        self.channel_senders.get(channel_id).cloned()
    }
    
    pub fn update_user_status(&mut self, user_id: Uuid, status: UserStatus) -> bool {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.status = status;
            true
        } else {
            false
        }
    }
    
    pub fn get_server_info(&self) -> ServerModel {
        ServerModel {
            id: Uuid::new_v4(), // In a real implementation, this would be stored
            name: "Open Reverb Server".to_string(),
            description: Some("A VoIP and video chat server".to_string()),
            channels: self.channels.values().cloned().collect(),
            users: self.users.values().cloned().collect(),
        }
    }
    
    pub fn get_channel(&self, channel_id: &Uuid) -> Option<&Channel> {
        self.channels.get(channel_id)
    }
    
    pub fn get_user(&self, user_id: &Uuid) -> Option<&User> {
        self.users.get(user_id)
    }
}