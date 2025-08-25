use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

use crate::connection::Connection;

// Video configuration constants
const VIDEO_WIDTH: i32 = 640;
const VIDEO_HEIGHT: i32 = 480;
const VIDEO_FRAMERATE: i32 = 30;
const VIDEO_BITRATE: i32 = 1_000_000; // 1 Mbps

#[cfg(feature = "video")]
use gstreamer as gst;
#[cfg(feature = "video")]
use gstreamer_app as gst_app;
#[cfg(feature = "video")]
use gstreamer_video as gst_video;
pub struct VideoManager {
    // State
    active: Arc<AtomicBool>,
    
    // Video device and configuration
    device_name: Option<String>,
    
    // Channels for video data
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    
    // User and channel info
    user_id: Uuid,
    channel_id: Uuid,
    
    // Connection to server
    connection: Arc<Connection>,
    
    // Type of capture
    capture_type: CaptureType,
    
    // Video pipeline (when using gstreamer)
    #[cfg(feature = "video")]
    pipeline: Option<gst::Pipeline>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureType {
    Camera,
    Screen,
}

// VideoPlayback is responsible for rendering received video streams
pub struct VideoPlayback {
    // Video data buffers for each user
    video_buffers: std::collections::HashMap<Uuid, Vec<u8>>,
    
    // Video frame dimensions
    width: i32,
    height: i32,
    
    // Last update time for each user
    last_updates: std::collections::HashMap<Uuid, std::time::Instant>,
}

impl VideoPlayback {
    pub fn new() -> Self {
        Self {
            video_buffers: std::collections::HashMap::new(),
            width: VIDEO_WIDTH,
            height: VIDEO_HEIGHT,
            last_updates: std::collections::HashMap::new(),
        }
    }
    
    pub fn process_video_data(&mut self, user_id: Uuid, data: Vec<u8>) {
        self.video_buffers.insert(user_id, data);
        self.last_updates.insert(user_id, std::time::Instant::now());
    }
    
    pub fn get_video_frame(&self, user_id: Uuid) -> Option<&Vec<u8>> {
        self.video_buffers.get(&user_id)
    }
    
    pub fn get_dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }
    
    pub fn is_active(&self, user_id: Uuid) -> bool {
        if let Some(last_update) = self.last_updates.get(&user_id) {
            // Consider the stream active if we received data in the last 5 seconds
            last_update.elapsed() < Duration::from_secs(5)
        } else {
            false
        }
    }
}

impl VideoManager {
    pub fn new(user_id: Uuid, channel_id: Uuid, connection: Arc<Connection>, capture_type: CaptureType) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(2);
        
        Self {
            active: Arc::new(AtomicBool::new(false)),
            device_name: None,
            tx,
            rx,
            user_id,
            channel_id,
            connection,
            capture_type,
            #[cfg(feature = "video")]
            pipeline: None,
        }
    }
    
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
    
    pub fn set_device(&mut self, device_name: &str) {
        self.device_name = Some(device_name.to_string());
    }
    
    pub fn initialize(&mut self) -> Result<()> {
        // Initialize video backend if needed
        #[cfg(feature = "video")]
        {
            gst::init()?;
        }
        
        Ok(())
    }
    
    pub fn start_camera(&mut self) -> Result<()> {
        if self.is_active() {
            return Ok(());
        }
        
        self.capture_type = CaptureType::Camera;
        self.start_capture()
    }
    
    pub fn start_screen_sharing(&mut self) -> Result<()> {
        if self.is_active() {
            return Ok(());
        }
        
        self.capture_type = CaptureType::Screen;
        self.start_capture()
    }
    
    fn start_capture(&mut self) -> Result<()> {
        // Start sender task for video data
        let rx = self.rx.clone();
        let connection = self.connection.clone();
        let user_id = self.user_id;
        let channel_id = self.channel_id;
        let active = self.active.clone();
        let is_screen_share = self.capture_type == CaptureType::Screen;
        
        #[cfg(feature = "video")]
        {
            // In a real implementation with gstreamer, we would initialize the pipeline here
            // For simplicity, we're omitting the actual video capture code
            tracing::info!("Video capture would be initialized with GStreamer in a full implementation");
        }
        
        // Generate mock video data for demonstration
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            // Generate mock frame data (RGB data)
            let frame_size = (VIDEO_WIDTH * VIDEO_HEIGHT * 3) as usize;
            let mut dummy_frame = vec![0u8; frame_size];
            
            // Generate some pattern for the frame
            for i in 0..frame_size / 3 {
                let x = (i % VIDEO_WIDTH as usize) as f32 / VIDEO_WIDTH as f32;
                let y = (i / VIDEO_WIDTH as usize) as f32 / VIDEO_HEIGHT as f32;
                
                dummy_frame[i * 3] = (x * 255.0) as u8;      // R
                dummy_frame[i * 3 + 1] = (y * 255.0) as u8;  // G
                dummy_frame[i * 3 + 2] = 128;                 // B
            }
            
            // Send a frame periodically
            let _frame_interval = std::time::Duration::from_millis(1000 / VIDEO_FRAMERATE as u64);
            let _ = tx.try_send(dummy_frame);
        });
        
        std::thread::spawn(move || {
            active.store(true, Ordering::SeqCst);
            
            // Send started message
            let started_message = if is_screen_share {
                open_reverb_common::protocol::Message::ScreenShareStarted { user_id }
            } else {
                open_reverb_common::protocol::Message::VideoStarted { user_id }
            };
            
            if let Err(e) = connection.get_sender().send(started_message) {
                tracing::error!("Failed to send video/screenshare started message: {}", e);
            }
            
            while active.load(Ordering::SeqCst) {
                if let Ok(data) = rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    // Send video data
                    let message = if is_screen_share {
                        open_reverb_common::protocol::Message::ScreenShareData {
                            user_id,
                            channel_id,
                            data,
                        }
                    } else {
                        open_reverb_common::protocol::Message::VideoData {
                            user_id,
                            channel_id,
                            data,
                        }
                    };
                    
                    if let Err(e) = connection.get_sender().send(message) {
                        tracing::error!("Failed to send video/screenshare data: {}", e);
                    }
                }
            }
            
            // Send stopped message
            let stopped_message = if is_screen_share {
                open_reverb_common::protocol::Message::ScreenShareStopped { user_id }
            } else {
                open_reverb_common::protocol::Message::VideoStopped { user_id }
            };
            
            if let Err(e) = connection.get_sender().send(stopped_message) {
                tracing::error!("Failed to send video/screenshare stopped message: {}", e);
            }
        });
        
        Ok(())
    }
    
    pub fn stop(&mut self) {
        self.active.store(false, Ordering::SeqCst);
        
        #[cfg(feature = "video")]
        if let Some(pipeline) = &self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
            self.pipeline = None;
        }
    }
    
    pub fn get_available_video_devices() -> Vec<String> {
        // In a real implementation, we would enumerate available video devices
        vec!["Default Camera".to_string(), "External Webcam".to_string()]
    }
    
    pub fn get_available_screens() -> Vec<String> {
        // For screen sharing, we typically just return a list of monitors
        vec!["Primary Display".to_string(), "Secondary Display".to_string()]
    }
}