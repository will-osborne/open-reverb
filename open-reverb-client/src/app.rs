use eframe::{egui, CreationContext};
use egui::{Color32, Ui};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

use crate::audio::AudioManager;
use crate::config::{self, ClientConfig, Theme};
use crate::connection::Connection;
use crate::ui::style;
use crate::video::{VideoManager, VideoPlayback, CaptureType};

pub struct DemoApp {
    name: String,
    server_url: String,
    password: String,
    connection: Arc<Connection>,
    status_message: Option<String>,
    show_settings: bool,
    theme: Theme,
    
    // Media components
    audio_manager: Option<AudioManager>,
    video_manager: Option<VideoManager>,
    screen_manager: Option<VideoManager>,
    video_playback: VideoPlayback,
    
    // Media state
    audio_active: bool,
    video_active: bool,
    screen_active: bool,
    
    // Selected devices
    selected_audio_input: Option<String>,
    selected_audio_output: Option<String>,
    selected_video_device: Option<String>,
}

impl DemoApp {
    pub fn new(cc: &CreationContext) -> Self {
        // Set up styles
        style::setup_style(&cc.egui_ctx);
        
        let connection = Arc::new(Connection::new());
        
        Self {
            name: "".to_string(),
            server_url: "127.0.0.1:8080".to_string(),
            password: "".to_string(),
            connection,
            status_message: None,
            show_settings: false,
            theme: Theme::Dark,
            
            audio_manager: None,
            video_manager: None,
            screen_manager: None,
            video_playback: VideoPlayback::new(),
            
            audio_active: false,
            video_active: false,
            screen_active: false,
            
            selected_audio_input: None,
            selected_audio_output: None,
            selected_video_device: None,
        }
    }
    fn handle_message(&mut self, message: open_reverb_common::protocol::Message) {
        use open_reverb_common::protocol::Message;
        
        match message {
            Message::LoginResponse { success, user_id, error } => {
                if success {
                    if let Some(id) = user_id {
                        info!("Login successful with user ID: {}", id);
                        self.status_message = Some(format!("Login successful with user ID: {}", id));
                    }
                } else if let Some(err) = error {
                    error!("Login failed: {}", err);
                    self.status_message = Some(format!("Login failed: {}", err));
                }
            }
            Message::VoiceData { user_id, channel_id, data } => {
                // Process received voice data
                // In a real implementation, this would be sent to the audio playback system
                info!("Received voice data from user {}", user_id);
            }
            Message::VideoData { user_id, channel_id, data } => {
                // Process received video data
                self.video_playback.process_video_data(user_id, data);
            }
            Message::ScreenShareData { user_id, channel_id, data } => {
                // Process received screen share data
                self.video_playback.process_video_data(user_id, data);
            }
            _ => {}
        }
    }
    
    fn toggle_audio(&mut self) {
        if let Some(user_id) = self.connection.get_user_id() {
            if self.audio_active {
                // Stop audio
                if let Some(audio_manager) = &mut self.audio_manager {
                    audio_manager.stop_audio();
                    self.audio_active = false;
                    info!("Audio streaming stopped");
                }
            } else {
                // Start audio
                if let Some(channel_id) = self.connection.get_current_channel_id() {
                    if self.audio_manager.is_none() {
                        self.audio_manager = Some(AudioManager::new(user_id, channel_id, self.connection.clone()));
                    }
                    
                    if let Some(audio_manager) = &mut self.audio_manager {
                        match audio_manager.start_audio() {
                            Ok(_) => {
                                self.audio_active = true;
                                info!("Audio streaming started");
                            }
                            Err(e) => {
                                error!("Failed to start audio: {}", e);
                                self.status_message = Some(format!("Failed to start audio: {}", e));
                            }
                        }
                    }
                } else {
                    self.status_message = Some("Join a channel first".to_string());
                }
            }
        } else {
            self.status_message = Some("You need to log in first".to_string());
        }
    }
    
    fn toggle_video(&mut self) {
        if let Some(user_id) = self.connection.get_user_id() {
            if self.video_active {
                // Stop video
                if let Some(video_manager) = &mut self.video_manager {
                    video_manager.stop();
                    self.video_active = false;
                    info!("Video streaming stopped");
                }
            } else {
                // Start video
                if let Some(channel_id) = self.connection.get_current_channel_id() {
                    if self.video_manager.is_none() {
                        self.video_manager = Some(VideoManager::new(user_id, channel_id, self.connection.clone(), CaptureType::Camera));
                    }
                    
                    if let Some(video_manager) = &mut self.video_manager {
                        // Initialize GStreamer if needed
                        if let Err(e) = video_manager.initialize() {
                            error!("Failed to initialize video: {}", e);
                            self.status_message = Some(format!("Failed to initialize video: {}", e));
                            return;
                        }
                        
                        match video_manager.start_camera() {
                            Ok(_) => {
                                self.video_active = true;
                                info!("Video streaming started");
                            }
                            Err(e) => {
                                error!("Failed to start video: {}", e);
                                self.status_message = Some(format!("Failed to start video: {}", e));
                            }
                        }
                    }
                } else {
                    self.status_message = Some("Join a channel first".to_string());
                }
            }
        } else {
            self.status_message = Some("You need to log in first".to_string());
        }
    }
    
    fn toggle_screen_sharing(&mut self) {
        if let Some(user_id) = self.connection.get_user_id() {
            if self.screen_active {
                // Stop screen sharing
                if let Some(screen_manager) = &mut self.screen_manager {
                    screen_manager.stop();
                    self.screen_active = false;
                    info!("Screen sharing stopped");
                }
            } else {
                // Start screen sharing
                if let Some(channel_id) = self.connection.get_current_channel_id() {
                    if self.screen_manager.is_none() {
                        self.screen_manager = Some(VideoManager::new(user_id, channel_id, self.connection.clone(), CaptureType::Screen));
                    }
                    
                    if let Some(screen_manager) = &mut self.screen_manager {
                        // Initialize GStreamer if needed
                        if let Err(e) = screen_manager.initialize() {
                            error!("Failed to initialize screen sharing: {}", e);
                            self.status_message = Some(format!("Failed to initialize screen sharing: {}", e));
                            return;
                        }
                        
                        match screen_manager.start_screen_sharing() {
                            Ok(_) => {
                                self.screen_active = true;
                                info!("Screen sharing started");
                            }
                            Err(e) => {
                                error!("Failed to start screen sharing: {}", e);
                                self.status_message = Some(format!("Failed to start screen sharing: {}", e));
                            }
                        }
                    }
                } else {
                    self.status_message = Some("Join a channel first".to_string());
                }
            }
        } else {
            self.status_message = Some("You need to log in first".to_string());
        }
    }
    
    fn stop_all_media(&mut self) {
        // Stop audio
        if self.audio_active && self.audio_manager.is_some() {
            self.audio_manager.as_mut().unwrap().stop_audio();
            self.audio_active = false;
        }
        
        // Stop video
        if self.video_active && self.video_manager.is_some() {
            self.video_manager.as_mut().unwrap().stop();
            self.video_active = false;
        }
        
        // Stop screen sharing
        if self.screen_active && self.screen_manager.is_some() {
            self.screen_manager.as_mut().unwrap().stop();
            self.screen_active = false;
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process messages from the server by cloning the Arc
        let messages = {
            let connection = Arc::clone(&self.connection);
            let connection_ref = unsafe { &mut *(Arc::as_ptr(&connection) as *mut Connection) };
            connection_ref.process_messages()
        };
        
        for message in messages {
            info!("Received message: {:?}", message);
            self.handle_message(message);
        }
        
        // Request continuous repaints for message processing
        ctx.request_repaint_after(Duration::from_millis(100));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading(style::heading("Open Reverb"));
                ui.add_space(20.0);
                
                ui.label(style::body_text("Server Address:"));
                ui.text_edit_singleline(&mut self.server_url);
                ui.add_space(10.0);
                
                ui.label(style::body_text("Username:"));
                ui.text_edit_singleline(&mut self.name);
                ui.add_space(10.0);
                
                ui.label(style::body_text("Password:"));
                ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                ui.add_space(20.0);
                
                if ui.button(if self.connection.is_connected() { "Disconnect" } else { "Connect" }).clicked() {
                    if self.connection.is_connected() {
                        // Stop any active media first
                        self.stop_all_media();
                        
                        // Disconnect from server
                        Arc::get_mut(&mut self.connection).unwrap().disconnect();
                        self.status_message = Some("Disconnected from server".to_string());
                        info!("Disconnected from server");
                    } else {
                        // Connect to server
                        match Arc::get_mut(&mut self.connection).unwrap().connect(&self.server_url) {
                            Ok(_) => {
                                info!("Connected to server at {}", self.server_url);
                                self.status_message = Some("Connected to server".to_string());
                                
                                // Login
                                if !self.name.is_empty() {
                                    match Arc::get_mut(&mut self.connection).unwrap().login(&self.name, &self.password) {
                                        Ok(_) => {
                                            info!("Login request sent for user: {}", self.name);
                                            self.status_message = Some(format!("Login request sent for user: {}", self.name));
                                        }
                                        Err(e) => {
                                            error!("Failed to login: {}", e);
                                            self.status_message = Some(format!("Login error: {}", e));
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to connect: {}", e);
                                self.status_message = Some(format!("Connection error: {}", e));
                            }
                        }
                    }
                }
                
                // Status message
                if let Some(message) = &self.status_message {
                    ui.add_space(10.0);
                    ui.label(style::body_text(message));
                }
                
                // Connection status
                if self.connection.is_connected() {
                    ui.add_space(10.0);
                    ui.label(style::body_text("Connection status: Connected"));
                    
                    // User ID if logged in
                    if let Some(user_id) = self.connection.get_user_id() {
                        ui.label(style::body_text(&format!("Logged in with ID: {}", user_id)));
                    } else {
                        ui.label(style::body_text("Not logged in yet"));
                    }
                }
                
                ui.add_space(30.0);
                ui.label(style::secondary_text(&format!("Version {}", open_reverb_common::version())));
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    ui.label(style::body_text("This is a simplified demo of the Open Reverb client UI."));
                });
                
                // Media controls section when connected
                if self.connection.is_connected() && self.connection.get_user_id().is_some() {
                    ui.add_space(20.0);
                    ui.heading(style::subheading("Media Controls"));
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button(if self.audio_active { "Stop Audio" } else { "Start Audio" }).clicked() {
                            self.toggle_audio();
                        }
                        
                        if ui.button(if self.video_active { "Stop Video" } else { "Start Video" }).clicked() {
                            self.toggle_video();
                        }
                        
                        if ui.button(if self.screen_active { "Stop Sharing" } else { "Share Screen" }).clicked() {
                            self.toggle_screen_sharing();
                        }
                    });
                    
                    // Show active media status
                    if self.audio_active || self.video_active || self.screen_active {
                        ui.add_space(10.0);
                        ui.label(style::body_text("Active Media:"));
                        
                        if self.audio_active {
                            ui.label(style::body_text("• Audio streaming active"));
                        }
                        
                        if self.video_active {
                            ui.label(style::body_text("• Video streaming active"));
                        }
                        
                        if self.screen_active {
                            ui.label(style::body_text("• Screen sharing active"));
                        }
                    }
                } else {
                    ui.add_space(10.0);
                    ui.label(style::body_text("The full implementation includes:"));
                    
                    ui.add_space(5.0);
                    bullet_point(ui, "Voice communication");
                    bullet_point(ui, "Video calling");
                    bullet_point(ui, "Screen sharing");
                    bullet_point(ui, "Channel-based communication");
                    bullet_point(ui, "User status management");
                }
            });
        });
    }
}

fn bullet_point(ui: &mut Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("•").color(Color32::from_rgb(88, 101, 242)));
        ui.label(style::body_text(text));
    });
}