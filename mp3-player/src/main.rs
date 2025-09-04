use anyhow::{Context, Result};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_queue::ArrayQueue;
use std::path::PathBuf;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::signal;

/// Simple MP3 (audio) player using ffmpeg for decode and cpal for playback
#[derive(Parser, Debug)]
#[command(author, version, about = "MP3 player using ffmpeg + cpal (tokio)")]
struct Cli {
    /// Path or URL to audio file (any ffmpeg-supported input)
    input: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    // Select default output device and configuration
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("No default output device available")?;

    let supported = device
        .default_output_config()
        .context("No default output stream config")?;
    let sample_format = supported.sample_format();
    let mut config: cpal::StreamConfig = supported.config();

    let channels = config.channels as u16;
    let sample_rate = config.sample_rate.0;

    eprintln!(
        "Using output device: {} @ {} Hz, channels: {}, format: {:?}",
        device.name().unwrap_or_else(|_| "Unknown".to_string()),
        sample_rate,
        channels,
        sample_format
    );

    // Shared ring buffer of decoded samples as i16 interleaved
    // Capacity ~5 seconds of audio
    let capacity = (sample_rate as usize) * (channels as usize) * 5;
    let sample_queue: Arc<ArrayQueue<i16>> = Arc::new(ArrayQueue::new(capacity));
    let decoding_done = Arc::new(AtomicBool::new(false));

    // Spawn ffmpeg to decode to s16le PCM matching the output device
    let mut child = Command::new("ffmpeg")
        .arg("-v")
        .arg("error")
        .arg("-i")
        .arg(args.input.as_os_str())
        .arg("-f")
        .arg("s16le")
        .arg("-acodec")
        .arg("pcm_s16le")
        .arg("-ac")
        .arg(channels.to_string())
        .arg("-ar")
        .arg(sample_rate.to_string())
        .arg("pipe:1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to start ffmpeg. Is it installed?")?;

    let mut stdout = child
        .stdout
        .take()
        .context("Failed to capture ffmpeg stdout")?;

    let queue_for_reader = Arc::clone(&sample_queue);
    let done_for_reader = Arc::clone(&decoding_done);

    // Reader task: read raw bytes, convert to i16, push into ring buffer
    tokio::spawn(async move {
        let mut buf = vec![0u8; 32 * 1024];
        loop {
            match stdout.read(&mut buf).await {
                Ok(0) => {
                    // EOF
                    done_for_reader.store(true, Ordering::SeqCst);
                    break;
                }
                Ok(n) => {
                    // Convert in-place chunked by 2 bytes -> i16 little-endian
                    let mut idx = 0usize;
                    while idx + 1 < n {
                        let sample = i16::from_le_bytes([buf[idx], buf[idx + 1]]);
                        // Push, drop if full
                        let _ = queue_for_reader.push(sample);
                        idx += 2;
                    }
                }
                Err(err) => {
                    eprintln!("Error reading ffmpeg stdout: {}", err);
                    done_for_reader.store(true, Ordering::SeqCst);
                    break;
                }
            }
        }
    });

    // Build output stream according to the backend's native sample format
    let queue_for_callback = Arc::clone(&sample_queue);
    let err_fn = |err| eprintln!("Stream error: {}", err);

    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            let data_cb = move |output: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                for out in output.iter_mut() {
                    if let Some(s) = queue_for_callback.pop() {
                        // Convert i16 [-32768, 32767] to f32 [-1.0, 1.0]
                        *out = (s as f32) / 32768.0;
                    } else {
                        *out = 0.0;
                    }
                }
            };
            device
                .build_output_stream(&config, data_cb, err_fn, None)
                .context("Failed to build f32 output stream")?
        }
        cpal::SampleFormat::I16 => {
            let data_cb = move |output: &mut [i16], _info: &cpal::OutputCallbackInfo| {
                for out in output.iter_mut() {
                    if let Some(s) = queue_for_callback.pop() {
                        *out = s;
                    } else {
                        *out = 0;
                    }
                }
            };
            device
                .build_output_stream(&config, data_cb, err_fn, None)
                .context("Failed to build i16 output stream")?
        }
        cpal::SampleFormat::U16 => {
            let data_cb = move |output: &mut [u16], _info: &cpal::OutputCallbackInfo| {
                for out in output.iter_mut() {
                    if let Some(s) = queue_for_callback.pop() {
                        // Map i16 to u16 by offsetting and clamping
                        let v = (s as i32) + 32768;
                        *out = v.max(0).min(65535) as u16;
                    } else {
                        *out = 32768; // silence midpoint
                    }
                }
            };
            device
                .build_output_stream(&config, data_cb, err_fn, None)
                .context("Failed to build u16 output stream")?
        }
        other => anyhow::bail!("Unsupported sample format: {:?}", other),
    };

    stream.play().context("Failed to start audio stream")?;

    // Graceful shutdown: wait for Ctrl+C or natural end of stream
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_for_signal = should_quit.clone();
    tokio::spawn(async move {
        let _ = signal::ctrl_c().await;
        quit_for_signal.store(true, Ordering::SeqCst);
    });

    loop {
        if should_quit.load(Ordering::SeqCst) {
            break;
        }
        if decoding_done.load(Ordering::SeqCst) && sample_queue.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Ensure ffmpeg has exited
    let _ = child.wait().await;

    Ok(())
}
