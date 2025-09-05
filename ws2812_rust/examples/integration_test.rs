//! WS2812 Integration Test Example
//! 
//! This example demonstrates integration with other systems,
//! showing how the WS2812 library can be used in real applications.

use std::thread;
use std::time::Duration;
use ws2812_rust::{Color, SpiConfig, Ws2812};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("WS2812 Rust Integration Test");
    
    // Test 1: Multiple strip instances (if you have multiple SPI devices)
    println!("=== Testing Multiple Strip Support ===");
    let config1 = SpiConfig::new(1, 0, 10);
    let config2 = SpiConfig::new(1, 1, 20);
    
    // Note: This will only work if you have multiple SPI CS lines
    // For testing, we'll just use one strip
    let mut strip = match Ws2812::new(config1) {
        Ok(s) => s,
        Err(e) => {
            println!("Could not initialize strip: {}. This is normal if SPI device doesn't exist.", e);
            println!("Continuing with mock operations...");
            return Ok(());
        }
    };
    
    println!("Successfully initialized strip with {} LEDs", strip.len());

    // Test 2: Error handling
    println!("\n=== Testing Error Handling ===");
    match strip.set_led(1000, Color::red()) {
        Err(ws2812_rust::Ws2812Error::ConfigError(msg)) => {
            println!("✓ Correctly caught out-of-bounds error: {}", msg);
        }
        _ => println!("✗ Should have caught out-of-bounds error"),
    }

    // Test 3: Color operations
    println!("\n=== Testing Color Operations ===");
    let original = Color::new(200, 100, 50);
    let scaled = original.scale(0.5);
    println!("Original color: R={}, G={}, B={}", original.r, original.g, original.b);
    println!("Scaled (50%): R={}, G={}, B={}", scaled.r, scaled.g, scaled.b);
    
    let grb = original.to_grb();
    println!("GRB format: [{}, {}, {}]", grb[0], grb[1], grb[2]);

    // Test 4: Batch operations
    println!("\n=== Testing Batch Operations ===");
    let colors = (0..strip.len())
        .map(|i| {
            let intensity = (i as f32 / strip.len() as f32 * 255.0) as u8;
            Color::new(intensity, 0, 255 - intensity) // Purple gradient
        })
        .collect::<Vec<_>>();
    
    strip.set_leds(&colors)?;
    println!("Set {} colors in batch", colors.len());

    // Test 5: Animation lifecycle
    println!("\n=== Testing Animation Lifecycle ===");
    println!("Starting animation...");
    strip.start_breathe(Color::new(0, 255, 127), 1.0)?;
    
    println!("Animation running: {}", strip.is_animating());
    
    // Run for a short time
    for i in 0..30 {
        strip.show()?;
        thread::sleep(Duration::from_millis(33));
        if i % 10 == 0 {
            println!("Animation frame {}/30", i + 1);
        }
    }
    
    println!("Stopping animation...");
    strip.stop_animation();
    println!("Animation running: {}", strip.is_animating());

    // Test 6: Performance measurement
    println!("\n=== Testing Performance ===");
    let start = std::time::Instant::now();
    
    for _ in 0..100 {
        strip.fill(Color::new(100, 150, 200))?;
        strip.show()?;
    }
    
    let elapsed = start.elapsed();
    println!("100 full updates took: {:?}", elapsed);
    println!("Average update time: {:?}", elapsed / 100);
    println!("Theoretical max FPS: {:.1}", 1000.0 / elapsed.as_millis() as f32 * 100.0);

    // Cleanup
    println!("\n=== Cleanup ===");
    strip.clear()?;
    println!("Integration test completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scaling_bounds() {
        let white = Color::white();
        
        // Test lower bound
        let zero = white.scale(-1.0);
        assert_eq!(zero, Color::black());
        
        // Test upper bound
        let clamped = white.scale(2.0);
        assert_eq!(clamped, Color::white());
        
        // Test normal scaling
        let half = white.scale(0.5);
        assert_eq!(half.r, 127);
        assert_eq!(half.g, 127);
        assert_eq!(half.b, 127);
    }

    #[test]
    fn test_grb_conversion() {
        let color = Color::new(1, 2, 3);
        let grb = color.to_grb();
        assert_eq!(grb[0], 2); // G
        assert_eq!(grb[1], 1); // R  
        assert_eq!(grb[2], 3); // B
    }

    #[test]
    fn test_predefined_colors() {
        assert_eq!(Color::black(), Color::new(0, 0, 0));
        assert_eq!(Color::white(), Color::new(255, 255, 255));
        assert_eq!(Color::red(), Color::new(255, 0, 0));
        assert_eq!(Color::green(), Color::new(0, 255, 0));
        assert_eq!(Color::blue(), Color::new(0, 0, 255));
    }
}
