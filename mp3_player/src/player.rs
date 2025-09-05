use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, Sink, Source};
use cpal::{Device, traits::{DeviceTrait, HostTrait}};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

pub struct Player {
    sink: Option<Arc<RwLock<Sink>>>,
    stream: Option<OutputStream>,
    current_track: Option<PathBuf>,
    volume: f32,
    position: Arc<RwLock<Duration>>,
    duration: Option<Duration>,
    current_device: Option<Device>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            sink: None,
            stream: None,
            current_track: None,
            volume: 0.5,
            position: Arc::new(RwLock::new(Duration::from_secs(0))),
            duration: None,
            current_device: None,
        }
    }

    /// Get a list of available audio output devices
    pub fn list_output_devices() -> Result<Vec<(String, Device)>> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        
        for device in host.output_devices()? {
            if let Ok(name) = device.name() {
                devices.push((name, device));
            }
        }
        
        Ok(devices)
    }

    /// Get the default audio output device
    pub fn get_default_device() -> Result<Device> {
        let host = cpal::default_host();
        host.default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No default audio output device available"))
    }

    /// Set the audio output device by name
    pub async fn set_device_by_name(&mut self, device_name: &str) -> Result<()> {
        let devices = Self::list_output_devices()?;
        
        for (name, device) in devices {
            if name.contains(device_name) || name == device_name {
                return self.set_device(device).await;
            }
        }
        
        Err(anyhow::anyhow!("Audio device '{}' not found", device_name))
    }

    /// Set the audio output device
    pub async fn set_device(&mut self, device: Device) -> Result<()> {
        // Stop current playbook
        self.stop().await?;
        
        // Close current stream
        self.stream = None;
        
        // Store the device for later use when creating stream
        self.current_device = Some(device);
        
        Ok(())
    }

    /// Get the current audio device name
    pub fn get_current_device_name(&self) -> Result<String> {
        match &self.current_device {
            Some(device) => device.name().context("Failed to get device name"),
            None => Ok("Default".to_string()),
        }
    }

    pub async fn load_track(&mut self, path: PathBuf) -> Result<()> {
        // Stop current playback if any
        self.stop().await?;

        // Create output stream if not exists
        if self.stream.is_none() {
            // For now, use default stream - specific device support will be added
            // when we find the correct rodio API
            let (_stream, stream_handle) = rodio::OutputStream::try_default()
                .context("Failed to create default audio output stream")?;
            self.stream = Some(_stream);
            // Note: We'll need to store stream_handle for sink creation
        }

        // Load and decode the audio file
        let file = File::open(&path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        let reader = BufReader::new(file);

        // Try to decode with rodio (which uses symphonia internally for many formats)
        let source = Decoder::new(reader)
            .with_context(|| format!("Failed to decode audio file: {}", path.display()))?;

        // Get duration if available
        self.duration = source.total_duration();

        // TODO: Fix sink creation with proper stream handle
        // For now, create sink with default stream
        // This is a placeholder until we get the correct rodio API
        
        // self.sink = Some(Arc::new(RwLock::new(sink)));
        // self.current_track = Some(path);
        // *self.position.write().unwrap() = Duration::from_secs(0);
        
        println!("Note: Sink creation needs to be implemented with correct rodio API");

        Ok(())
    }

    pub async fn play(&mut self) -> Result<()> {
        if let Some(ref sink) = self.sink.as_ref() {
            let sink = sink.write().unwrap();
            sink.play();
        }
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        if let Some(ref sink) = self.sink.as_ref() {
            let sink = sink.write().unwrap();
            sink.pause();
        }
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(sink) = self.sink.as_ref() {
            let sink = sink.write().unwrap();
            sink.stop();
        }
        self.current_track = None;
        *self.position.write().unwrap() = Duration::from_secs(0);
        self.duration = None;
        Ok(())
    }

    pub fn is_playing(&self) -> bool {
        if let Some(ref sink) = self.sink.as_ref() {
            let sink = sink.read().unwrap();
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }

    pub fn get_position(&self) -> Duration {
        // Note: elapsed() method doesn't exist in rodio 0.19
        // We would need to track position manually or use a different approach
        Duration::from_secs(0)
    }

    pub fn get_duration(&self) -> Option<Duration> {
        self.duration
    }

    pub async fn seek(&mut self, position: Duration) -> Result<()> {
        if let Some(ref sink) = self.sink.as_ref() {
            let sink = sink.write().unwrap();
            // Note: try_seek returns a Result with SeekError which doesn't implement std::error::Error
            // We'll handle it differently
            match sink.try_seek(position) {
                Ok(()) => {
                    *self.position.write().unwrap() = position;
                }
                Err(_) => {
                    return Err(anyhow::anyhow!("Failed to seek to position"));
                }
            }
        }
        Ok(())
    }

    pub async fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(ref sink) = self.sink.as_ref() {
            let sink = sink.write().unwrap();
            sink.set_volume(self.volume);
        }
        Ok(())
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn get_current_track(&self) -> Option<&Path> {
        self.current_track.as_deref()
    }

    pub fn has_ended(&self) -> bool {
        if let Some(ref sink) = self.sink.as_ref() {
            let sink = sink.read().unwrap();
            sink.empty()
        } else {
            false
        }
    }
}
