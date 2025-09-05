# WS2812 Rust Driver

A high-performance Rust implementation of WS2812 (NeoPixel) LED driver using SPI interface. This library provides both synchronous and asynchronous APIs for controlling WS2812 LED strips with built-in animation patterns.

## Features

- 🚀 **SPI-based communication** for reliable timing without CPU-intensive bit-banging
- 🎨 **RGB and GRB color format** support with easy color manipulation
- ✨ **Built-in animations**: breathe, chase, and custom patterns
- 🧵 **Thread-safe** animation control with automatic cleanup
- ⚡ **High performance** using efficient bit manipulation and buffering
- 🔧 **Configurable** LED count, SPI timing, and animation parameters

## Hardware Requirements

- Linux system with SPI support
- WS2812/NeoPixel LED strip connected to SPI MOSI pin
- SPI device (e.g., `/dev/spidev1.0`)

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
ws2812_rust = "0.1"
```

### Basic Usage

```rust
use ws2812_rust::{Ws2812, Color, SpiConfig};

// Configure for SPI bus 1, CS 0, with 30 LEDs
let config = SpiConfig::new(1, 0, 30);
let mut strip = Ws2812::new(config)?;

// Set all LEDs to red
strip.fill(Color::red())?;
strip.show()?;

// Set individual LED
strip.set_led(0, Color::blue())?;
strip.show()?;

// Create custom colors
let purple = Color::new(128, 0, 128);
strip.set_led(5, purple)?;
strip.show()?;
```

### Animations

```rust
// Breathing animation (color pulses in and out)
strip.start_breathe(Color::blue(), 0.5)?; // 0.5 Hz frequency

// Run animation loop
loop {
    strip.show()?;
    std::thread::sleep(std::time::Duration::from_millis(33)); // ~30 FPS
}

// Chase animation (single LED moves around)
strip.start_chase(Color::red(), 2.0, true)?; // 2 Hz, clockwise

// Stop any running animation
strip.stop_animation();
```

### Advanced Usage

```rust
// Multiple LED control
let colors = vec![
    Color::red(),
    Color::green(),
    Color::blue(),
];
strip.set_leds(&colors)?;
strip.show()?;

// Color scaling (brightness control)
let dim_white = Color::white().scale(0.1); // 10% brightness
strip.fill(dim_white)?;
strip.show()?;

// HSV to RGB conversion (example utility)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    // Implementation for rainbow effects
    // ... (see examples for full implementation)
}
```

## API Reference

### Core Types

#### `Ws2812`
Main driver structure for controlling WS2812 LEDs.

**Methods:**
- `new(config: SpiConfig) -> Result<Self, Ws2812Error>`
- `set_led(index: usize, color: Color) -> Result<(), Ws2812Error>`
- `get_led(index: usize) -> Result<Color, Ws2812Error>`
- `fill(color: Color) -> Result<(), Ws2812Error>`
- `set_leds(colors: &[Color]) -> Result<(), Ws2812Error>`
- `clear() -> Result<(), Ws2812Error>`
- `show() -> Result<(), Ws2812Error>`
- `len() -> usize`

**Animation Methods:**
- `start_breathe(color: Color, hz: f32) -> Result<(), Ws2812Error>`
- `start_chase(color: Color, hz: f32, clockwise: bool) -> Result<(), Ws2812Error>`
- `stop_animation()`
- `is_animating() -> bool`

#### `Color`
RGB color representation with utility methods.

**Methods:**
- `new(r: u8, g: u8, b: u8) -> Self`
- `black()`, `white()`, `red()`, `green()`, `blue()` - Predefined colors
- `scale(factor: f32) -> Self` - Brightness scaling
- `to_grb() -> [u8; 3]` - Convert to GRB format

#### `SpiConfig`
Configuration for SPI interface.

**Methods:**
- `new(bus: u8, cs: u8, num_leds: usize) -> Self`
- `device_path() -> String`

### Error Handling

```rust
use ws2812_rust::Ws2812Error;

match strip.set_led(100, Color::red()) {
    Ok(()) => println!("LED set successfully"),
    Err(Ws2812Error::ConfigError(msg)) => println!("Invalid LED index: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Examples

### Run Basic Example
```bash
cargo run --example basic
```

### Run Animation Example
```bash
cargo run --example animation
```

## Hardware Setup

### Raspberry Pi SPI Connection

1. Enable SPI in `/boot/config.txt`:
   ```
   dtparam=spi=on
   ```

2. Connect WS2812 strip:
   - VCC → 5V (external power recommended for many LEDs)
   - GND → Ground
   - DIN → GPIO 10 (MOSI)

3. Verify SPI device exists:
   ```bash
   ls /dev/spidev*
   ```

### Timing Configuration

The library uses these WS2812 timing constants:
- **LED_ZERO**: `0b1100_0000` (312.5ns high, 625ns low)
- **LED_ONE**: `0b1111_1100` (625ns high, 312.5ns low)  
- **SPI Speed**: 6.5 MHz
- **Reset**: 42 bytes of zeros (>50μs low)

## Performance

- **SPI-based**: No CPU-intensive bit-banging, reliable timing
- **Buffered**: Efficient bulk updates with single SPI transaction
- **Thread-safe**: Safe concurrent access to LED buffer
- **Low latency**: Minimal overhead for real-time applications

## Comparison with Python Version

This Rust implementation provides equivalent functionality to the Python `ws2812.py` with these improvements:

- ✅ **Memory safety** - No buffer overflows or memory leaks
- ✅ **Better performance** - Zero-copy operations and efficient threading
- ✅ **Type safety** - Compile-time error checking
- ✅ **Resource management** - Automatic cleanup on drop
- ✅ **Concurrency** - Safe multi-threaded access

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

Licensed under the Apache License, Version 2.0.
