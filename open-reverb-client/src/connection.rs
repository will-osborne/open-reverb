use anyhow::Result;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::sync::{Arc, Mutex};
use tracing::{error, info};
use uuid::Uuid;
use crossbeam_channel::{bounded, Sender, Receiver};

use open_reverb_common::protocol::Message;

pub struct Connection {
    connected: bool,
    user_id: Option<Uuid>,
    stream: Option<TcpStream>,
    message_sender: Sender<Message>,
    message_receiver: Receiver<Message>,
    current_channel_id: Option<Uuid>,
}

impl Connection {
    pub fn new() -> Self {
        let (sender, receiver) = bounded::<Message>(100);
        Self {
            connected: false,
            user_id: None,
            stream: None,
            message_sender: sender,
            message_receiver: receiver,
            current_channel_id: None,
        }
    }
    
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    pub fn connect(&mut self, server_url: &str) -> Result<()> {
        if self.connected {
            return Ok(());
        }
        
        info!("Connecting to server at {}", server_url);
        
        // Connect to the server
        let stream = TcpStream::connect(server_url)?;
        stream.set_nonblocking(true)?;
        
        // Store the stream
        self.stream = Some(stream);
        self.connected = true;
        
        Ok(())
    }
    
    pub fn disconnect(&mut self) {
        self.stream = None;
        self.connected = false;
        self.user_id = None;
    }
    
    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        if !self.connected || self.stream.is_none() {
            return Err(anyhow::anyhow!("Not connected to server"));
        }
        
        let login_request = Message::LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };
        
        self.send_message(&login_request)?;
        
        Ok(())
    }
    
    pub fn process_messages(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();
        
        if !self.connected || self.stream.is_none() {
            return messages;
        }
        
        // Try to read messages from the stream
        if let Some(stream) = &mut self.stream {
            let mut buffer = [0; 4096];
            
            match stream.read(&mut buffer) {
                Ok(0) => {
                    // Connection closed
                    info!("Connection closed by server");
                    self.disconnect();
                }
                Ok(n) => {
                    // Process received data
                    if let Ok(message) = serde_json::from_slice::<Message>(&buffer[..n]) {
                        // Handle login response to save user ID
                        if let Message::LoginResponse {
                            success: true,
                            user_id: Some(uid),
                            ..
                        } = message
                        {
                            self.user_id = Some(uid);
                        }
                        
                        messages.push(message);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, that's fine
                }
                Err(e) => {
                    error!("Error reading from socket: {}", e);
                    self.disconnect();
                }
            }
        }
        
        messages
    }
    
    fn send_message(&mut self, message: &Message) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            let message_bytes = serde_json::to_vec(message)?;
            let message_len = message_bytes.len() as u32;
            let len_bytes = message_len.to_be_bytes();
            
            // Send message length
            stream.write_all(&len_bytes)?;
            
            // Send message data
            stream.write_all(&message_bytes)?;
            
            stream.flush()?;
        }
        
        Ok(())
    }
    
    pub fn join_channel(&mut self, channel_id: Uuid) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to server"));
        }
        
        let join_request = Message::JoinChannel { channel_id };
        self.send_message(&join_request)?;
        
        Ok(())
    }
    
    pub fn leave_channel(&mut self, channel_id: Uuid) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to server"));
        }
        
        let leave_request = Message::LeaveChannel { channel_id };
        self.send_message(&leave_request)?;
        
        Ok(())
    }
    
    pub fn update_status(&mut self, status: open_reverb_common::models::UserStatus) -> Result<()> {
        if !self.connected || self.user_id.is_none() {
            return Err(anyhow::anyhow!("Not connected to server or not logged in"));
        }
        
        let status_update = Message::StatusUpdate { 
            user_id: self.user_id.unwrap(), 
            status 
        };
        
        self.send_message(&status_update)?;
        
        Ok(())
    }
    
    pub fn send_voice_data(&mut self, user_id: Uuid, channel_id: Uuid, data: Vec<u8>) -> Result<()> {
        if !self.connected || self.user_id.is_none() {
            return Err(anyhow::anyhow!("Not connected to server or not logged in"));
        }
        
        let voice_data = Message::VoiceData {
            user_id,
            channel_id,
            data,
        };
        
        self.send_message(&voice_data)?;
        
        Ok(())
    }
    
    pub fn send_video_data(&mut self, user_id: Uuid, channel_id: Uuid, data: Vec<u8>) -> Result<()> {
        if !self.connected || self.user_id.is_none() {
            return Err(anyhow::anyhow!("Not connected to server or not logged in"));
        }
        
        let video_data = Message::VideoData {
            user_id,
            channel_id,
            data,
        };
        
        self.send_message(&video_data)?;
        
        Ok(())
    }
    
    pub fn send_screen_share_data(&mut self, user_id: Uuid, channel_id: Uuid, data: Vec<u8>) -> Result<()> {
        if !self.connected || self.user_id.is_none() {
            return Err(anyhow::anyhow!("Not connected to server or not logged in"));
        }
        
        let screen_data = Message::ScreenShareData {
            user_id,
            channel_id,
            data,
        };
        
        self.send_message(&screen_data)?;
        
        Ok(())
    }
    
    pub fn get_sender(&self) -> Sender<Message> {
        self.message_sender.clone()
    }
    
    pub fn get_current_channel_id(&self) -> Option<Uuid> {
        self.current_channel_id
    }
    
    pub fn set_current_channel_id(&mut self, channel_id: Option<Uuid>) {
        self.current_channel_id = channel_id;
    }
    
    pub fn get_user_id(&self) -> Option<Uuid> {
        self.user_id
    }
}