# tokio-mp3-player

A minimal Tokio-based CLI MP3 player that shells out to `ffmpeg` to decode and plays audio via ALSA on Linux.

## Prerequisites

- ffmpeg
- ALSA (libasound2) and a working output device
- Rust toolchain (cargo, rustc)

## Install prerequisites (Debian/Ubuntu)

```bash
sudo apt-get update && sudo apt-get install -y ffmpeg libasound2 libasound2-dev build-essential pkg-config
```

## Build

```bash
cargo build --release
```

## Run

```bash
./target/release/tokio-mp3-player <file.mp3> --device default --show-progress
```

Controls (future work):
- p: toggle pause
- s: stop
- q: quit
- +: volume up
- -: volume down

This minimal version demonstrates process management and progress printing. Advanced controls can be implemented via `ffmpeg` filters or by controlling the ALSA device directly via a Rust audio library.

