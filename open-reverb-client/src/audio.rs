use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use std::sync::{mpsc, atomic::{AtomicBool, Ordering}, Arc};
use std::time::Duration;
use uuid::Uuid;

use crate::connection::Connection;

// Sample rate and buffer size for audio processing
const SAMPLE_RATE: u32 = 48000;
const CHANNELS: u16 = 1;
const BUFFER_SIZE: usize = 960; // 20ms at 48kHz

#[cfg(feature = "audio")]
use cpal::{self, traits::{DeviceTrait, HostTrait, StreamTrait}};
#[cfg(feature = "audio")]
use cpal::{InputCallbackInfo, OutputCallbackInfo, SampleFormat, Stream};

pub struct AudioManager {
    // State
    active: Arc<AtomicBool>,
    
    // Audio device streams
    #[cfg(feature = "audio")]
    input_stream: Option<Stream>,
    #[cfg(feature = "audio")]
    output_streams: Vec<Stream>,
    #[cfg(not(feature = "audio"))]
    mock_audio_thread: Option<std::thread::JoinHandle<()>>,
    #[cfg(not(feature = "audio"))]
    mock_audio_stop: Option<mpsc::Sender<()>>,
    
    // Channels for audio data
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    
    // User and channel info
    user_id: Uuid,
    channel_id: Uuid,
    
    // Connection to server
    connection: Arc<Connection>,
}

impl AudioManager {
    pub fn new(user_id: Uuid, channel_id: Uuid, connection: Arc<Connection>) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(10);
        
        Self {
            active: Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "audio")]
            input_stream: None,
            #[cfg(feature = "audio")]
            output_streams: Vec::new(),
            #[cfg(not(feature = "audio"))]
            mock_audio_thread: None,
            #[cfg(not(feature = "audio"))]
            mock_audio_stop: None,
            tx,
            rx,
            user_id,
            channel_id,
            connection,
        }
    }
    
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
    
    pub fn start_audio(&mut self) -> Result<()> {
        if self.is_active() {
            return Ok(());
        }
        
        #[cfg(feature = "audio")]
        {
            // Initialize audio with cpal
            let host = cpal::default_host();
            
            // Set up input device
            let input_device = host.default_input_device().ok_or_else(|| {
                anyhow::anyhow!("No input device found")
            })?;
            
            let input_config = input_device.default_input_config()?;
            
            // Set up input stream based on sample format
            match input_config.sample_format() {
                SampleFormat::F32 => self.setup_input_stream::<f32>(&input_device)?,
                SampleFormat::I16 => self.setup_input_stream::<i16>(&input_device)?,
                SampleFormat::U16 => self.setup_input_stream::<u16>(&input_device)?,
                format => return Err(anyhow::anyhow!("Unsupported sample format: {:?}", format)),
            }
            
            // Set up output device
            let output_device = host.default_output_device().ok_or_else(|| {
                anyhow::anyhow!("No output device found")
            })?;
            
            let output_config = output_device.default_output_config()?;
            
            // Set up output stream based on sample format
            match output_config.sample_format() {
                SampleFormat::F32 => self.setup_output_stream::<f32>(&output_device)?,
                SampleFormat::I16 => self.setup_output_stream::<i16>(&output_device)?,
                SampleFormat::U16 => self.setup_output_stream::<u16>(&output_device)?,
                format => return Err(anyhow::anyhow!("Unsupported sample format: {:?}", format)),
            }
        }
        
        #[cfg(not(feature = "audio"))]
        {
            // Mock audio implementation for builds without audio support
            let (stop_tx, stop_rx) = mpsc::channel::<()>();
            self.mock_audio_stop = Some(stop_tx);
            
            let tx = self.tx.clone();
            
            // Create a thread that generates mock audio data
            let handle = std::thread::spawn(move || {
                let sample_interval = Duration::from_millis(20); // 20ms chunks
                let mut sample_data = vec![0u8; BUFFER_SIZE * 2]; // 16-bit samples
                
                loop {
                    // Generate a simple sine wave
                    for i in 0..BUFFER_SIZE {
                        let t = i as f32 / SAMPLE_RATE as f32;
                        let value = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.1;
                        let sample = (value * 32767.0) as i16;
                        sample_data[i * 2] = (sample & 0xFF) as u8;
                        sample_data[i * 2 + 1] = ((sample >> 8) & 0xFF) as u8;
                    }
                    
                    let _ = tx.try_send(sample_data.clone());
                    
                    // Check if we should stop
                    if stop_rx.try_recv().is_ok() {
                        break;
                    }
                    
                    std::thread::sleep(sample_interval);
                }
            });
            
            self.mock_audio_thread = Some(handle);
        }
        
        // Start sender task
        let rx = self.rx.clone();
        let connection = self.connection.clone();
        let user_id = self.user_id;
        let channel_id = self.channel_id;
        let active = self.active.clone();
        
        std::thread::spawn(move || {
            active.store(true, Ordering::SeqCst);
            
            // Send "voice started" message
            let voice_started = open_reverb_common::protocol::Message::VoiceStarted { user_id };
            if let Err(e) = connection.get_sender().send(voice_started) {
                tracing::error!("Failed to send voice started message: {}", e);
            }
            
            while active.load(Ordering::SeqCst) {
                if let Ok(data) = rx.recv() {
                    if let Err(e) = connection.get_sender().send(open_reverb_common::protocol::Message::VoiceData { user_id, channel_id, data }) {
                        tracing::error!("Failed to send voice data: {}", e);
                    }
                }
            }
            
            // Send "voice stopped" message
            let voice_stopped = open_reverb_common::protocol::Message::VoiceStopped { user_id };
            if let Err(e) = connection.get_sender().send(voice_stopped) {
                tracing::error!("Failed to send voice stopped message: {}", e);
            }
        });
        
        Ok(())
    }
    
    pub fn stop_audio(&mut self) {
        self.active.store(false, Ordering::SeqCst);
        
        #[cfg(feature = "audio")]
        {
            self.input_stream = None;
            self.output_streams.clear();
        }
        
        #[cfg(not(feature = "audio"))]
        {
            if let Some(stop_tx) = &self.mock_audio_stop {
                let _ = stop_tx.send(());
            }
            
            if let Some(handle) = self.mock_audio_thread.take() {
                let _ = handle.join();
            }
            
            self.mock_audio_stop = None;
        }
    }
    
    #[cfg(feature = "audio")]
    fn setup_input_stream<T>(&mut self, device: &cpal::Device) -> Result<()>
    where
        T: cpal::Sample + Send + 'static,
    {
        let config = cpal::StreamConfig {
            channels: CHANNELS,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Fixed(BUFFER_SIZE as u32),
        };
        
        let tx = self.tx.clone();
        
        let input_stream = device.build_input_stream(
            &config,
            move |data: &[T], _: &InputCallbackInfo| {
                // Convert samples to i16 bytes
                let bytes: Vec<u8> = data
                    .iter()
                    .map(|sample| {
                        let value = sample.to_i16();
                        [value as u8, (value >> 8) as u8]
                    })
                    .flatten()
                    .collect();
                
                // Send bytes to sender task
                let _ = tx.try_send(bytes);
            },
            move |err| {
                tracing::error!("Error in input stream: {}", err);
            },
        )?;
        
        input_stream.play()?;
        self.input_stream = Some(input_stream);
        
        Ok(())
    }
    
    #[cfg(feature = "audio")]
    fn setup_output_stream<T>(&mut self, device: &cpal::Device) -> Result<()>
    where
        T: cpal::Sample + Send + 'static,
    {
        let config = cpal::StreamConfig {
            channels: CHANNELS,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Fixed(BUFFER_SIZE as u32),
        };
        
        // This is a placeholder for handling incoming audio data
        // In a real implementation, we would have a buffer for each user
        // and mix them together for output
        let output_stream = device.build_output_stream(
            &config,
            move |data: &mut [T], _: &OutputCallbackInfo| {
                // Fill buffer with silence for now
                for sample in data.iter_mut() {
                    *sample = T::from(&0i16);
                }
            },
            move |err| {
                tracing::error!("Error in output stream: {}", err);
            },
        )?;
        
        output_stream.play()?;
        self.output_streams.push(output_stream);
        
        Ok(())
    }
}