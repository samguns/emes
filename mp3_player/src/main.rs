mod player;
mod ui;
mod playlist;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;

use crate::{
    player::Player,
    playlist::Playlist,
    ui::UI,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to MP3 file or directory containing MP3 files
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,
    
    /// Start playing immediately
    #[arg(short, long)]
    autoplay: bool,
    
    /// Shuffle playlist
    #[arg(short, long)]
    shuffle: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create playlist
    let mut playlist = Playlist::new();
    
    // Load files from path if provided
    if let Some(path) = args.path {
        if path.is_file() {
            playlist.add_file(path)?;
        } else if path.is_dir() {
            playlist.load_directory(path)?;
        }
    } else {
        // Load from current directory if no path specified
        playlist.load_directory(std::env::current_dir()?)?;
    }
    
    if args.shuffle {
        playlist.shuffle();
    }
    
    // Create player
    let player = Arc::new(Mutex::new(Player::new()));
    let ui = Arc::new(Mutex::new(UI::new()));
    
    // Start playing if autoplay is enabled
    if args.autoplay && !playlist.is_empty() {
        if let Some(track) = playlist.current() {
            let mut player_lock = player.lock().await;
            player_lock.load_track(track.path.clone()).await?;
            player_lock.play().await?;
        }
    }
    
    // Main event loop
    let result = run_app(&mut terminal, player, ui, &mut playlist).await;
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    player: Arc<Mutex<Player>>,
    ui: Arc<Mutex<UI>>,
    playlist: &mut Playlist,
) -> Result<()> {
    loop {
        // Update UI state
        {
            let player_lock = player.lock().await;
            let mut ui_lock = ui.lock().await;
            ui_lock.update_player_state(
                player_lock.is_playing(),
                player_lock.get_position(),
                player_lock.get_duration(),
                player_lock.get_volume(),
            );
            ui_lock.update_playlist(playlist.get_tracks(), playlist.current_index());
            if let Some(track) = playlist.current() {
                ui_lock.update_current_track(Some(track.clone()));
            }
        }
        
        // Draw UI
        {
            let ui_lock = ui.lock().await;
            terminal.draw(|f| ui_lock.draw(f))?;
        }
        
        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char(' ') => {
                        // Play/Pause toggle
                        let mut player_lock = player.lock().await;
                        if player_lock.is_playing() {
                            player_lock.pause().await?;
                        } else {
                            player_lock.play().await?;
                        }
                    }
                    KeyCode::Enter => {
                        // Play selected track
                        if let Some(track) = playlist.current() {
                            let mut player_lock = player.lock().await;
                            player_lock.load_track(track.path.clone()).await?;
                            player_lock.play().await?;
                        }
                    }
                    KeyCode::Right => {
                        // Next track
                        if playlist.next() {
                            if let Some(track) = playlist.current() {
                                let mut player_lock = player.lock().await;
                                player_lock.load_track(track.path.clone()).await?;
                                player_lock.play().await?;
                            }
                        }
                    }
                    KeyCode::Left => {
                        // Previous track
                        if playlist.previous() {
                            if let Some(track) = playlist.current() {
                                let mut player_lock = player.lock().await;
                                player_lock.load_track(track.path.clone()).await?;
                                player_lock.play().await?;
                            }
                        }
                    }
                    KeyCode::Up => {
                        // Move selection up in playlist
                        playlist.move_selection_up();
                    }
                    KeyCode::Down => {
                        // Move selection down in playlist
                        playlist.move_selection_down();
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        // Volume up
                        let mut player_lock = player.lock().await;
                        let current = player_lock.get_volume();
                        player_lock.set_volume((current + 0.1).min(1.0)).await?;
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        // Volume down
                        let mut player_lock = player.lock().await;
                        let current = player_lock.get_volume();
                        player_lock.set_volume((current - 0.1).max(0.0)).await?;
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        // Stop
                        let mut player_lock = player.lock().await;
                        player_lock.stop().await?;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        // Toggle repeat
                        playlist.toggle_repeat();
                    }
                    KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => {
                        // Toggle help
                        let mut ui_lock = ui.lock().await;
                        ui_lock.toggle_help();
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}