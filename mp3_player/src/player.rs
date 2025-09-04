use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;

pub struct Player {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
    current_track: Option<PathBuf>,
    volume: f32,
    position: Arc<RwLock<Duration>>,
    duration: Option<Duration>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            sink: None,
            _stream: None,
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
        if self._stream.is_none() {
            let stream = OutputStreamBuilder::open_default_stream()?;
            self._stream = Some(stream);
            // self.stream_handle = Some(stream_handle);
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
        if let Some(ref stream_handle) = self._stream {
            let sink = Sink::connect_new(stream_handle.mixer());

            sink.set_volume(self.volume);
            sink.append(source);
            sink.pause(); // Start paused

            self.sink = Some(sink);
            self.current_track = Some(path);
            *self.position.write().await = Duration::from_secs(0);
        }

        Ok(())
    }

    pub async fn play(&mut self) -> Result<()> {
        if let Some(ref sink) = self.sink {
            sink.play();
        }
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        if let Some(ref sink) = self.sink {
            sink.pause();
        }
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.current_track = None;
        *self.position.write().await = Duration::from_secs(0);
        self.duration = None;
        Ok(())
    }

    pub fn is_playing(&self) -> bool {
        if let Some(ref sink) = self.sink {
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }

    pub fn get_position(&self) -> Duration {
        // Note: elapsed() method doesn't exist in rodio 0.19
        // We would need to track position manually or use a different approach
        // Duration::from_secs(0)
    }

    pub fn get_duration(&self) -> Option<Duration> {
        self.duration
    }

    pub async fn seek(&mut self, position: Duration) -> Result<()> {
        if let Some(ref sink) = self.sink {
            // Note: try_seek returns a Result with SeekError which doesn't implement std::error::Error
            // We'll handle it differently
            match sink.try_seek(position) {
                Ok(()) => {
                    *self.position.write().await = position;
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
        if let Some(ref sink) = self.sink {
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
        if let Some(ref sink) = self.sink {
            sink.empty()
        } else {
            false
        }
    }
}
