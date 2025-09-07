//! Basic WS2812 usage example
//! 
//! This example shows how to:
//! - Initialize the WS2812 driver
//! - Set individual LED colors
//! - Fill all LEDs with one color
//! - Clear the strip

use std::thread::sleep;
use std::time::Duration;
use ws2812_rust::{Color, SpiConfig, Ws2812};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("WS2812 Rust Basic Example");
    
    // Configure for SPI bus 1, CS 0, with 30 LEDs
    // Adjust these values for your hardware setup
    let config = SpiConfig::new(1, 0, 30);
    let mut strip = Ws2812::new(config)?;
    
    println!("Initialized strip with {} LEDs", strip.len());

    // Example 1: Fill all LEDs with red
    println!("Setting all LEDs to red...");
    strip.fill(Color::red())?;
    strip.show()?;
    sleep(Duration::from_secs(2));

    // Example 2: Fill all LEDs with green
    println!("Setting all LEDs to green...");
    strip.fill(Color::green())?;
    strip.show()?;
    sleep(Duration::from_secs(2));

    // Example 3: Fill all LEDs with blue
    println!("Setting all LEDs to blue...");
    strip.fill(Color::blue())?;
    strip.show()?;
    sleep(Duration::from_secs(2));

    // Example 4: Set individual LEDs to create a pattern
    println!("Creating rainbow pattern...");
    for i in 0..strip.len() {
        let hue = (i as f32 / strip.len() as f32) * 360.0;
        let color = hsv_to_rgb(hue, 1.0, 1.0);
        strip.set_led(i, color)?;
    }
    strip.show()?;
    sleep(Duration::from_secs(3));

    // Example 5: Dim the rainbow
    println!("Dimming rainbow...");
    for i in 0..strip.len() {
        let current_color = strip.get_led(i)?;
        let dimmed = current_color.scale(0.3);
        strip.set_led(i, dimmed)?;
    }
    strip.show()?;
    sleep(Duration::from_secs(2));

    // Example 6: Clear all LEDs
    println!("Clearing all LEDs...");
    strip.clear()?;
    
    println!("Example complete!");
    Ok(())
}

/// Convert HSV to RGB color
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r_prime + m) * 255.0) as u8;
    let g = ((g_prime + m) * 255.0) as u8;
    let b = ((b_prime + m) * 255.0) as u8;

    Color::new(r, g, b)
}
