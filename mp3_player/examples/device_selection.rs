use anyhow::Result;
use mp3_player::Player;
use std::io::{self, Write};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Audio Device Selection Example");
    println!("==============================\n");

    // Create a new player
    let mut player = Player::new();

    // List available audio devices
    println!("Available audio output devices:");
    let devices = Player::list_output_devices()?;
    
    if devices.is_empty() {
        println!("No audio output devices found!");
        return Ok(());
    }

    for (i, (name, _)) in devices.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }

    // Show current device
    println!("\nCurrent device: {}", player.get_current_device_name()?);

    // Get user input for device selection
    print!("\nEnter device number (or press Enter for default): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if !input.trim().is_empty() {
        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= devices.len() {
                let (device_name, device) = &devices[choice - 1];
                println!("Setting audio device to: {}", device_name);
                
                player.set_device(device.clone()).await?;
                println!("Device set successfully!");
                println!("Current device: {}", player.get_current_device_name()?);
            } else {
                println!("Invalid choice, using default device");
            }
        } else {
            println!("Invalid input, using default device");
        }
    } else {
        println!("Using default device");
    }

    // Example of setting device by name
    println!("\n=== Setting Device by Name ===");
    print!("Enter device name (partial match, or Enter to skip): ");
    io::stdout().flush()?;
    
    let mut device_name_input = String::new();
    io::stdin().read_line(&mut device_name_input)?;
    
    if !device_name_input.trim().is_empty() {
        match player.set_device_by_name(device_name_input.trim()).await {
            Ok(()) => {
                println!("Successfully set device!");
                println!("Current device: {}", player.get_current_device_name()?);
            }
            Err(e) => println!("Failed to set device: {}", e),
        }
    }

    // Test audio playback if a file is provided
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let audio_file = PathBuf::from(&args[1]);
        
        println!("\n=== Testing Audio Playback ===");
        println!("Loading audio file: {}", audio_file.display());
        
        match player.load_track(audio_file).await {
            Ok(()) => {
                println!("Audio file loaded successfully!");
                
                if let Some(duration) = player.get_duration() {
                    println!("Duration: {:.2} seconds", duration.as_secs_f32());
                }
                
                println!("Playing for 5 seconds...");
                player.play().await?;
                
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                
                println!("Stopping playback...");
                player.stop().await?;
                
                println!("Playback test completed!");
            }
            Err(e) => println!("Failed to load audio file: {}", e),
        }
    } else {
        println!("\nTo test audio playback, run:");
        println!("cargo run --example device_selection -- /path/to/audio/file.mp3");
    }

    println!("\nExample completed!");
    Ok(())
}
