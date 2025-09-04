use std::process::Stdio;

use anyhow::{Context, Result};
use clap::Parser;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "tokio-mp3-player", version, about = "Tokio-based MP3 player using ffmpeg -> ALSA")] 
struct Cli {
    /// Input MP3 file path
    input: String,

    /// ALSA device name (e.g., default)
    #[arg(short = 'd', long = "device", default_value = "default")]
    device: String,

    /// Start position, e.g. 00:01:23 or seconds (float)
    #[arg(short = 's', long = "start", default_value = "0")]
    start: String,

    /// Show ffmpeg progress
    #[arg(long = "show-progress")]
    show_progress: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let mut args: Vec<String> = vec!["-re".into()];
    if cli.start != "0" {
        args.push("-ss".into());
        args.push(cli.start.clone());
    }
    args.push("-i".into());
    args.push(cli.input.clone());
    args.extend(vec![
        "-vn".into(),
        "-f".into(), "alsa".into(),
        cli.device.clone(),
    ]);

    if cli.show_progress {
        args.splice(0..0, ["-hide_banner".into(), "-loglevel".into(), "info".into()]);
    } else {
        args.splice(0..0, ["-hide_banner".into(), "-loglevel".into(), "error".into()]);
    }

    info!(?args, "Launching ffmpeg");

    let mut cmd = Command::new("ffmpeg");
    cmd.args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().context("failed to spawn ffmpeg")?;

    if cli.show_progress {
        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    if line.contains("time=") || line.contains("speed=") || line.contains("size=") {
                        eprintln!("{line}");
                    }
                }
            });
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        error!(?status, "ffmpeg exited with non-zero status");
        anyhow::bail!("ffmpeg failed: {status}");
    }
    Ok(())
}

use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "tokio-mp3-player", version, about = "Tokio-based MP3 player using ffmpeg -> ALSA", long_about = None)]
struct Cli {
    /// Input MP3 file path
    input: String,

    /// ALSA device name (e.g., default)
    #[arg(short = d, long = "device", default_value = "default")]
    device: String,

    /// Start position, e.g. 00:01:23 or seconds (float)
    #[arg(short = s, long = "start", default_value = "0")]
    start: String,

    /// Show ffmpeg progress
    #[arg(long = "show-progress")]
    show_progress: bool,
}

#[derive(Subcommand, Debug)]
enum CommandArg {
    /// Play (default)
    Play,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let (ctrl_tx, mut ctrl_rx) = mpsc::unbounded_channel::<PlayerControl>();

    let mut player = Player::new(cli.input.clone(), cli.device.clone());

    if cli.show_progress {
        player.enable_progress();
    }

    if cli.start != "0" {
        player.set_start(cli.start.clone());
    }

    let mut handle = tokio::spawn(async move {
        if let Err(e) = player.play().await {
            error!(error = ?e, "Playback error");
        }
    });

    // Simple stdin control loop: p= pause/resume, s=stop, q=quit, +=vol up, -=vol down
    tokio::spawn(async move {
        use tokio::io::{stdin, AsyncReadExt};
        let mut input = stdin();
        let mut buf = [0u8; 1];
        loop {
            if input.read_exact(&mut buf).await.is_err() {
                break;
            }
            match buf[0] as char {
                p => { let _ = ctrl_tx.send(PlayerControl::TogglePause);} 
                s => { let _ = ctrl_tx.send(PlayerControl::Stop);} 
                q => { let _ = ctrl_tx.send(PlayerControl::Quit);} 
                + => { let _ = ctrl_tx.send(PlayerControl::VolumeUp);} 
                - => { let _ = ctrl_tx.send(PlayerControl::VolumeDown);} 
                _ => {}
            }
        }
    });

    // Consume control messages (no-op in this simplified version)
    tokio::spawn(async move {
        while let Some(_msg) = ctrl_rx.recv().await {
            // In a full implementation, we would communicate with ffmpeg via filters or restart process
        }
    });

    handle.await?;

    Ok(())
}

#[derive(Debug)]
enum PlayerControl {
    TogglePause,
    Stop,
    Quit,
    VolumeUp,
    VolumeDown,
}

struct Player {
    input_path: String,
    alsa_device: String,
    start: Option<String>,
    show_progress: bool,
}

impl Player {
    fn new(input_path: String, alsa_device: String) -> Self {
        Self { input_path, alsa_device, start: None, show_progress: false }
    }

    fn set_start(&mut self, start: String) { self.start = Some(start); }
    fn enable_progress(&mut self) { self.show_progress = true; }

    async fn play(&mut self) -> Result<()> {
        let mut args: Vec<String> = vec![
            "-re".into(), // read at native rate
        ];

        if let Some(start) = &self.start {
            args.push("-ss".into());
            args.push(start.clone());
        }

        args.push("-i".into());
        args.push(self.input_path.clone());

        // Use ffmpeg to decode to PCM s16le stereo 44.1kHz and send to ALSA
        args.extend(vec![
            "-vn".into(),
            "-f".into(), "alsa".into(),
            self.alsa_device.clone(),
        ]);

        if self.show_progress {
            // progress on stderr by default; use -stats_period
            args.splice(0..0, ["-hide_banner".into(), "-loglevel".into(), "info".into()]);
        } else {
            args.splice(0..0, ["-hide_banner".into(), "-loglevel".into(), "error".into()]);
        }

        info!(?args, "Launching ffmpeg");

        let mut cmd = Command::new("ffmpeg");
        cmd.args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("failed to spawn ffmpeg")?;

        if self.show_progress {
            if let Some(stderr) = child.stderr.take() {
                let mut reader = BufReader::new(stderr).lines();
                tokio::spawn(async move {
                    while let Ok(Some(line)) = reader.next_line().await {
                        if line.contains("time=") || line.contains("speed=") || line.contains("size=") {
                            eprintln!("{line}");
                        }
                    }
                });
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("ffmpeg exited with status: {:?}", status);
        }
        Ok(())
    }
}
