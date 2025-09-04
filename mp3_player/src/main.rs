use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio::signal;
use tracing::{info, error};

mod player;
mod decoder;
mod audio;
mod cli;
mod error;

use player::Mp3Player;
// use cli::PlayerCommand;

#[derive(Parser)]
#[command(name = "mp3_player")]
#[command(about = "A high-performance MP3 player built with Rust, Tokio, and FFmpeg")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Play an MP3 file or directory
    Play {
        /// Path to MP3 file or directory
        path: PathBuf,
        /// Loop playback
        #[arg(short, long)]
        loop_playback: bool,
        /// Shuffle playback (for directories)
        #[arg(short, long)]
        shuffle: bool,
    },
    /// Interactive player mode
    Interactive {
        /// Path to MP3 file or directory
        path: PathBuf,
    },
    /// List supported audio formats
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    info!("Starting MP3 Player");

    match cli.command {
        Commands::Play { path, loop_playback, shuffle } => {
            let mut player = Mp3Player::new().await?;
            player.load_path(&path, shuffle).await?;
            
            if loop_playback {
                player.set_loop(true);
            }
            
            player.play().await?;
            
            // Wait for Ctrl+C to exit
            tokio::select! {
                _ = signal::ctrl_c() => {
                    info!("Received Ctrl+C, stopping player...");
                    player.stop().await?;
                }
                _ = player.wait_for_completion() => {
                    info!("Playback completed");
                }
            }
        },
        Commands::Interactive { path } => {
            let mut player = Mp3Player::new().await?;
            player.load_path(&path, false).await?;
            
            // Start interactive mode
            run_interactive_mode(player).await?;
        },
        Commands::Info => {
            print_audio_info();
        }
    }

    info!("MP3 Player shutting down");
    Ok(())
}

async fn run_interactive_mode(mut player: Mp3Player) -> Result<()> {
    use tokio::io::{self, AsyncBufReadExt, BufReader};
    use std::io::Write;
    
    println!("=== Interactive MP3 Player ===");
    println!("Commands: play, pause, stop, next, prev, volume <0-100>, seek <seconds>, quit");
    println!("Current playlist:");
    player.print_playlist();
    
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    
    loop {
        print!("\n> ");
        std::io::stdout().flush().ok();
        
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = line.trim();
                if let Err(e) = handle_command(&mut player, input).await {
                    error!("Command error: {}", e);
                }
            },
            Err(e) => {
                error!("Input error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

async fn handle_command(player: &mut Mp3Player, input: &str) -> Result<()> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }
    
    match parts[0].to_lowercase().as_str() {
        "play" => {
            player.play().await?;
            println!("Playing...");
        },
        "pause" => {
            player.pause().await?;
            println!("Paused");
        },
        "stop" => {
            player.stop().await?;
            println!("Stopped");
        },
        "next" => {
            player.next().await?;
            println!("Next track");
        },
        "prev" | "previous" => {
            player.previous().await?;
            println!("Previous track");
        },
        "volume" => {
            if parts.len() > 1 {
                if let Ok(vol) = parts[1].parse::<f32>() {
                    player.set_volume(vol / 100.0).await?;
                    println!("Volume set to {}%", vol);
                } else {
                    println!("Invalid volume. Use: volume <0-100>");
                }
            } else {
                let vol = player.get_volume().await * 100.0;
                println!("Current volume: {:.1}%", vol);
            }
        },
        "seek" => {
            if parts.len() > 1 {
                if let Ok(pos) = parts[1].parse::<f64>() {
                    player.seek(pos).await?;
                    println!("Seeked to {} seconds", pos);
                } else {
                    println!("Invalid position. Use: seek <seconds>");
                }
            } else {
                println!("Current position: {:.1}s", player.get_position().await);
            }
        },
        "status" => {
            player.print_status().await;
        },
        "playlist" => {
            player.print_playlist();
        },
        "quit" | "exit" => {
            player.stop().await?;
            std::process::exit(0);
        },
        "help" => {
            println!("Available commands:");
            println!("  play          - Start/resume playback");
            println!("  pause         - Pause playback");
            println!("  stop          - Stop playback");
            println!("  next          - Next track");
            println!("  prev          - Previous track");
            println!("  volume [0-100] - Set or show volume");
            println!("  seek <seconds> - Seek to position");
            println!("  status        - Show player status");
            println!("  playlist      - Show current playlist");
            println!("  quit/exit     - Exit player");
        },
        _ => {
            println!("Unknown command: {}. Type 'help' for available commands.", parts[0]);
        }
    }
    
    Ok(())
}

fn print_audio_info() {
    println!("=== Audio Format Support ===");
    println!("Supported formats:");
    println!("  - MP3 (MPEG-1/2/2.5 Layer III)");
    println!("  - AAC");
    println!("  - FLAC");
    println!("  - OGG Vorbis");
    println!("  - WAV");
    println!();
    println!("Audio backends:");
    println!("  - ALSA (Linux)");
    println!("  - PulseAudio (Linux)");
    println!("  - JACK (Linux)");
    println!("  - CoreAudio (macOS)");
    println!("  - WASAPI (Windows)");
    println!();
    println!("Features:");
    println!("  - Async/non-blocking playback");
    println!("  - Volume control");
    println!("  - Seeking");
    println!("  - Playlist management");
    println!("  - Shuffle and loop modes");
}