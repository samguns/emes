use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
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
}

impl Player {
    pub fn new() -> Self {
        Self {
            sink: None,
            stream: None,
            // stream_handle: None,
            current_track: None,
            volume: 0.5,
            position: Arc::new(RwLock::new(Duration::from_secs(0))),
            duration: None,
        }
    }

    pub async fn load_track(&mut self, path: PathBuf) -> Result<()> {
        // Stop current playback if any
        self.stop().await?;

        // Create output stream if not exists
        if self.stream.is_none() {
            let stream = OutputStreamBuilder::open_default_stream()?;
            self.stream = Some(stream);
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

        // Create new sink and append the source
        if let Some(ref stream_handle) = self.stream {
            let sink = Sink::connect_new(stream_handle.mixer());

            sink.set_volume(self.volume);
            sink.append(source);
            sink.pause(); // Start paused

            self.sink = Some(Arc::new(RwLock::new(sink)));
            self.current_track = Some(path);
            *self.position.write().unwrap() = Duration::from_secs(0);
        }

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
