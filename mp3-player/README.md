## MP3 Player (Rust + tokio + ffmpeg + cpal)

Simple CLI audio player that decodes with `ffmpeg` and plays via `cpal`.

### Requirements
- ffmpeg (installed on PATH)
- ALSA dev libs (Linux): `libasound2-dev`
- Rust 1.82+

### Build
```bash
cargo build --release
```

### Run
```bash
./target/release/mp3-player <path-or-url>
```

Examples:
```bash
./target/release/mp3-player ./song.mp3
./target/release/mp3-player https://example.com/stream.mp3
```

Press Ctrl+C to stop.

### Notes
- Output device/config is chosen from system default.
- Decoding uses `ffmpeg` to `s16le` PCM, buffered and played through `cpal`.
