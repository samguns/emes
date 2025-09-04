# MP3 Player

A high-performance MP3 player built with Rust, Tokio, and FFmpeg. This player supports various audio formats and provides both simple playback and interactive modes.

## Features

- **Multi-format Support**: MP3, FLAC, OGG Vorbis, WAV, AAC, M4A
- **Async Architecture**: Built with Tokio for non-blocking operations
- **FFmpeg Integration**: High-quality audio decoding using FFmpeg
- **Interactive Mode**: Full-featured command-line interface
- **Playlist Management**: Support for single files and directories
- **Audio Controls**: Play, pause, stop, volume control, seeking
- **Shuffle & Loop**: Randomize playback order and repeat modes
- **Cross-platform**: Supports ALSA, PulseAudio, JACK (Linux), CoreAudio (macOS), WASAPI (Windows)

## Installation

### Prerequisites

- Rust (latest stable version)
- FFmpeg development libraries
- Audio system libraries (ALSA on Linux)

#### Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install -y ffmpeg libavformat-dev libavcodec-dev libavutil-dev \
                        libswscale-dev libswresample-dev libavfilter-dev \
                        libavdevice-dev pkg-config libasound2-dev
```

#### Build from source:
```bash
git clone <repository-url>
cd mp3_player
cargo build --release
```

The binary will be available at `target/release/mp3_player`.

## Usage

### Basic Playback

Play a single MP3 file:
```bash
./target/release/mp3_player play /path/to/song.mp3
```

Play all audio files in a directory:
```bash
./target/release/mp3_player play /path/to/music/directory
```

Play with shuffle mode:
```bash
./target/release/mp3_player play --shuffle /path/to/music/directory
```

Play with loop mode:
```bash
./target/release/mp3_player play --loop /path/to/music/directory
```

### Interactive Mode

Launch interactive mode for full control:
```bash
./target/release/mp3_player interactive /path/to/music/directory
```

#### Interactive Commands:
- `play` - Start/resume playback
- `pause` - Pause playback
- `stop` - Stop playback
- `next` - Skip to next track
- `prev` - Go to previous track
- `volume <0-100>` - Set volume (0-100%)
- `volume` - Show current volume
- `seek <seconds>` - Seek to position (not yet implemented)
- `status` - Show player status
- `playlist` - Show current playlist
- `help` - Show available commands
- `quit` / `exit` - Exit player

### Information

Show supported formats and audio backends:
```bash
./target/release/mp3_player info
```

### Command Line Options

```bash
./target/release/mp3_player --help
```

Enable verbose logging:
```bash
./target/release/mp3_player --verbose play /path/to/music
```

## Architecture

The MP3 player is built with a modular architecture:

### Core Modules

1. **Decoder** (`src/decoder.rs`):
   - FFmpeg-based audio decoding
   - Supports multiple audio formats
   - Handles resampling to consistent output format
   - Async-friendly design with blocking operations in background threads

2. **Audio Output** (`src/audio.rs`):
   - Rodio-based audio playback
   - Cross-platform audio backend support
   - Volume control and playback state management
   - Buffer management for smooth playback

3. **Player** (`src/player.rs`):
   - Main orchestration logic
   - Playlist management with shuffle and loop support
   - Async state management
   - Track navigation and control

4. **CLI** (`src/cli.rs`):
   - Command-line interface structures
   - Interactive mode commands
   - Player status and response types

5. **Error Handling** (`src/error.rs`):
   - Comprehensive error types
   - FFmpeg error integration
   - User-friendly error messages

### Key Design Decisions

- **Async Architecture**: Uses Tokio for non-blocking I/O and concurrent operations
- **Thread Safety**: Extensive use of Arc<Mutex<T>> and Arc<RwLock<T>> for shared state
- **FFmpeg Integration**: Direct FFmpeg bindings for high-quality audio processing
- **Modular Design**: Clear separation of concerns between decoding, playback, and control
- **Error Handling**: Comprehensive error types with context-aware messages

## Supported Formats

- **MP3**: MPEG-1/2/2.5 Layer III
- **FLAC**: Free Lossless Audio Codec
- **OGG**: Ogg Vorbis
- **WAV**: Waveform Audio File Format
- **AAC**: Advanced Audio Coding
- **M4A**: MPEG-4 Audio

## Audio Backends

- **ALSA** (Linux)
- **PulseAudio** (Linux)
- **JACK** (Linux)
- **CoreAudio** (macOS)
- **WASAPI** (Windows)

## Performance

The player is designed for high performance:
- Async I/O prevents blocking on file operations
- Background decoding with buffering for smooth playback
- Efficient memory management with streaming audio processing
- FFmpeg's optimized codecs for fast decoding

## Limitations

- Seeking is not yet implemented (planned feature)
- No GUI interface (command-line only)
- Limited metadata display
- No playlist persistence

## Future Enhancements

- [ ] Implement seeking functionality
- [ ] Add metadata display (artist, title, album, etc.)
- [ ] Playlist save/load functionality
- [ ] Equalizer support
- [ ] GUI interface
- [ ] Network streaming support
- [ ] Plugin architecture for additional formats

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## License

This project is open source. Please check the license file for details.

## Troubleshooting

### Common Issues

1. **FFmpeg not found**: Ensure FFmpeg development libraries are installed
2. **Audio device errors**: Check that your audio system is running (PulseAudio/ALSA)
3. **Permission errors**: Ensure the binary has execute permissions
4. **File format not supported**: Check that the file is a valid audio format

### Debug Mode

Run with verbose logging for troubleshooting:
```bash
./target/release/mp3_player --verbose play /path/to/file.mp3
```

This will show detailed information about:
- File loading and format detection
- Audio decoder initialization
- Playback state changes
- Error conditions

## Examples

### Play a single song:
```bash
./target/release/mp3_player play ~/Music/song.mp3
```

### Interactive session with a music library:
```bash
./target/release/mp3_player interactive ~/Music/
```

### Shuffled playback of an album:
```bash
./target/release/mp3_player play --shuffle ~/Music/Album/
```

### Loop a playlist:
```bash
./target/release/mp3_player play --loop ~/Music/Playlist/
```