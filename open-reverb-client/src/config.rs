use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server_url: String,
    pub username: Option<String>,
    pub remember_credentials: bool,
    pub theme: Theme,
    pub notification_sounds: bool,
    
    // Media settings
    pub audio_input_device: Option<String>,
    pub audio_output_device: Option<String>,
    pub video_device: Option<String>,
    pub audio_volume: f32,
    pub microphone_volume: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "127.0.0.1:8080".to_string(),
            username: None,
            remember_credentials: false,
            theme: Theme::System,
            notification_sounds: true,
            
            // Media settings
            audio_input_device: None,
            audio_output_device: None,
            video_device: None,
            audio_volume: 1.0,
            microphone_volume: 1.0,
        }
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "open-reverb", "client")
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    
    let config_dir = proj_dirs.config_dir();
    
    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }
    
    Ok(config_dir.to_path_buf())
}

pub fn load_config() -> Result<ClientConfig> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.json");
    
    // If config file doesn't exist, create default
    if !config_path.exists() {
        save_config(&ClientConfig::default())?;
        return Ok(ClientConfig::default());
    }
    
    // Load config from file
    let config_str = fs::read_to_string(config_path)?;
    let config: ClientConfig = serde_json::from_str(&config_str)?;
    
    Ok(config)
}

pub fn save_config(config: &ClientConfig) -> Result<()> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.json");
    
    let json = serde_json::to_string_pretty(config)?;
    fs::write(config_path, json)?;
    
    Ok(())
}