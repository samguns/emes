#!/bin/bash

echo "🎵 MP3 Player Demo"
echo "=================="
echo ""
echo "This demo shows how to use the MP3 player."
echo ""

# Check if the binary exists
if [ ! -f "target/release/mp3_player" ]; then
    echo "Building the MP3 player..."
    cargo build --release
fi

echo "Available MP3 files in current directory:"
ls -1 *.mp3 2>/dev/null || echo "No MP3 files found"
echo ""

echo "Usage examples:"
echo ""
echo "1. Play all MP3 files in current directory:"
echo "   ./target/release/mp3_player"
echo ""
echo "2. Play a specific file:"
echo "   ./target/release/mp3_player test_audio.mp3"
echo ""
echo "3. Play with autoplay enabled:"
echo "   ./target/release/mp3_player --autoplay ."
echo ""
echo "4. Play with shuffle:"
echo "   ./target/release/mp3_player --shuffle --autoplay ."
echo ""
echo "Keyboard controls:"
echo "  Space       - Play/Pause"
echo "  ← / →       - Previous/Next track"
echo "  ↑ / ↓       - Navigate playlist"
echo "  Enter       - Play selected track"
echo "  + / -       - Volume up/down"
echo "  s           - Stop"
echo "  r           - Toggle repeat"
echo "  h           - Show help"
echo "  q           - Quit"
echo ""
echo "Starting the player with autoplay..."
echo ""

./target/release/mp3_player --autoplay .