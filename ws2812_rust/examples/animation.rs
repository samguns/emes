//! WS2812 Animation Example
//! 
//! This example demonstrates:
//! - Breathing animation
//! - Chase animation (clockwise and counter-clockwise)
//! - Animation control (start/stop)

use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;
use ws2812_rust::{Color, SpiConfig, Ws2812};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("WS2812 Rust Animation Example");
    println!("This example will run various animations. Press Enter to continue between animations...");
    
    // Configure for SPI bus 1, CS 0, with 30 LEDs
    // Adjust these values for your hardware setup
    let config = SpiConfig::new(1, 0, 30);
    let mut strip = Ws2812::new(config)?;
    
    println!("Initialized strip with {} LEDs", strip.len());

    // Animation 1: Red breathing
    println!("\n=== Red Breathing Animation ===");
    println!("Starting red breathing animation at 0.5 Hz...");
    strip.start_breathe(Color::red(), 0.5)?;
    
    // Run animation and update the strip
    for _ in 0..150 { // About 5 seconds at 30fps
        strip.show()?;
        sleep(Duration::from_millis(33)); // ~30 FPS
    }
    
    strip.stop_animation();
    wait_for_enter("Press Enter to continue to next animation...");

    // Animation 2: Green breathing (faster)
    println!("\n=== Green Breathing Animation (Fast) ===");
    println!("Starting green breathing animation at 1.0 Hz...");
    strip.start_breathe(Color::green(), 1.0)?;
    
    for _ in 0..150 { // About 5 seconds
        strip.show()?;
        sleep(Duration::from_millis(33));
    }
    
    strip.stop_animation();
    wait_for_enter("Press Enter to continue to chase animation...");

    // Animation 3: Blue chase (clockwise)
    println!("\n=== Blue Chase Animation (Clockwise) ===");
    println!("Starting blue chase animation at 2.0 Hz clockwise...");
    strip.start_chase(Color::blue(), 2.0, true)?;
    
    for _ in 0..300 { // About 10 seconds
        strip.show()?;
        sleep(Duration::from_millis(33));
    }
    
    strip.stop_animation();
    wait_for_enter("Press Enter to continue to counter-clockwise chase...");

    // Animation 4: Purple chase (counter-clockwise)
    println!("\n=== Purple Chase Animation (Counter-clockwise) ===");
    let purple = Color::new(128, 0, 128);
    println!("Starting purple chase animation at 1.5 Hz counter-clockwise...");
    strip.start_chase(purple, 1.5, false)?;
    
    for _ in 0..225 { // About 7.5 seconds
        strip.show()?;
        sleep(Duration::from_millis(33));
    }
    
    strip.stop_animation();
    wait_for_enter("Press Enter to continue to color cycle...");

    // Animation 5: Manual color cycling
    println!("\n=== Manual Color Cycling ===");
    println!("Cycling through colors manually...");
    
    let colors = [
        Color::red(),
        Color::new(255, 127, 0),  // Orange
        Color::new(255, 255, 0),  // Yellow
        Color::green(),
        Color::new(0, 255, 255),  // Cyan
        Color::blue(),
        Color::new(127, 0, 255),  // Purple
        Color::new(255, 0, 127),  // Pink
    ];

    for _ in 0..3 { // Repeat the cycle 3 times
        for color in &colors {
            strip.fill(*color)?;
            strip.show()?;
            sleep(Duration::from_millis(500));
        }
    }
    
    wait_for_enter("Press Enter to continue to rainbow wave...");

    // Animation 6: Rainbow wave
    println!("\n=== Rainbow Wave Animation ===");
    println!("Creating moving rainbow wave...");
    
    for frame in 0..300 { // About 10 seconds
        for i in 0..strip.len() {
            let hue = ((i as f32 + frame as f32 * 2.0) / strip.len() as f32 * 360.0) % 360.0;
            let color = hsv_to_rgb(hue, 1.0, 0.5); // Half brightness
            strip.set_led(i, color)?;
        }
        strip.show()?;
        sleep(Duration::from_millis(33));
    }

    // Final cleanup
    println!("\n=== Cleanup ===");
    println!("Clearing all LEDs...");
    strip.clear()?;
    
    println!("Animation example complete!");
    Ok(())
}

/// Wait for user to press Enter
fn wait_for_enter(message: &str) {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
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
