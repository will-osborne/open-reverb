use std::error::Error;
use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, RwLock};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tracing::{error, info};
use uuid::Uuid;

use open_reverb_common::protocol::Message;
use crate::server::Server;

pub async fn handle_connection(
    socket: TcpStream,
    server: Arc<RwLock<Server>>,
) -> Result<(), Box<dyn Error>> {
    // Split the socket into a reader and writer
    let (read_half, write_half) = socket.into_split();
    
    // Set up framed reader and writer for length-delimited messages
    let mut reader = FramedRead::new(read_half, LengthDelimitedCodec::new());
    let mut writer = FramedWrite::new(write_half, LengthDelimitedCodec::new());
    
    // User state
    let mut user_id: Option<Uuid> = None;
    let mut channel_id: Option<Uuid> = None;
    let mut broadcast_rx: Option<broadcast::Receiver<Message>> = None;
    
    // Process messages
    while let Some(result) = reader.next().await {
        let bytes = result?;
        let message: Message = serde_json::from_slice(&bytes)?;
        
        match message {
            Message::LoginRequest { username, password } => {
                // In a real implementation, validate the password against a database
                // For this example, just create a new user
                let uid = {
                    let mut server_write = server.write().await;
                    server_write.add_user(username)
                };
                
                user_id = Some(uid);
                
                // Send login response
                let response = Message::LoginResponse {
                    success: true,
                    user_id: Some(uid),
                    error: None,
                };
                
                let response_bytes = serde_json::to_vec(&response)?;
                writer.send(bytes::Bytes::from(response_bytes)).await?;
                
                // Send server information
                let server_info = {
                    let server_read = server.read().await;
                    server_read.get_server_info()
                };
                
                let server_info_msg = Message::ServerInfo { server: server_info };
                let server_info_bytes = serde_json::to_vec(&server_info_msg)?;
                writer.send(bytes::Bytes::from(server_info_bytes)).await?;
            }
            
            Message::JoinChannel { channel_id: cid } => {
                if let Some(uid) = user_id {
                    let success = {
                        let mut server_write = server.write().await;
                        server_write.join_channel(uid, cid)
                    };
                    
                    if success {
                        channel_id = Some(cid);
                        
                        // Subscribe to channel broadcast
                        let channel_sender = {
                            let server_read = server.read().await;
                            server_read.get_channel_sender(&cid)
                        };
                        
                        if let Some(sender) = channel_sender {
                            broadcast_rx = Some(sender.subscribe());
                            
                            // Notify others that user joined
                            let user = {
                                let server_read = server.read().await;
                                server_read.get_user(&uid).cloned()
                            };
                            
                            if let Some(user) = user {
                                let user_joined_msg = Message::UserJoined { user };
                                let _ = sender.send(user_joined_msg);
                            }
                        }
                    }
                }
            }
            
            Message::LeaveChannel { channel_id: cid } => {
                if let Some(uid) = user_id {
                    let mut server_write = server.write().await;
                    server_write.leave_channel(uid);
                    channel_id = None;
                    broadcast_rx = None;
                }
            }
            
            Message::VoiceData { user_id: uid, channel_id: cid, data } => {
                if let Some(channel_sender) = {
                    let server_read = server.read().await;
                    server_read.get_channel_sender(&cid)
                } {
                    // Forward the voice data to all users in the channel
                    let _ = channel_sender.send(message);
                }
            }
            
            Message::VideoData { user_id: uid, channel_id: cid, data } => {
                if let Some(channel_sender) = {
                    let server_read = server.read().await;
                    server_read.get_channel_sender(&cid)
                } {
                    // Forward the video data to all users in the channel
                    let _ = channel_sender.send(message);
                }
            }
            
            Message::ScreenShareData { user_id: uid, channel_id: cid, data } => {
                if let Some(channel_sender) = {
                    let server_read = server.read().await;
                    server_read.get_channel_sender(&cid)
                } {
                    // Forward the screen share data to all users in the channel
                    let _ = channel_sender.send(message);
                }
            }
            
            Message::StatusUpdate { user_id: uid, status } => {
                if let Some(user_id) = user_id {
                    let mut server_write = server.write().await;
                    server_write.update_user_status(user_id, status);
                    
                    // Broadcast status update to all users
                    if let Some(cid) = channel_id {
                        if let Some(channel_sender) = server_write.get_channel_sender(&cid) {
                            let _ = channel_sender.send(message);
                        }
                    }
                }
            }
            
            Message::Ping => {
                // Respond with a pong
                let pong = Message::Pong;
                let pong_bytes = serde_json::to_vec(&pong)?;
                writer.send(bytes::Bytes::from(pong_bytes)).await?;
            }
            
            _ => {
                // Handle other message types or ignore them
            }
        }
        
        // Check for broadcast messages from the channel
        if let Some(ref mut rx) = broadcast_rx {
            while let Ok(msg) = rx.try_recv() {
                let msg_bytes = serde_json::to_vec(&msg)?;
                writer.send(bytes::Bytes::from(msg_bytes)).await?;
            }
        }
    }
    
    // User disconnected, clean up
    if let Some(uid) = user_id {
        let mut server_write = server.write().await;
        server_write.remove_user(uid);
        
        if let Some(cid) = channel_id {
            if let Some(channel_sender) = server_write.get_channel_sender(&cid) {
                let user_left_msg = Message::UserLeft { user_id: uid };
                let _ = channel_sender.send(user_left_msg);
            }
        }
    }
    
    Ok(())
}