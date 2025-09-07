use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

const CHECK_SINK_EMPTY_INTERVAL: Duration = Duration::from_secs(1);

struct Inner {
    sink: Option<Sink>,
    stream: Option<OutputStream>,
    current_track: Option<String>,
    current_index: Option<usize>,
    volume: f32,
    position: Duration,
    duration: Option<Duration>,
    playlist: Option<Playlist>,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            sink: None,
            stream: None,
            current_track: None,
            current_index: None,
            volume: 1.0,
            position: Duration::from_secs(0),
            duration: None,
            playlist: None,
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
        let mut check_sink_interval = tokio::time::interval(CHECK_SINK_EMPTY_INTERVAL);
        check_sink_interval.tick().await;

        while !shutdown_token.is_cancelled() {
            tokio::select! {
                () = shutdown_token.cancelled() => {
                    tracing::info!("Shutting down music player");
                },
                _ = check_sink_interval.tick() => {
                    self.play_next();
                },
            }
        }
    }

    fn play_next(&self) {
        let mut should_play_next = false;

        {
            let inner = self.inner.lock().unwrap();
            let playlist = match inner.playlist {
                Some(ref playlist) => playlist,
                None => return,
            };
            if let Some(ref sink) = inner.sink {
                if sink.empty() && !sink.is_paused() && playlist.tracks.len() > 0 {
                    should_play_next = true;
                }
            }
        }

        if should_play_next {
            let _ = self.next();
        }
    }

    fn load_track(&self, track_name: &str, path: &Path) -> Result<()> {
        self.stop()?;

        let mut inner = self.inner.lock().unwrap();
        if inner.stream.is_none() {
            let host = cpal::default_host();
            let mut devices = host.output_devices().expect("No output devices found");
            // Find the output device with the name contains "es3288"
            let device = devices
                .find(|d| d.name().unwrap().contains("es8388"))
                .unwrap();
            let stream = OutputStreamBuilder::from_device(device)
                .unwrap()
                .open_stream()?;
            // let stream = OutputStreamBuilder::open_default_stream()?;
            inner.stream = Some(stream);
        }

        // Load and decode the audio file
        let file = File::open(&path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        let reader = BufReader::new(file);

        // Try to decode with rodio (which uses symphonia internally for many formats)
        let source = Decoder::new(reader)
            .with_context(|| format!("Failed to decode audio file: {}", path.display()))?;

        // Get duration if available
        inner.duration = source.total_duration();

        if let Some(ref stream_handle) = inner.stream {
            let sink = Sink::connect_new(stream_handle.mixer());

            sink.set_volume(inner.volume);
            sink.append(source);
            sink.pause(); // Start paused

            inner.sink = Some(sink);
            inner.current_track = Some(track_name.to_string());
            inner.position = Duration::from_secs(0);
        }
        Ok(())
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

    pub fn play(&self, playlist: &Vec<Track>, selected_index: usize) -> Result<()> {
        {
            let mut inner = self.inner.lock().unwrap();
            inner.playlist = Some(Playlist {
                tracks: playlist.clone(),
            });
            inner.current_index = Some(selected_index);
        }

        let track_name = playlist[selected_index].name.clone();
        let path = PathBuf::from(playlist[selected_index].path.clone());
        self.load_track(&track_name, &path)?;

        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        if let Some(ref sink) = inner.sink {
            sink.play();
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
            sink.clear();
        }

        inner.current_track = None;
        inner.position = Duration::from_secs(0);
        inner.duration = None;
        Ok(())
    }

    pub fn toggle(&self) -> Result<()> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        if let Some(ref sink) = inner.sink {
            if sink.is_paused() {
                sink.play();
            } else {
                sink.pause();
            }
        }
        Ok(())
    }

    pub fn is_paused(&self) -> bool {
        let inner = self.inner.lock();
        match inner {
            Ok(inner) => {
                if let Some(ref sink) = inner.sink {
                    return sink.is_paused();
                } else {
                    return true;
                }
            }
            Err(_) => {
                return true;
            }
        }
    }

    pub fn seek(&self, delta: f32) -> Result<()> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        if let Some(ref sink) = inner.sink {
            let pos = sink.get_pos().as_secs();
            let new_pos = if delta > 0.0 {
                pos as u64 + delta as u64
            } else {
                pos as u64 - delta as u64
            };
            match sink.try_seek(Duration::from_secs(new_pos)) {
                Ok(()) => {}
                Err(_) => {
                    return Err(anyhow::anyhow!("Failed to seek"));
                }
            }
        }
        Ok(())
    }

    pub fn seek_to(&self, seconds: f32) -> Result<()> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        if let Some(ref sink) = inner.sink {
            match sink.try_seek(Duration::from_secs(seconds as u64)) {
                Ok(()) => {}
                Err(_) => {
                    return Err(anyhow::anyhow!("Failed to seek to"));
                }
            }
        }
        Ok(())
    }

    pub fn next(&self) -> Result<()> {
        self.load_next_track()?;

        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        if let Some(ref sink) = inner.sink {
            sink.play();
        }
        Ok(())
    }

    pub fn prev(&self) -> Result<()> {
        self.load_prev_track()?;

        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock inner"))?;
        if let Some(ref sink) = inner.sink {
            sink.play();
        }
        Ok(())
    }

    fn load_next_track(&self) -> Result<()> {
        self.stop()?;

        let mut inner = self.inner.lock().unwrap();

        if inner.current_index.is_none() {
            return Ok(());
        }

        let current_index = inner.current_index.unwrap();
        if current_index + 1 < inner.playlist.as_ref().unwrap().tracks.len() {
            inner.current_index = Some(current_index + 1);
        } else {
            inner.current_index = Some(0);
        }
        let current_index = inner.current_index.unwrap();

        let track_name = inner.playlist.as_ref().unwrap().tracks[current_index]
            .name
            .clone();
        let path = PathBuf::from(
            inner.playlist.as_ref().unwrap().tracks[current_index]
                .path
                .clone(),
        );

        if inner.stream.is_none() {
            let stream = OutputStreamBuilder::open_default_stream()?;
            inner.stream = Some(stream);
        }

        // Load and decode the audio file
        let file = File::open(&path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        let reader = BufReader::new(file);

        // Try to decode with rodio (which uses symphonia internally for many formats)
        let source = Decoder::new(reader)
            .with_context(|| format!("Failed to decode audio file: {}", path.display()))?;

        // Get duration if available
        inner.duration = source.total_duration();

        if let Some(ref stream_handle) = inner.stream {
            let sink = Sink::connect_new(stream_handle.mixer());

            sink.set_volume(inner.volume);
            sink.append(source);
            sink.pause(); // Start paused

            inner.sink = Some(sink);
            inner.current_track = Some(track_name.to_string());
            inner.position = Duration::from_secs(0);
        }

        Ok(())
    }

    fn load_prev_track(&self) -> Result<()> {
        self.stop()?;

        let mut inner = self.inner.lock().unwrap();
        if inner.current_index.is_none() {
            return Ok(());
        }

        let current_index = inner.current_index.unwrap();
        if current_index > 0 {
            inner.current_index = Some(current_index - 1);
        } else {
            inner.current_index = Some(inner.playlist.as_ref().unwrap().tracks.len() - 1);
        }

        let current_index = inner.current_index.unwrap();
        let track_name = inner.playlist.as_ref().unwrap().tracks[current_index]
            .name
            .clone();
        let path = PathBuf::from(
            inner.playlist.as_ref().unwrap().tracks[current_index]
                .path
                .clone(),
        );

        if inner.stream.is_none() {
            let stream = OutputStreamBuilder::open_default_stream()?;
            inner.stream = Some(stream);
        }

        // Load and decode the audio file
        let file = File::open(&path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        let reader = BufReader::new(file);

        // Try to decode with rodio (which uses symphonia internally for many formats)
        let source = Decoder::new(reader)
            .with_context(|| format!("Failed to decode audio file: {}", path.display()))?;

        // Get duration if available
        inner.duration = source.total_duration();

        if let Some(ref stream_handle) = inner.stream {
            let sink = Sink::connect_new(stream_handle.mixer());

            sink.set_volume(inner.volume);
            sink.append(source);
            sink.pause(); // Start paused

            inner.sink = Some(sink);
            inner.current_track = Some(track_name.to_string());
            inner.position = Duration::from_secs(0);
        }

        Ok(())
    }

    // pub fn is_playing(&self) -> bool {
    //     let inner = self.inner.lock().unwrap();
    //     if let Some(ref sink) = inner.sink {
    //         !sink.is_paused() && !sink.empty()
    //     } else {
    //         false
    //     }
    // }

    pub fn status(&self) -> Result<PlayerStatus> {
        let inner = self.inner.lock().unwrap();
        if inner.sink.is_none() {
            return Ok(PlayerStatus {
                paused: true,
                position: None,
                position_sec: None,
                duration: None,
                duration_sec: None,
                volume: 0.0,
                current_track: None,
                track: None,
            });
        }

        let sink = inner.sink.as_ref().unwrap();

        let is_playing = !sink.is_paused() && !sink.empty();
        let pos = sink.get_pos().as_secs();
        let position = format!("{:02}:{:02}", pos / 60, pos % 60);
        let duration = inner
            .duration
            .map(|d| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60));
        let volume = inner.volume;
        let current_track = inner.current_track.clone().map(|p| p);

        Ok(PlayerStatus {
            paused: !is_playing,
            position: Some(position),
            position_sec: Some(pos),
            duration: duration,
            duration_sec: Some(inner.duration.map(|d| d.as_secs()).unwrap_or(0)),
            volume: volume,
            current_track: current_track,
            track: Some(0),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct PlayerStatus {
    pub paused: bool,
    pub position: Option<String>,
    pub position_sec: Option<u64>,
    pub duration: Option<String>,
    pub duration_sec: Option<u64>,
    pub volume: f32,
    pub current_track: Option<String>,
    pub track: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Track {
    name: String,
    path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Playlist {
    tracks: Vec<Track>,
}
