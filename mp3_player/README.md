# 🎵 MP3 Player

A modern, terminal-based MP3 player written in Rust with async support using Tokio and audio decoding via Symphonia/Rodio.

## Features

- 🎵 **Multi-format Support**: Play MP3, FLAC, OGG, WAV, M4A, and AAC files
- 🎨 **Beautiful TUI**: Interactive terminal interface with real-time updates
- 📁 **Playlist Management**: Load entire directories, navigate tracks
- 🎮 **Playback Controls**: Play, pause, stop, next, previous
- 🔊 **Volume Control**: Adjust volume on the fly
- 🔁 **Repeat Mode**: Loop through your playlist
- 🎲 **Shuffle**: Randomize playlist order
- ⚡ **Async Architecture**: Built with Tokio for responsive performance
- 🦀 **Pure Rust**: No external FFmpeg dependency required

## Prerequisites

- Rust 1.70 or higher
- Linux: ALSA development libraries (`libasound2-dev` on Ubuntu/Debian)
- macOS: Core Audio (included with macOS)
- Windows: WASAPI (included with Windows)

## Installation

### From Source

1. Clone the repository:
```bash
git clone https://github.com/yourusername/mp3_player.git
cd mp3_player
```

2. Build the project:
```bash
cargo build --release
```

3. The binary will be available at `target/release/mp3_player`

### Install System Dependencies

#### Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install libasound2-dev
```

#### Fedora:
```bash
sudo dnf install alsa-lib-devel
```

#### macOS:
No additional dependencies needed.

#### Windows:
No additional dependencies needed.

## Usage

### Basic Usage

```bash
# Play all audio files in current directory
./mp3_player

# Play a specific file
./mp3_player song.mp3

# Play all files in a directory
./mp3_player /path/to/music/

# Start with autoplay
./mp3_player --autoplay /path/to/music/

# Shuffle playlist
./mp3_player --shuffle /path/to/music/
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `Space` | Play/Pause |
| `Enter` | Play selected track |
| `←` / `→` | Previous/Next track |
| `↑` / `↓` | Navigate playlist |
| `+` / `-` | Volume up/down |
| `s` | Stop playback |
| `r` | Toggle repeat mode |
| `h` / `?` | Show help |
| `q` / `Ctrl+C` | Quit |

## Command Line Options

```
mp3_player [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to MP3 file or directory containing audio files

Options:
  -a, --autoplay  Start playing immediately
  -s, --shuffle   Shuffle playlist
  -h, --help      Print help
  -V, --version   Print version
```

## Architecture

The player is built with a modular architecture:

- **Player Module**: Handles audio decoding and playback using Rodio/Symphonia
- **Playlist Module**: Manages track lists and navigation
- **UI Module**: Renders the terminal interface using Ratatui
- **Main**: Orchestrates components with Tokio async runtime

### Audio Pipeline

1. **File Loading**: Walks directories to find audio files
2. **Decoding**: Uses Symphonia for format detection and decoding
3. **Playback**: Rodio handles audio output to system audio
4. **Control**: Async event loop processes user input

## Supported Formats

- MP3 (`.mp3`)
- FLAC (`.flac`)
- OGG Vorbis (`.ogg`)
- WAV (`.wav`)
- AAC/M4A (`.aac`, `.m4a`)

## Building for Different Platforms

### Linux
```bash
cargo build --release
```

### macOS
```bash
cargo build --release
```

### Windows
```bash
cargo build --release
```

### Cross-compilation
For cross-compilation, you may need to install additional targets:
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## Troubleshooting

### No Sound Output
- Check system volume settings
- Ensure audio device is properly connected
- On Linux, check ALSA configuration: `aplay -l`

### File Not Playing
- Verify file format is supported
- Check file permissions
- Ensure file is not corrupted

### Performance Issues
- Built in release mode for optimal performance
- Large playlists (1000+ files) may take time to load initially

## Development

### Running Tests
```bash
cargo test
```

### Running with Debug Output
```bash
RUST_LOG=debug cargo run
```

### Code Structure
```
src/
├── main.rs      # Application entry point and event loop
├── player.rs    # Audio playback engine
├── playlist.rs  # Playlist management
└── ui.rs        # Terminal user interface
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Rodio](https://github.com/RustAudio/rodio) - Audio playback library
- [Symphonia](https://github.com/pdeljanov/Symphonia) - Pure Rust audio decoding
- [Ratatui](https://github.com/ratatui/ratatui) - Terminal UI framework
- [Tokio](https://tokio.rs/) - Async runtime for Rust

## Future Enhancements

- [ ] Seeking within tracks
- [ ] Equalizer support
- [ ] Visualization (spectrum analyzer)
- [ ] Network streaming support
- [ ] Playlist save/load
- [ ] ID3 tag reading for metadata
- [ ] Gapless playback
- [ ] MPRIS support (Linux)