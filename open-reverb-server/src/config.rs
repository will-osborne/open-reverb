use std::error::Error;
use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub database_url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            max_connections: 1000,
            database_url: "sqlite::memory:".to_string(),
        }
    }
}

lazy_static! {
    static ref CONFIG: ServerConfig = load_config().unwrap_or_default();
}

pub fn load_config() -> Result<ServerConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::with_name("config/default").required(false))
        .add_source(File::with_name("config/local").required(false))
        .build()?;
    
    config.try_deserialize()
}

pub fn get_config() -> &'static ServerConfig {
    &CONFIG
}