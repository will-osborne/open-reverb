use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

use open_reverb_common::models::{Channel, Server, User, UserStatus};
use open_reverb_common::protocol::Message;

// Server state containing users, channels, and sessions
struct ServerState {
    users: HashMap<Uuid, User>,
    channels: HashMap<Uuid, Channel>,
    sessions: HashMap<String, SessionInfo>,
}

struct SessionInfo {
    user_id: Option<Uuid>,
    channels: Vec<Uuid>,
    addr: String,
}

impl ServerState {
    fn new() -> Self {
        // Create a default server with some channels
        let mut channels = HashMap::new();
        
        // General channel
        let general_id = Uuid::new_v4();
        channels.insert(general_id, Channel {
            id: general_id,
            name: "General".to_string(),
            description: Some("General voice channel".to_string()),
            parent_id: None,
            members: Vec::new(),
        });
        
        // Gaming channel
        let gaming_id = Uuid::new_v4();
        channels.insert(gaming_id, Channel {
            id: gaming_id,
            name: "Gaming".to_string(),
            description: Some("For gaming sessions".to_string()),
            parent_id: None,
            members: Vec::new(),
        });
        
        Self {
            users: HashMap::new(),
            channels,
            sessions: HashMap::new(),
        }
    }
    
    // Add a new session
    fn add_session(&mut self, addr: String) {
        self.sessions.insert(addr.clone(), SessionInfo {
            user_id: None,
            channels: Vec::new(),
            addr,
        });
    }
    
    // Remove a session
    fn remove_session(&mut self, addr: &str) -> Option<SessionInfo> {
        let session = self.sessions.remove(addr);
        
        // If the session had a user, mark them as offline
        if let Some(session_info) = &session {
            if let Some(user_id) = session_info.user_id {
                if let Some(user) = self.users.get_mut(&user_id) {
                    user.status = UserStatus::Offline;
                }
            }
        }
        
        session
    }
    
    // Handle login request
    fn handle_login(&mut self, addr: &str, username: String, _password: String) -> Message {
        // In a real implementation, we would validate the password
        // For this demo, we'll accept any password
        
        // Check if user already exists by username
        let user_id = {
            let user_by_name = self.users.iter().find(|(_, user)| user.username == username);
            
            if let Some((id, _)) = user_by_name {
                // User exists
                let user_id = *id;
                // Update status to Online
                if let Some(user) = self.users.get_mut(&user_id) {
                    user.status = UserStatus::Online;
                }
                user_id
            } else {
                // Create a new user
                let new_id = Uuid::new_v4();
                self.users.insert(new_id, User {
                    id: new_id,
                    username: username.clone(),
                    status: UserStatus::Online,
                });
                new_id
            }
        };
        
        // Update session
        if let Some(session) = self.sessions.get_mut(addr) {
            session.user_id = Some(user_id);
            
            // Return successful login response
            Message::LoginResponse {
                success: true,
                user_id: Some(user_id),
                error: None,
            }
        } else {
            // Session not found
            Message::LoginResponse {
                success: false,
                user_id: None,
                error: Some("Session not found".to_string()),
            }
        }
    }
    
    
    // Get server info
    fn get_server_info(&self) -> Server {
        Server {
            id: Uuid::new_v4(), // Generate a server ID
            name: "Open Reverb Server".to_string(),
            description: Some("A voice, video, and text communication server".to_string()),
            channels: self.channels.values().cloned().collect(),
            users: self.users.values().cloned().collect(),
        }
    }
}

// Handle a client connection
async fn handle_connection(
    socket: TcpStream,
    addr: String,
    server_state: Arc<Mutex<ServerState>>,
    tx: Arc<broadcast::Sender<(Uuid, Message)>>
) -> Result<(), Box<dyn Error>> {
    // Add the session
    {
        let mut state = server_state.lock().unwrap();
        state.add_session(addr.clone());
    }
    
    // Create a channel for receiving broadcasts
    let mut rx = tx.subscribe();
    
    // Split the socket for reading and writing
    let (mut reader, writer) = tokio::io::split(socket);
    
    // Buffer for incoming data
    let mut len_buf = [0u8; 4];
    let mut user_id = None;
    
    // Writer needs to be used across tasks, so we need to wrap it in an Arc<Mutex>
    let writer = Arc::new(tokio::sync::Mutex::new(writer));
    
    // Setup a task to forward messages from the broadcast channel to this client
    let addr_clone = addr.clone();
    let server_state_clone = Arc::clone(&server_state);
    let writer_clone = Arc::clone(&writer);
    
    let forward_task = tokio::spawn(async move {
        while let Ok((sender_id, message)) = rx.recv().await {
            // Don't send messages back to the sender
            let current_user_id = {
                let state = server_state_clone.lock().unwrap();
                state.sessions.get(&addr_clone).and_then(|s| s.user_id)
            };
            
            if current_user_id.is_none() || current_user_id.unwrap() != sender_id {
                let message_bytes = serde_json::to_vec(&message).unwrap_or_default();
                let message_len = message_bytes.len() as u32;
                let len_bytes = message_len.to_be_bytes();
                
                let mut writer = writer_clone.lock().await;
                
                if writer.write_all(&len_bytes).await.is_err() {
                    break;
                }
                
                if writer.write_all(&message_bytes).await.is_err() {
                    break;
                }
                
                if writer.flush().await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Main loop for handling incoming messages
    loop {
        // Read message length (4 bytes)
        match reader.read_exact(&mut len_buf).await {
            Ok(_) => {
                let message_len = u32::from_be_bytes(len_buf) as usize;
                
                // Read message data
                let mut message_buf = vec![0u8; message_len];
                if let Err(e) = reader.read_exact(&mut message_buf).await {
                    error!("Error reading message data: {}", e);
                    break;
                }
                
                // Parse message
                match serde_json::from_slice::<Message>(&message_buf) {
                    Ok(message) => {
                        info!("Received message: {:?}", message);
                        
                        // Handle message based on type
                        let response = match message {
                            Message::LoginRequest { username, password } => {
                                let response = {
                                    let mut state = server_state.lock().unwrap();
                                    state.handle_login(&addr, username, password)
                                };
                                
                                if let Message::LoginResponse { success: true, user_id: Some(id), .. } = &response {
                                    user_id = Some(*id);
                                    
                                    // Send server info after successful login
                                    let server_info = {
                                        let state = server_state.lock().unwrap();
                                        state.get_server_info()
                                    };
                                    
                                    // First send login response
                                    let login_bytes = serde_json::to_vec(&response)?;
                                    let login_len = login_bytes.len() as u32;
                                    let login_len_bytes = login_len.to_be_bytes();
                                    
                                    let mut writer_lock = writer.lock().await;
                                    writer_lock.write_all(&login_len_bytes).await?;
                                    writer_lock.write_all(&login_bytes).await?;
                                    writer_lock.flush().await?;
                                    drop(writer_lock); // Release the lock explicitly
                                    
                                    // Then send server info
                                    let server_info_msg = Message::ServerInfo { server: server_info };
                                    let server_bytes = serde_json::to_vec(&server_info_msg)?;
                                    let server_len = server_bytes.len() as u32;
                                    let server_len_bytes = server_len.to_be_bytes();
                                    
                                    let mut writer_lock = writer.lock().await;
                                    writer_lock.write_all(&server_len_bytes).await?;
                                    writer_lock.write_all(&server_bytes).await?;
                                    writer_lock.flush().await?;
                                    
                                    // No need for another response
                                    continue;
                                }
                                
                                Some(response)
                            },
                            Message::Ping => {
                                Some(Message::Pong)
                            },
                            Message::StatusUpdate { user_id, status } => {
                                // Update user status
                                {
                                    let mut state = server_state.lock().unwrap();
                                    if let Some(user) = state.users.get_mut(&user_id) {
                                        user.status = status;
                                    }
                                }
                                
                                // Broadcast status update to all clients
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            Message::JoinChannel { channel_id } => {
                                // Add user to channel
                                {
                                    let mut state = server_state.lock().unwrap();
                                    if let Some(session) = state.sessions.get_mut(&addr) {
                                        if !session.channels.contains(&channel_id) {
                                            session.channels.push(channel_id);
                                        }
                                    }
                                }
                                
                                // Broadcast to all clients
                                let _ = tx.send((user_id.unwrap(), message.clone()));
                                
                                None
                            },
                            Message::LeaveChannel { channel_id } => {
                                // Remove user from channel
                                {
                                    let mut state = server_state.lock().unwrap();
                                    if let Some(session) = state.sessions.get_mut(&addr) {
                                        session.channels.retain(|&id| id != channel_id);
                                    }
                                }
                                
                                // Broadcast to all clients
                                let _ = tx.send((user_id.unwrap(), message.clone()));
                                
                                None
                            },
                            Message::VoiceData { user_id, channel_id: _, ref data } => {
                                // Broadcast voice data to all clients in the channel
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            Message::VideoData { user_id, channel_id: _, ref data } => {
                                // Broadcast video data to all clients in the channel
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            Message::ScreenShareData { user_id, channel_id: _, ref data } => {
                                // Broadcast screen share data to all clients in the channel
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            Message::VoiceStarted { user_id } => {
                                // Broadcast voice started to all clients
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            Message::VoiceStopped { user_id } => {
                                // Broadcast voice stopped to all clients
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            Message::VideoStarted { user_id } => {
                                // Broadcast video started to all clients
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            Message::VideoStopped { user_id } => {
                                // Broadcast video stopped to all clients
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            Message::ScreenShareStarted { user_id } => {
                                // Broadcast screen share started to all clients
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            Message::ScreenShareStopped { user_id } => {
                                // Broadcast screen share stopped to all clients
                                let _ = tx.send((user_id, message.clone()));
                                
                                None
                            },
                            _ => None,
                        };
                        
                        // Send response if needed
                        if let Some(response) = response {
                            let response_bytes = serde_json::to_vec(&response)?;
                            let response_len = response_bytes.len() as u32;
                            let response_len_bytes = response_len.to_be_bytes();
                            
                            let mut writer_lock = writer.lock().await;
                            writer_lock.write_all(&response_len_bytes).await?;
                            writer_lock.write_all(&response_bytes).await?;
                            writer_lock.flush().await?;
                        }
                    },
                    Err(e) => {
                        error!("Error parsing message: {}", e);
                        break;
                    }
                }
            },
            Err(e) => {
                if e.kind() != std::io::ErrorKind::UnexpectedEof {
                    error!("Error reading message length: {}", e);
                }
                break;
            }
        }
    }
    
    // Connection closed, cleanup
    {
        let mut state = server_state.lock().unwrap();
        if let Some(session) = state.remove_session(&addr) {
            if let Some(uid) = session.user_id {
                // Broadcast that user left
                let _ = tx.send((uid, Message::UserLeft { user_id: uid }));
            }
        }
    }
    
    // Cancel the forward task
    forward_task.abort();
    
    info!("Connection closed for {}", addr);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting Open Reverb Server");
    
    // Bind to address
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);
    
    // Create a server state
    let server_state = Arc::new(Mutex::new(ServerState::new()));
    
    // Create a broadcast channel for messages
    let (tx, _) = broadcast::channel::<(Uuid, Message)>(100);
    let tx = Arc::new(tx);
    
    // Accept connections
    loop {
        let (socket, addr) = listener.accept().await?;
        info!("New connection from {}", addr);
        
        // Clone the server state and channel for this connection
        let server_state = Arc::clone(&server_state);
        let tx = Arc::clone(&tx);
        
        // Spawn a new task for each connection
        tokio::spawn(async move {
            info!("Connection established with {}", addr);
            
            if let Err(e) = handle_connection(socket, addr.to_string(), server_state, tx).await {
                error!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
}