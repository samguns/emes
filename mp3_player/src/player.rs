use crate::audio::AudioOutput;
// use crate::cli::{PlayerStatus, PlaybackState};
use crate::decoder::AudioDecoder;
use crate::error::{PlayerError, Result};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

pub struct Mp3Player {
    audio_output: Arc<AudioOutput>,
    playlist: Arc<RwLock<Playlist>>,
    state: Arc<RwLock<PlayerState>>,
    decode_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    playback_start_time: Arc<Mutex<Option<Instant>>>,
    playback_position: Arc<Mutex<f64>>,
}

#[derive(Debug)]
struct Playlist {
    tracks: VecDeque<PathBuf>,
    current_index: usize,
    shuffle: bool,
    loop_enabled: bool,
    original_order: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerState {
    Stopped,
    Playing,
    Paused,
    Loading,
}

impl Mp3Player {
    pub async fn new() -> Result<Self> {
        let audio_output = Arc::new(AudioOutput::new()?);
        let playlist = Arc::new(RwLock::new(Playlist::new()));
        let state = Arc::new(RwLock::new(PlayerState::Stopped));
        let decode_handle = Arc::new(Mutex::new(None));
        let playback_start_time = Arc::new(Mutex::new(None));
        let playback_position = Arc::new(Mutex::new(0.0));

        Ok(Mp3Player {
            audio_output,
            playlist,
            state,
            decode_handle,
            playback_start_time,
            playback_position,
        })
    }

    pub async fn load_path(&mut self, path: &Path, shuffle: bool) -> Result<()> {
        let mut playlist = self.playlist.write().await;
        
        if path.is_file() {
            if is_audio_file(path) {
                playlist.add_track(path.to_path_buf());
                info!("Loaded single file: {:?}", path);
            } else {
                return Err(PlayerError::InvalidFormat(format!("{:?}", path)));
            }
        } else if path.is_dir() {
            let mut tracks = Vec::new();
            
            for entry in WalkDir::new(path).follow_links(true) {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() && is_audio_file(path) {
                            tracks.push(path.to_path_buf());
                        }
                    }
                    Err(e) => {
                        warn!("Error walking directory: {}", e);
                    }
                }
            }
            
            if tracks.is_empty() {
                return Err(PlayerError::NoAudioTracks);
            }
            
            tracks.sort();
            for track in tracks {
                playlist.add_track(track);
            }
            
            info!("Loaded {} tracks from directory: {:?}", playlist.len(), path);
        } else {
            return Err(PlayerError::FileNotFound(format!("{:?}", path)));
        }
        
        playlist.set_shuffle(shuffle);
        Ok(())
    }

    pub async fn play(&mut self) -> Result<()> {
        let mut state = self.state.write().await;
        
        match *state {
            PlayerState::Paused => {
                // Resume playback
                self.audio_output.play()?;
                *state = PlayerState::Playing;
                
                // Update playback start time
                if let Ok(mut start_time) = self.playback_start_time.lock() {
                    let position = *self.playback_position.lock().unwrap();
                    *start_time = Some(Instant::now() - std::time::Duration::from_secs_f64(position));
                }
                
                info!("Playback resumed");
                return Ok(());
            }
            PlayerState::Playing => {
                info!("Already playing");
                return Ok(());
            }
            _ => {}
        }

        *state = PlayerState::Loading;
        drop(state);

        // Get current track
        let current_track = {
            let playlist = self.playlist.read().await;
            playlist.current_track().cloned()
        };

        if let Some(track_path) = current_track {
            self.play_track(&track_path).await?;
        } else {
            return Err(PlayerError::NoAudioTracks);
        }

        Ok(())
    }

    async fn play_track(&self, path: &Path) -> Result<()> {
        info!("Playing track: {:?}", path);

        // Stop any existing playback
        self.stop_current_decode().await;

        // Create decoder
        let mut decoder = AudioDecoder::new(path)?;
        let (tx, rx) = std::sync::mpsc::channel();

        // Start audio output
        self.audio_output.play_stream(rx).await?;

        // Start decoding in background using spawn_blocking for FFmpeg operations
        let decode_handle = tokio::task::spawn_blocking(move || {
            if let Err(e) = decoder.decode_stream_sync(tx) {
                error!("Decode error: {}", e);
            }
        });

        // Store decode handle
        if let Ok(mut handle) = self.decode_handle.lock() {
            *handle = Some(decode_handle);
        }

        // Update state
        {
            let mut state = self.state.write().await;
            *state = PlayerState::Playing;
        }

        // Reset position tracking
        {
            let mut start_time = self.playback_start_time.lock().unwrap();
            *start_time = Some(Instant::now());
            let mut position = self.playback_position.lock().unwrap();
            *position = 0.0;
        }

        self.audio_output.play()?;
        info!("Track playback started");

        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        let mut state = self.state.write().await;
        
        if *state == PlayerState::Playing {
            self.audio_output.pause()?;
            *state = PlayerState::Paused;
            
            // Update position
            if let (Ok(start_time), Ok(mut position)) = (
                self.playback_start_time.lock(),
                self.playback_position.lock()
            ) {
                if let Some(start) = *start_time {
                    *position = start.elapsed().as_secs_f64();
                }
            }
            
            info!("Playback paused");
        }
        
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.stop_current_decode().await;
        self.audio_output.stop()?;

        {
            let mut state = self.state.write().await;
            *state = PlayerState::Stopped;
        }

        // Reset position
        {
            let mut start_time = self.playback_start_time.lock().unwrap();
            *start_time = None;
            let mut position = self.playback_position.lock().unwrap();
            *position = 0.0;
        }

        info!("Playback stopped");
        Ok(())
    }

    pub async fn next(&mut self) -> Result<()> {
        {
            let mut playlist = self.playlist.write().await;
            playlist.next();
        }
        
        if self.is_playing().await {
            self.play().await?;
        }
        
        Ok(())
    }

    pub async fn previous(&mut self) -> Result<()> {
        {
            let mut playlist = self.playlist.write().await;
            playlist.previous();
        }
        
        if self.is_playing().await {
            self.play().await?;
        }
        
        Ok(())
    }

    pub async fn seek(&mut self, _position: f64) -> Result<()> {
        // TODO: Implement seeking in decoder
        warn!("Seeking not yet implemented");
        Ok(())
    }

    pub async fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.audio_output.set_volume(volume)
    }

    pub async fn get_volume(&self) -> f32 {
        self.audio_output.get_volume()
    }

    pub async fn get_position(&self) -> f64 {
        if let (Ok(start_time), Ok(position)) = (
            self.playback_start_time.lock(),
            self.playback_position.lock()
        ) {
            if let Some(start) = *start_time {
                match *self.state.read().await {
                    PlayerState::Playing => start.elapsed().as_secs_f64(),
                    _ => *position,
                }
            } else {
                *position
            }
        } else {
            0.0
        }
    }

    pub async fn get_duration(&self) -> f64 {
        let playlist = self.playlist.read().await;
        if let Some(track) = playlist.current_track() {
            if let Ok(decoder) = AudioDecoder::new(track) {
                return decoder.duration();
            }
        }
        0.0
    }

    pub fn set_loop(&mut self, enabled: bool) {
        tokio::spawn({
            let playlist = self.playlist.clone();
            async move {
                let mut playlist = playlist.write().await;
                playlist.set_loop(enabled);
            }
        });
    }

    pub async fn is_playing(&self) -> bool {
        *self.state.read().await == PlayerState::Playing
    }

    pub async fn wait_for_completion(&self) -> Result<()> {
        // Wait for current track to finish
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            
            if self.audio_output.empty() && *self.state.read().await == PlayerState::Playing {
                // Track finished, check if we should play next
                let should_continue = {
                    let mut playlist = self.playlist.write().await;
                    if playlist.loop_enabled {
                        playlist.next();
                        true
                    } else if playlist.has_next() {
                        playlist.next();
                        true
                    } else {
                        false
                    }
                };
                
                if should_continue {
                    if let Some(next_track) = {
                        let playlist = self.playlist.read().await;
                        playlist.current_track().cloned()
                    } {
                        self.play_track(&next_track).await?;
                    }
                } else {
                    break;
                }
            }
            
            if *self.state.read().await == PlayerState::Stopped {
                break;
            }
        }
        
        Ok(())
    }

    pub async fn print_status(&self) {
        let state = *self.state.read().await;
        let playlist = self.playlist.read().await;
        let position = self.get_position().await;
        let duration = self.get_duration().await;
        let volume = self.get_volume().await;
        
        println!("=== Player Status ===");
        println!("State: {:?}", state);
        if let Some(track) = playlist.current_track() {
            println!("Current track: {:?}", track.file_name().unwrap_or_default());
            println!("Position: {:.1}s / {:.1}s", position, duration);
        }
        println!("Volume: {:.1}%", volume * 100.0);
        println!("Playlist: {} tracks", playlist.len());
        println!("Loop: {}, Shuffle: {}", playlist.loop_enabled, playlist.shuffle);
    }

    pub fn print_playlist(&self) {
        tokio::spawn({
            let playlist = self.playlist.clone();
            async move {
                let playlist = playlist.read().await;
                println!("=== Playlist ({} tracks) ===", playlist.len());
                for (i, track) in playlist.tracks.iter().enumerate() {
                    let marker = if i == playlist.current_index { ">" } else { " " };
                    println!("{} {}: {:?}", marker, i + 1, track.file_name().unwrap_or_default());
                }
            }
        });
    }

    async fn stop_current_decode(&self) {
        if let Ok(mut handle) = self.decode_handle.lock() {
            if let Some(h) = handle.take() {
                h.abort();
                debug!("Stopped current decode task");
            }
        }
    }
}

impl Playlist {
    fn new() -> Self {
        Self {
            tracks: VecDeque::new(),
            current_index: 0,
            shuffle: false,
            loop_enabled: false,
            original_order: Vec::new(),
        }
    }

    fn add_track(&mut self, track: PathBuf) {
        self.tracks.push_back(track.clone());
        self.original_order.push(track);
    }

    fn current_track(&self) -> Option<&PathBuf> {
        self.tracks.get(self.current_index)
    }

    fn next(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if self.current_index + 1 < self.tracks.len() {
            self.current_index += 1;
        } else if self.loop_enabled {
            self.current_index = 0;
        }
    }

    fn previous(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if self.current_index > 0 {
            self.current_index -= 1;
        } else if self.loop_enabled {
            self.current_index = self.tracks.len() - 1;
        }
    }

    fn has_next(&self) -> bool {
        self.current_index + 1 < self.tracks.len()
    }

    fn set_shuffle(&mut self, enabled: bool) {
        if enabled && !self.shuffle {
            // Enable shuffle
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            let mut tracks: Vec<_> = self.tracks.clone().into();
            tracks.shuffle(&mut rng);
            self.tracks = tracks.into();
            self.current_index = 0;
        } else if !enabled && self.shuffle {
            // Disable shuffle - restore original order
            self.tracks = self.original_order.clone().into();
            self.current_index = 0;
        }
        self.shuffle = enabled;
    }

    fn set_loop(&mut self, enabled: bool) {
        self.loop_enabled = enabled;
    }

    fn len(&self) -> usize {
        self.tracks.len()
    }
}

fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            matches!(ext_str.to_lowercase().as_str(), "mp3" | "flac" | "ogg" | "wav" | "aac" | "m4a")
        } else {
            false
        }
    } else {
        false
    }
}