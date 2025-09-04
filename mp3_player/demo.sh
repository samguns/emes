#!/bin/bash

echo "=== MP3 Player Demo ==="
echo ""

echo "1. Project Structure:"
echo "===================="
find . -name "*.rs" -o -name "*.toml" -o -name "*.md" | grep -v target | sort
echo ""

echo "2. Help Information:"
echo "==================="
./target/release/mp3_player --help
echo ""

echo "3. Supported Formats and Features:"
echo "================================="
./target/release/mp3_player info
echo ""

echo "4. Build Information:"
echo "===================="
echo "Built with:"
echo "- Rust (with Tokio async runtime)"
echo "- FFmpeg for audio decoding"
echo "- Rodio for cross-platform audio output"
echo "- Clap for CLI interface"
echo ""

echo "5. Key Features Implemented:"
echo "==========================="
echo "✓ Multi-format audio support (MP3, FLAC, OGG, WAV, AAC)"
echo "✓ Async/non-blocking architecture with Tokio"
echo "✓ FFmpeg integration for high-quality decoding"
echo "✓ Interactive command-line interface"
echo "✓ Playlist management (files and directories)"
echo "✓ Audio controls (play, pause, stop, volume)"
echo "✓ Shuffle and loop modes"
echo "✓ Cross-platform audio backend support"
echo "✓ Comprehensive error handling"
echo "✓ Modular, extensible architecture"
echo ""

echo "6. Code Statistics:"
echo "=================="
echo "Lines of code by module:"
wc -l src/*.rs | sort -n
echo ""

echo "7. Dependencies:"
echo "==============="
grep -A 20 "\[dependencies\]" Cargo.toml
echo ""

echo "Demo completed! The MP3 player is fully functional."
echo "Note: Audio playback requires proper audio device setup."