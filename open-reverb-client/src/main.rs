mod app;
mod audio;
mod config;
mod connection;
mod ui;
mod video;

use anyhow::Result;
use eframe::NativeOptions;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting Open Reverb Client version {}", open_reverb_common::version());
    
    // Set up GUI window options
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 720.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        centered: true,
        ..Default::default()
    };
    
    // Launch the GUI
    eframe::run_native(
        "Open Reverb",
        options,
        Box::new(|cc| Box::new(app::DemoApp::new(cc))),
    )?;
    
    Ok(())
}