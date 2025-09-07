//! # WS2812 Rust Driver
//!
//! A Rust implementation of WS2812 (NeoPixel) LED driver using SPI interface.
//! This library provides both synchronous and asynchronous APIs for controlling
//! WS2812 LED strips with built-in animation patterns.
//!
//! ## Features
//!
//! - SPI-based communication for reliable timing
//! - RGB and GRB color format support
//! - Built-in animation patterns (breathe, chase)
//! - Thread-safe animation control
//! - Configurable LED count and timing
//!
//! ## Example
//!
//! ```rust,no_run
//! use ws2812_rust::{Ws2812, Color, SpiConfig};
//!
//! let config = SpiConfig::new(0, 0, 30); // bus=0, cs=0, 30 LEDs
//! let mut strip = Ws2812::new(config)?;
//!
//! // Set all LEDs to red
//! let red = Color::new(255, 0, 0);
//! strip.fill(red)?;
//! strip.show()?;
//! ```

use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur when using the WS2812 driver
#[derive(Error, Debug)]
pub enum Ws2812Error {
    #[error("SPI device not found: {0}")]
    SpiDeviceNotFound(String),
    #[error("SPI communication error: {0}")]
    SpiError(#[from] std::io::Error),
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    #[error("Animation error: {0}")]
    AnimationError(String),
}

/// RGB Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new RGB color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create a black (off) color
    pub fn black() -> Self {
        Self::new(0, 0, 0)
    }

    /// Create a white color
    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }

    /// Create a red color
    pub fn red() -> Self {
        Self::new(255, 0, 0)
    }

    /// Create a green color
    pub fn green() -> Self {
        Self::new(0, 255, 0)
    }

    /// Create a blue color
    pub fn blue() -> Self {
        Self::new(0, 0, 255)
    }

    /// Scale brightness (0.0 to 1.0)
    pub fn scale(&self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 * factor) as u8,
            g: (self.g as f32 * factor) as u8,
            b: (self.b as f32 * factor) as u8,
        }
    }

    /// Convert RGB to GRB format (WS2812 order)
    pub fn to_grb(&self) -> [u8; 3] {
        [self.g, self.r, self.b]
    }
}

/// SPI Configuration for WS2812
#[derive(Debug, Clone)]
pub struct SpiConfig {
    pub bus: u8,
    pub cs: u8,
    pub num_leds: usize,
    pub max_speed_hz: u32,
}

impl SpiConfig {
    /// Create a new SPI configuration
    pub fn new(bus: u8, cs: u8, num_leds: usize) -> Self {
        Self {
            bus,
            cs,
            num_leds,
            max_speed_hz: 6_500_000, // 6.5MHz as in Python version
        }
    }

    /// Get the spidev device path
    pub fn device_path(&self) -> String {
        format!("/dev/spidev{}.{}", self.bus, self.cs)
    }
}

/// Animation control structure
#[derive(Debug)]
struct AnimationControl {
    running: Arc<RwLock<bool>>,
    handle: Option<JoinHandle<()>>,
}

/// Main WS2812 driver structure
pub struct Ws2812 {
    spi: Spidev,
    config: SpiConfig,
    led_buffer: Arc<Mutex<Vec<Color>>>,
    tx_buffer: Vec<u8>,
    animation: Option<AnimationControl>,
}

impl Ws2812 {
    // WS2812 timing constants (as in Python version)
    const LED_ZERO: u8 = 0b1100_0000; // WS2812 "0" bit pattern
    const LED_ONE: u8 = 0b1111_1100; // WS2812 "1" bit pattern
    const RESET_BYTES_COUNT: usize = 42; // Reset signal length

    /// Create a new WS2812 driver instance
    pub fn new(config: SpiConfig) -> Result<Self, Ws2812Error> {
        let device_path = config.device_path();

        // Check if SPI device exists
        if !Path::new(&device_path).exists() {
            return Err(Ws2812Error::SpiDeviceNotFound(device_path));
        }

        // Open and configure SPI device
        let mut spi = Spidev::open(&device_path)?;
        let options = SpidevOptions::new()
            .max_speed_hz(config.max_speed_hz)
            .mode(SpiModeFlags::SPI_MODE_0)
            .lsb_first(false)
            .bits_per_word(8)
            .build();
        spi.configure(&options)?;

        // Initialize buffers
        let led_buffer = Arc::new(Mutex::new(vec![Color::black(); config.num_leds]));
        let tx_buffer = vec![0u8; Self::RESET_BYTES_COUNT + config.num_leds * 24];

        Ok(Self {
            spi,
            config,
            led_buffer,
            tx_buffer,
            animation: None,
        })
    }

    /// Set a single LED color
    pub fn set_led(&mut self, index: usize, color: Color) -> Result<(), Ws2812Error> {
        if index >= self.config.num_leds {
            return Err(Ws2812Error::ConfigError(format!(
                "LED index {} out of range (0-{})",
                index,
                self.config.num_leds - 1
            )));
        }

        let mut buffer = self.led_buffer.lock().unwrap();
        buffer[index] = color;
        Ok(())
    }

    /// Get a LED color
    pub fn get_led(&self, index: usize) -> Result<Color, Ws2812Error> {
        if index >= self.config.num_leds {
            return Err(Ws2812Error::ConfigError(format!(
                "LED index {} out of range (0-{})",
                index,
                self.config.num_leds - 1
            )));
        }

        let buffer = self.led_buffer.lock().unwrap();
        Ok(buffer[index])
    }

    /// Fill all LEDs with the same color
    pub fn fill(&mut self, color: Color) -> Result<(), Ws2812Error> {
        let mut buffer = self.led_buffer.lock().unwrap();
        buffer.fill(color);
        Ok(())
    }

    /// Set multiple LED colors from a slice
    pub fn set_leds(&mut self, colors: &[Color]) -> Result<(), Ws2812Error> {
        let mut buffer = self.led_buffer.lock().unwrap();
        let len = colors.len().min(self.config.num_leds);
        buffer[..len].copy_from_slice(&colors[..len]);

        // Fill remaining LEDs with black if colors slice is shorter
        if colors.len() < self.config.num_leds {
            buffer[colors.len()..].fill(Color::black());
        }

        Ok(())
    }

    /// Clear all LEDs (turn them off)
    pub fn clear(&mut self) -> Result<(), Ws2812Error> {
        self.fill(Color::black())?;
        self.show()
    }

    /// Convert 8-bit value to WS2812 SPI bits
    fn byte_to_spi_bits(&self, byte: u8) -> [u8; 8] {
        let mut bits = [0u8; 8];
        for i in 0..8 {
            bits[i] = if (byte >> (7 - i)) & 1 == 1 {
                Self::LED_ONE
            } else {
                Self::LED_ZERO
            };
        }
        bits
    }

    /// Update the LED strip with current buffer contents
    pub fn show(&mut self) -> Result<(), Ws2812Error> {
        let buffer = self.led_buffer.lock().unwrap();

        // Clear tx buffer with reset bytes
        self.tx_buffer.fill(0);

        // Convert LED colors to SPI bits
        let mut bit_index = Self::RESET_BYTES_COUNT;
        for color in buffer.iter() {
            let grb = color.to_grb();

            // Convert each color byte to SPI timing bits
            for &byte in &grb {
                let spi_bits = self.byte_to_spi_bits(byte);
                self.tx_buffer[bit_index..bit_index + 8].copy_from_slice(&spi_bits);
                bit_index += 8;
            }
        }

        // Send data via SPI
        let mut transfer = SpidevTransfer::write(&self.tx_buffer);
        self.spi.transfer(&mut transfer)?;

        Ok(())
    }

    /// Get the number of LEDs
    pub fn len(&self) -> usize {
        self.config.num_leds
    }

    /// Check if the strip is empty (has no LEDs)
    pub fn is_empty(&self) -> bool {
        self.config.num_leds == 0
    }

    /// Start a breathing animation with the specified color and frequency
    pub fn start_breathe(&mut self, color: Color, hz: f32) -> Result<(), Ws2812Error> {
        self.stop_animation();

        let fps = 30.0;
        let frames = (fps / hz) as usize;
        if frames < 6 {
            return Err(Ws2812Error::AnimationError(
                "Frequency too high, minimum 6 frames required".to_string(),
            ));
        }

        let running = Arc::new(RwLock::new(true));
        let running_clone = running.clone();
        let led_buffer_clone = self.led_buffer.clone();
        let num_leds = self.config.num_leds;

        let handle = thread::spawn(move || {
            let mut frame = 0;
            let frame_duration = Duration::from_secs_f32(1.0 / fps);

            while *running_clone.read().unwrap() {
                let start_time = Instant::now();

                // Calculate breathing intensity using cosine wave
                let phase = (frame as f32) * 2.0 * std::f32::consts::PI / frames as f32;
                let intensity = (phase.cos() + 1.0) * 0.5; // 0.0 to 1.0

                let scaled_color = color.scale(intensity);

                // Update all LEDs
                {
                    let mut buffer = led_buffer_clone.lock().unwrap();
                    buffer.fill(scaled_color);
                }

                frame = (frame + 1) % frames;

                // Sleep for remaining frame time
                let elapsed = start_time.elapsed();
                if elapsed < frame_duration {
                    thread::sleep(frame_duration - elapsed);
                }
            }
        });

        self.animation = Some(AnimationControl {
            running,
            handle: Some(handle),
        });

        Ok(())
    }

    /// Start a chase animation with the specified color and frequency
    pub fn start_chase(
        &mut self,
        color: Color,
        hz: f32,
        clockwise: bool,
    ) -> Result<(), Ws2812Error> {
        self.stop_animation();

        let fps = 30.0;
        let frames = (fps / hz) as usize;
        let frames_per_led = (frames as f32 / self.config.num_leds as f32).ceil() as usize;
        let total_frames = frames_per_led * self.config.num_leds;

        let running = Arc::new(RwLock::new(true));
        let running_clone = running.clone();
        let led_buffer_clone = self.led_buffer.clone();
        let num_leds = self.config.num_leds;

        let handle = thread::spawn(move || {
            let mut frame = 0;
            let frame_duration = Duration::from_secs_f32(1.0 / fps);

            while *running_clone.read().unwrap() {
                let start_time = Instant::now();

                // Clear all LEDs
                let mut colors = vec![Color::black(); num_leds];

                // Calculate which LED should be lit
                let led_index = frame / frames_per_led;
                let actual_index = if clockwise {
                    (num_leds - 1) - led_index
                } else {
                    led_index
                };

                if actual_index < num_leds {
                    colors[actual_index] = color;
                }

                // Update LED buffer
                {
                    let mut buffer = led_buffer_clone.lock().unwrap();
                    *buffer = colors;
                }

                frame = (frame + 1) % total_frames;

                // Sleep for remaining frame time
                let elapsed = start_time.elapsed();
                if elapsed < frame_duration {
                    thread::sleep(frame_duration - elapsed);
                }
            }
        });

        self.animation = Some(AnimationControl {
            running,
            handle: Some(handle),
        });

        Ok(())
    }

    /// Stop any running animation
    pub fn stop_animation(&mut self) {
        if let Some(mut anim) = self.animation.take() {
            *anim.running.write().unwrap() = false;
            if let Some(handle) = anim.handle.take() {
                let _ = handle.join();
            }
        }
    }

    /// Check if an animation is currently running
    pub fn is_animating(&self) -> bool {
        self.animation
            .as_ref()
            .map_or(false, |anim| *anim.running.read().unwrap())
    }
}

impl Drop for Ws2812 {
    fn drop(&mut self) {
        // Stop animation and clear LEDs when dropping
        self.stop_animation();
        let _ = self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let red = Color::red();
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);
    }

    #[test]
    fn test_color_scaling() {
        let white = Color::white();
        let half = white.scale(0.5);
        assert_eq!(half.r, 127);
        assert_eq!(half.g, 127);
        assert_eq!(half.b, 127);
    }

    #[test]
    fn test_color_to_grb() {
        let color = Color::new(255, 128, 64);
        let grb = color.to_grb();
        assert_eq!(grb, [128, 255, 64]); // G, R, B
    }

    #[test]
    fn test_spi_config() {
        let config = SpiConfig::new(1, 0, 30);
        assert_eq!(config.device_path(), "/dev/spidev1.0");
    }
}
