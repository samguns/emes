use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

struct Inner {
    sink: Option<Sink>,
    stream_handle: Option<OutputStreamBuilder>,
    current_track: Option<PathBuf>,
    volume: f32,
    position: Duration,
    duration: Option<Duration>,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            sink: None,
            stream_handle: None,
            current_track: None,
            volume: 0.5,
            position: Duration::from_secs(0),
            duration: None,
        }
    }
}

pub struct MusicPlayer {
    inner: Arc<Mutex<Inner>>,
}

impl MusicPlayer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
        }
    }

    pub async fn run(&self, shutdown_token: CancellationToken) {
        while !shutdown_token.is_cancelled() {
            tokio::select! {
                () = shutdown_token.cancelled() => {
                    tracing::info!("Shutting down music player");
                },
            }
        }
    }

    pub async fn load_track(&self, path: &PathBuf) -> Result<()> {
        self.stop()?;
        Ok(())
    }

    pub fn get_volume(&self) -> f32 {
        let inner = self.inner.lock().unwrap();
        inner.volume
    }

    pub fn set_volume(&self, volume: f32) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        inner.volume = volume.clamp(0.0, 1.0);
        if let Some(ref sink) = inner.sink {
            sink.set_volume(inner.volume);
        }
        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        if let Some(ref sink) = inner.sink {
            sink.play();
        }
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        if let Some(ref sink) = inner.sink {
            sink.pause();
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        if let Some(ref sink) = inner.sink {
            sink.stop();
        }

        inner.current_track = None;
        inner.position = Duration::from_secs(0);
        inner.duration = None;
        Ok(())
    }

    pub fn is_playing(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        if let Some(ref sink) = inner.sink {
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }
}
