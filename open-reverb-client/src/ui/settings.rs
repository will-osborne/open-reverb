use egui::{Button, ComboBox, Slider, Ui, Window};

use crate::audio::AudioManager;
use crate::config::{ClientConfig, Theme};
use crate::ui::style;
use crate::video::VideoManager;

pub struct SettingsScreen {
    config: ClientConfig,
    modified: bool,
    available_audio_inputs: Vec<String>,
    available_audio_outputs: Vec<String>,
    available_video_devices: Vec<String>,
}

impl SettingsScreen {
    pub fn new(config: ClientConfig) -> Self {
        // Get available devices
        // In a real implementation, we would get these from the OS
        let available_audio_inputs = vec!["Default Microphone".to_string(), "Headset Microphone".to_string()];
        let available_audio_outputs = vec!["Default Speakers".to_string(), "Headphones".to_string()];
        let available_video_devices = VideoManager::get_available_video_devices();
        
        // If we have no video devices, add a placeholder
        let available_video_devices = if available_video_devices.is_empty() {
            vec!["Default Camera".to_string()]
        } else {
            available_video_devices
        };
        Self {
            config,
            modified: false,
            available_audio_inputs,
            available_audio_outputs,
            available_video_devices,
        }
    }
    
    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) -> Option<ClientConfig> {
        let mut result = None;
        let is_open = *open;
        
        if !is_open {
            return None;
        }
        
        Window::new("Settings")
            .resizable(false)
            .default_width(500.0)
            .show(ctx, |ui| {
                ui.heading(style::heading("Settings"));
                ui.add_space(10.0);
                
                ui.separator();
                ui.add_space(10.0);
                
                // Server settings
                ui.heading(style::subheading("Server"));
                ui.horizontal(|ui| {
                    ui.label("Server Address:");
                    if ui.text_edit_singleline(&mut self.config.server_url).changed() {
                        self.modified = true;
                    }
                });
                
                ui.add_space(20.0);
                
                // User interface settings
                ui.heading(style::subheading("User Interface"));
                
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    ComboBox::from_id_source("theme_selector")
                        .selected_text(self.theme_name(self.config.theme))
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(self.config.theme == Theme::Light, "Light").clicked() {
                                self.config.theme = Theme::Light;
                                self.modified = true;
                            }
                            if ui.selectable_label(self.config.theme == Theme::Dark, "Dark").clicked() {
                                self.config.theme = Theme::Dark;
                                self.modified = true;
                            }
                            if ui.selectable_label(self.config.theme == Theme::System, "System").clicked() {
                                self.config.theme = Theme::System;
                                self.modified = true;
                            }
                        });
                });
                
                ui.add_space(10.0);
                
                if ui.checkbox(&mut self.config.notification_sounds, "Notification Sounds").changed() {
                    self.modified = true;
                }
                
                if ui.checkbox(&mut self.config.remember_credentials, "Remember Credentials").changed() {
                    self.modified = true;
                }
                
                ui.add_space(20.0);
                
                // Audio settings
                ui.heading(style::subheading("Audio"));
                
                // Input device selection
                ui.horizontal(|ui| {
                    ui.label("Microphone:");
                    let selected_input = self.config.audio_input_device.clone().unwrap_or_else(|| "Default".to_string());
                    ComboBox::from_id_source("audio_input_selector")
                        .selected_text(&selected_input)
                        .show_ui(ui, |ui| {
                            for device in &self.available_audio_inputs {
                                if ui.selectable_label(
                                    self.config.audio_input_device.as_ref() == Some(device),
                                    device
                                ).clicked() {
                                    self.config.audio_input_device = Some(device.clone());
                                    self.modified = true;
                                }
                            }
                        });
                });
                
                // Output device selection
                ui.horizontal(|ui| {
                    ui.label("Speakers:");
                    let selected_output = self.config.audio_output_device.clone().unwrap_or_else(|| "Default".to_string());
                    ComboBox::from_id_source("audio_output_selector")
                        .selected_text(&selected_output)
                        .show_ui(ui, |ui| {
                            for device in &self.available_audio_outputs {
                                if ui.selectable_label(
                                    self.config.audio_output_device.as_ref() == Some(device),
                                    device
                                ).clicked() {
                                    self.config.audio_output_device = Some(device.clone());
                                    self.modified = true;
                                }
                            }
                        });
                });
                
                // Volume controls
                ui.horizontal(|ui| {
                    ui.label("Output Volume:");
                    if ui.add(Slider::new(&mut self.config.audio_volume, 0.0..=1.0)).changed() {
                        self.modified = true;
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Microphone Volume:");
                    if ui.add(Slider::new(&mut self.config.microphone_volume, 0.0..=1.0)).changed() {
                        self.modified = true;
                    }
                });
                
                ui.add_space(20.0);
                
                // Video settings
                ui.heading(style::subheading("Video"));
                
                // Camera selection
                ui.horizontal(|ui| {
                    ui.label("Camera:");
                    let selected_camera = self.config.video_device.clone().unwrap_or_else(|| "Default".to_string());
                    ComboBox::from_id_source("video_device_selector")
                        .selected_text(&selected_camera)
                        .show_ui(ui, |ui| {
                            for device in &self.available_video_devices {
                                if ui.selectable_label(
                                    self.config.video_device.as_ref() == Some(device),
                                    device
                                ).clicked() {
                                    self.config.video_device = Some(device.clone());
                                    self.modified = true;
                                }
                            }
                        });
                });
                
                ui.add_space(20.0);
                
                // Buttons
                ui.separator();
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut should_close = false;
                        
                        if ui.add_sized([100.0, 30.0], Button::new(style::body_text("Cancel"))).clicked() {
                            should_close = true;
                        }
                        
                        if ui.add_sized([100.0, 30.0], Button::new(style::body_text("Save")))
                            .clicked() 
                        {
                            result = Some(self.config.clone());
                            should_close = true;
                        }
                        
                        if should_close {
                            *open = false;
                        }
                    });
                });
            });
        
        result
    }
    
    fn theme_name(&self, theme: Theme) -> &'static str {
        match theme {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::System => "System",
        }
    }
    
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}