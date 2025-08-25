use egui::{Align, Button, Layout, Ui};

use crate::config::ClientConfig;
use crate::ui::style;

pub struct LoginScreen {
    server_url: String,
    username: String,
    password: String,
    remember_credentials: bool,
    error_message: Option<String>,
    connecting: bool,
}

impl LoginScreen {
    pub fn new(config: &ClientConfig) -> Self {
        Self {
            server_url: config.server_url.clone(),
            username: config.username.clone().unwrap_or_default(),
            password: String::new(),
            remember_credentials: config.remember_credentials,
            error_message: None,
            connecting: false,
        }
    }
    
    pub fn ui(&mut self, ui: &mut Ui) -> Option<(String, String, String)> {
        let mut login_info = None;
        
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(style::heading("Open Reverb"));
            ui.add_space(20.0);
            
            // Server URL
            ui.label(style::body_text("Server Address:"));
            ui.text_edit_singleline(&mut self.server_url);
            ui.add_space(10.0);
            
            // Username
            ui.label(style::body_text("Username:"));
            ui.text_edit_singleline(&mut self.username);
            ui.add_space(10.0);
            
            // Password
            ui.label(style::body_text("Password:"));
            let password_response = ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
            
            // Remember credentials
            ui.checkbox(&mut self.remember_credentials, "Remember credentials");
            ui.add_space(20.0);
            
            // Error message if any
            if let Some(error) = &self.error_message {
                ui.label(style::error_text(error));
                ui.add_space(10.0);
            }
            
            // Connect button
            let connect_text = if self.connecting { "Connecting..." } else { "Connect" };
            let connect_button = ui.add_sized(
                [200.0, 40.0],
                Button::new(style::body_text(connect_text))
                    .fill(style::ACCENT_COLOR)
            );
            
            // Handle connect button click or enter key in password field
            if (connect_button.clicked() || password_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                && !self.connecting
                && !self.username.is_empty()
                && !self.password.is_empty()
            {
                self.connecting = true;
                self.error_message = None;
                
                // Return login info to be processed by the caller
                login_info = Some((
                    self.server_url.clone(),
                    self.username.clone(),
                    self.password.clone(),
                ));
            }
            
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.add_space(10.0);
                ui.label(style::secondary_text(&format!("Version {}", open_reverb_common::version())));
            });
        });
        
        login_info
    }
    
    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
        self.connecting = false;
    }
    
    pub fn is_connecting(&self) -> bool {
        self.connecting
    }
    
    pub fn get_credentials(&self) -> (String, String, bool) {
        (self.username.clone(), self.password.clone(), self.remember_credentials)
    }
    
    pub fn get_server_url(&self) -> &str {
        &self.server_url
    }
}