use crate::error::{PlayerError, Result};
use crate::decoder::AudioChunk;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::sync::{Arc, Mutex};
use std::time::Duration;
// use tokio::sync::mpsc;
use tracing::{debug, info, warn};

pub struct AudioOutput {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,
    volume: Arc<Mutex<f32>>,
}

impl AudioOutput {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| PlayerError::AudioDevice(e.to_string()))?;
        
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| PlayerError::AudioDevice(e.to_string()))?;
        
        let sink = Arc::new(Mutex::new(sink));
        let volume = Arc::new(Mutex::new(1.0));
        
        info!("Audio output initialized");
        
        Ok(AudioOutput {
            _stream,
            stream_handle,
            sink,
            volume,
        })
    }
    
    pub async fn play_stream(&self, rx: std::sync::mpsc::Receiver<AudioChunk>) -> Result<()> {
        let sink = self.sink.clone();
        let volume = self.volume.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut buffer = Vec::new();
            let mut sample_rate = 44100;
            let mut channels = 2;
            
            while let Ok(chunk) = rx.recv() {
                sample_rate = chunk.sample_rate;
                channels = chunk.channels;
                buffer.extend_from_slice(&chunk.data);
                
                // Process buffer when we have enough samples (e.g., 1024 samples per channel)
                let samples_per_channel = 1024;
                let total_samples = samples_per_channel * channels as usize;
                
                while buffer.len() >= total_samples {
                    let chunk_data: Vec<f32> = buffer.drain(..total_samples).collect();
                    let audio_source = AudioSource::new(chunk_data, sample_rate, channels);
                    
                    // Apply volume
                    let vol = *volume.lock().unwrap();
                    let audio_source = audio_source.amplify(vol);
                    
                    if let Ok(sink_guard) = sink.lock() {
                        sink_guard.append(audio_source);
                    }
                }
            }
            
            // Process remaining samples
            if !buffer.is_empty() {
                let audio_source = AudioSource::new(buffer, sample_rate, channels);
                let vol = *volume.lock().unwrap();
                let audio_source = audio_source.amplify(vol);
                
                if let Ok(sink_guard) = sink.lock() {
                    sink_guard.append(audio_source);
                }
            }
            
            debug!("Audio stream processing completed");
        });
        
        Ok(())
    }
    
    pub fn play(&self) -> Result<()> {
        if let Ok(sink) = self.sink.lock() {
            sink.play();
            debug!("Audio playback started");
        }
        Ok(())
    }
    
    pub fn pause(&self) -> Result<()> {
        if let Ok(sink) = self.sink.lock() {
            sink.pause();
            debug!("Audio playback paused");
        }
        Ok(())
    }
    
    pub fn stop(&self) -> Result<()> {
        if let Ok(sink) = self.sink.lock() {
            sink.stop();
            debug!("Audio playback stopped");
        }
        Ok(())
    }
    
    pub fn set_volume(&self, volume: f32) -> Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        
        if let Ok(mut vol) = self.volume.lock() {
            *vol = clamped_volume;
        }
        
        if let Ok(sink) = self.sink.lock() {
            sink.set_volume(clamped_volume);
        }
        
        debug!("Volume set to: {:.2}", clamped_volume);
        Ok(())
    }
    
    pub fn get_volume(&self) -> f32 {
        self.volume.lock().map(|vol| *vol).unwrap_or_else(|_| {
            warn!("Failed to get volume, returning default");
            1.0
        })
    }
    
    pub fn is_paused(&self) -> bool {
        self.sink.lock()
            .map(|sink| sink.is_paused())
            .unwrap_or(false)
    }
    
    pub fn empty(&self) -> bool {
        self.sink.lock()
            .map(|sink| sink.empty())
            .unwrap_or(true)
    }
    
    pub fn len(&self) -> usize {
        self.sink.lock()
            .map(|sink| sink.len())
            .unwrap_or(0)
    }
}

// Custom audio source for rodio
struct AudioSource {
    data: Vec<f32>,
    position: usize,
    sample_rate: u32,
    channels: u16,
}

impl AudioSource {
    fn new(data: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self {
            data,
            position: 0,
            sample_rate,
            channels,
        }
    }
}

impl Iterator for AudioSource {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.data.len() {
            let sample = self.data[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for AudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.data.len() - self.position)
    }
    
    fn channels(&self) -> u16 {
        self.channels
    }
    
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    fn total_duration(&self) -> Option<Duration> {
        let samples_per_channel = self.data.len() / self.channels as usize;
        let duration_secs = samples_per_channel as f64 / self.sample_rate as f64;
        Some(Duration::from_secs_f64(duration_secs))
    }
}

unsafe impl Send for AudioOutput {}
unsafe impl Sync for AudioOutput {}