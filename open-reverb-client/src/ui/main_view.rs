use egui::{Button, Color32, Label, RichText, SidePanel, TopBottomPanel, Ui, Vec2};
use uuid::Uuid;

use open_reverb_common::models::{Channel, Server, User, UserStatus};
use crate::ui::style;
use crate::video::VideoPlayback;

pub struct MainView {
    current_user_id: Option<Uuid>,
    current_channel_id: Option<Uuid>,
    server_info: Option<Server>,
    
    // Audio state for visualization
    audio_levels: std::collections::HashMap<Uuid, f32>,
    audio_active: bool,
    video_active: bool,
    screen_share_active: bool,
    
    // Video playback
    video_playback: Option<VideoPlayback>,
    
    // UI state
    show_settings: bool,
}

impl MainView {
    pub fn new() -> Self {
        Self {
            current_user_id: None,
            current_channel_id: None,
            server_info: None,
            audio_levels: std::collections::HashMap::new(),
            audio_active: false,
            video_active: false,
            screen_share_active: false,
            video_playback: Some(VideoPlayback::new()),
            show_settings: false,
        }
    }
    
    pub fn ui(&mut self, ui: &mut Ui) {
        // Top bar with server name and controls
        TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let server_name = match &self.server_info {
                    Some(server) => &server.name,
                    None => "Not connected",
                };
                
                ui.heading(style::heading(server_name));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Settings").clicked() {
                        self.show_settings = true;
                    }
                    
                    // Status selector
                    let status = self.get_current_user_status();
                    let status_color = style::status_color(status);
                    
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("●").color(status_color));
                        ui.menu_button(self.status_text(status), |ui| {
                            if ui.add(Button::new(style::body_text("Online"))
                                .fill(if status == UserStatus::Online { style::ACCENT_COLOR } else { Color32::TRANSPARENT }))
                                .clicked() 
                            {
                                // This would update the status in a real implementation
                                ui.close_menu();
                            }
                            
                            if ui.add(Button::new(style::body_text("Away"))
                                .fill(if status == UserStatus::Away { style::ACCENT_COLOR } else { Color32::TRANSPARENT }))
                                .clicked() 
                            {
                                // This would update the status in a real implementation
                                ui.close_menu();
                            }
                            
                            if ui.add(Button::new(style::body_text("Do Not Disturb"))
                                .fill(if status == UserStatus::DoNotDisturb { style::ACCENT_COLOR } else { Color32::TRANSPARENT }))
                                .clicked() 
                            {
                                // This would update the status in a real implementation
                                ui.close_menu();
                            }
                        });
                    });
                    
                    // Username display
                    if let Some(user) = self.get_current_user() {
                        ui.label(style::body_text(&user.username));
                    }
                });
            });
        });
        
        // Side panel with channels and users
        SidePanel::left("channels_panel")
            .resizable(true)
            .default_width(250.0)
            .show_inside(ui, |ui| {
                ui.heading(style::subheading("Channels"));
                ui.separator();
                
                if let Some(server) = &self.server_info {
                    self.render_channels(ui, server);
                    
                    ui.add_space(20.0);
                    ui.heading(style::subheading("Users"));
                    ui.separator();
                    
                    self.render_users(ui, server);
                } else {
                    ui.label(style::secondary_text("Not connected to a server"));
                }
            });
        
        // Main content area
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(channel_id) = self.current_channel_id {
                if let Some(channel) = self.get_channel(channel_id) {
                    ui.heading(style::heading(&channel.name));
                    
                    if let Some(description) = &channel.description {
                        ui.label(style::secondary_text(description));
                    }
                    
                    ui.separator();
                    
                    // Media controls
                    ui.horizontal(|ui| {
                        if ui.button(if self.audio_active { "Mute" } else { "Unmute" }).clicked() {
                            self.audio_active = !self.audio_active;
                            // In a real implementation, this would toggle audio capture
                        }
                        
                        if ui.button(if self.video_active { "Stop Video" } else { "Start Video" }).clicked() {
                            self.video_active = !self.video_active;
                            // In a real implementation, this would toggle video capture
                        }
                        
                        if ui.button(if self.screen_share_active { "Stop Sharing" } else { "Share Screen" }).clicked() {
                            self.screen_share_active = !self.screen_share_active;
                            // In a real implementation, this would toggle screen sharing
                        }
                        
                        if ui.button("Leave Channel").clicked() {
                            // Leave the channel in a real implementation
                            self.current_channel_id = None;
                        }
                    });
                    
                    ui.separator();
                    
                    // Display area for video/screen sharing
                    if self.video_active || self.screen_share_active {
                        self.render_video_area(ui);
                    }
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading(style::heading("Welcome to Open Reverb"));
                    ui.label(style::body_text("Select a channel from the list to join"));
                });
            }
        });
    }
    
    pub fn set_current_user_id(&mut self, user_id: Uuid) {
        self.current_user_id = Some(user_id);
    }
    
    pub fn set_server_info(&mut self, server: Server) {
        self.server_info = Some(server);
    }
    
    pub fn update_audio_level(&mut self, user_id: Uuid, level: f32) {
        self.audio_levels.insert(user_id, level);
    }
    
    fn render_channels(&self, ui: &mut Ui, server: &Server) {
        for channel in &server.channels {
            let is_active = self.current_channel_id == Some(channel.id);
            let text = if is_active {
                RichText::new(&channel.name).color(style::ACCENT_COLOR).strong()
            } else {
                style::body_text(&channel.name)
            };
            
            if ui.selectable_label(is_active, text).clicked() && !is_active {
                // This would join the channel in a real implementation
                // self.current_channel_id = Some(channel.id);
            }
        }
    }
    
    fn render_users(&self, ui: &mut Ui, server: &Server) {
        for user in &server.users {
            let status_color = style::status_color(user.status);
            let is_current_user = self.current_user_id == Some(user.id);
            let is_speaking = self.audio_levels.get(&user.id).copied().unwrap_or(0.0) > 0.05;
            
            ui.horizontal(|ui| {
                // Status indicator
                ui.add(Label::new(RichText::new("●").color(status_color)));
                
                // Username
                let username_text = if is_current_user {
                    RichText::new(&user.username).strong()
                } else if is_speaking {
                    RichText::new(&user.username).color(style::ACCENT_COLOR)
                } else {
                    style::body_text(&user.username)
                };
                
                ui.add(Label::new(username_text));
                
                // Speaking indicator
                if is_speaking {
                    ui.add(Label::new(RichText::new("🔊")));
                }
            });
        }
    }
    
    fn get_current_user(&self) -> Option<&User> {
        if let Some(user_id) = self.current_user_id {
            if let Some(server) = &self.server_info {
                return server.users.iter().find(|u| u.id == user_id);
            }
        }
        None
    }
    
    fn get_current_user_status(&self) -> UserStatus {
        if let Some(user) = self.get_current_user() {
            user.status
        } else {
            UserStatus::Offline
        }
    }
    
    fn get_channel(&self, channel_id: Uuid) -> Option<&Channel> {
        if let Some(server) = &self.server_info {
            server.channels.iter().find(|c| c.id == channel_id)
        } else {
            None
        }
    }
    
    fn status_text(&self, status: UserStatus) -> &'static str {
        match status {
            UserStatus::Online => "Online",
            UserStatus::Away => "Away",
            UserStatus::DoNotDisturb => "Do Not Disturb",
            UserStatus::Offline => "Offline",
        }
    }
    
    pub fn is_showing_settings(&self) -> bool {
        self.show_settings
    }
    
    pub fn close_settings(&mut self) {
        self.show_settings = false;
    }
    
    fn render_video_area(&mut self, ui: &mut Ui) {
        // Allocate space for the video display
        let available_width = ui.available_width();
        let video_height = 400.0;
        
        ui.allocate_ui(Vec2::new(available_width, video_height), |ui| {
            if let Some(video_playback) = &self.video_playback {
                // Calculate participant layout
                let active_users = self.get_active_video_users();
                
                if active_users.is_empty() {
                    // No active video users
                    ui.centered_and_justified(|ui| {
                        ui.label(style::body_text("No active video participants"));
                    });
                    return;
                }
                
                // Determine layout based on number of participants
                let (cols, rows) = self.calculate_grid_layout(active_users.len());
                
                // Calculate dimensions for each video cell
                let cell_width = available_width / cols as f32;
                let cell_height = video_height / rows as f32;
                
                let mut row = 0;
                let mut col = 0;
                
                // Render each participant's video
                for user_id in active_users {
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(col as f32 * cell_width, row as f32 * cell_height),
                        egui::vec2(cell_width, cell_height),
                    );
                    
                    let response = ui.allocate_rect(rect, egui::Sense::hover());
                    
                    // Draw video frame or placeholder
                    if let Some(user) = self.get_user(user_id) {
                        // In a real implementation, we would render the video frame here
                        // using the texture from the video data
                        ui.painter().rect_filled(
                            rect.shrink(4.0),
                            4.0,
                            Color32::from_rgb(40, 40, 40),
                        );
                        
                        // Draw username
                        let text_rect = egui::Rect::from_min_max(
                            rect.left_bottom() + egui::vec2(8.0, -25.0),
                            rect.right_bottom() - egui::vec2(8.0, 5.0),
                        );
                        
                        ui.painter().rect_filled(
                            text_rect,
                            2.0,
                            Color32::from_rgba_premultiplied(0, 0, 0, 200),
                        );
                        
                        ui.painter().text(
                            text_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &user.username,
                            egui::TextStyle::Body.resolve(ui.style()),
                            Color32::WHITE,
                        );
                    }
                    
                    // Update grid position
                    col += 1;
                    if col >= cols {
                        col = 0;
                        row += 1;
                    }
                }
            } else {
                // Fallback if video playback is not initialized
                ui.centered_and_justified(|ui| {
                    ui.label(style::body_text("Video playback not available"));
                });
            }
        });
    }
    
    fn get_active_video_users(&self) -> Vec<Uuid> {
        // In a real implementation, this would track active video participants
        // For demo purposes, we'll use all users in the current channel
        if let Some(server) = &self.server_info {
            if let Some(channel_id) = self.current_channel_id {
                // Get users in the current channel
                return server.users.iter()
                    .filter(|_| true) // In a real implementation, filter by active video users
                    .map(|u| u.id)
                    .collect();
            }
        }
        Vec::new()
    }
    
    fn get_user(&self, user_id: Uuid) -> Option<&User> {
        if let Some(server) = &self.server_info {
            return server.users.iter().find(|u| u.id == user_id);
        }
        None
    }
    
    fn calculate_grid_layout(&self, count: usize) -> (usize, usize) {
        match count {
            0 => (1, 1),
            1 => (1, 1),
            2 => (2, 1),
            3 | 4 => (2, 2),
            5 | 6 => (3, 2),
            7 | 8 | 9 => (3, 3),
            _ => {
                // For more than 9 participants, use a scrollable grid
                let cols = 3;
                let rows = (count as f32 / cols as f32).ceil() as usize;
                (cols, rows)
            }
        }
    }
    
    pub fn update_video_frame(&mut self, user_id: Uuid, frame_data: Vec<u8>) {
        if let Some(video_playback) = &mut self.video_playback {
            video_playback.process_video_data(user_id, frame_data);
        }
    }
}