# Audio Device Selection in Rust

This document explains how to implement audio device selection for your Rust MP3 player using rodio and cpal.

## Overview

To open a specific audio device for `OutputStream` in rodio, you need to:

1. **List Available Devices** - Use `cpal` to enumerate audio devices
2. **Select Device** - Choose device by name or index
3. **Create Stream** - Create `OutputStream` with specific device
4. **Create Sink** - Connect `Sink` to the stream handle

## Implementation Approach

### 1. Dependencies Required

```toml
[dependencies]
rodio = "0.21.1"
cpal = "0.15"
```

### 2. Device Enumeration

```rust
use cpal::{traits::{DeviceTrait, HostTrait}};

pub fn list_output_devices() -> Result<Vec<(String, cpal::Device)>, cpal::DevicesError> {
    let host = cpal::default_host();
    let mut devices = Vec::new();
    
    for device in host.output_devices()? {
        if let Ok(name) = device.name() {
            devices.push((name, device));
        }
    }
    
    Ok(devices)
}
```

### 3. Device Selection (Correct API - TODO)

The challenge is finding the correct rodio v0.21 API for device-specific stream creation:

```rust
// THESE ARE THE METHODS WE NEED TO FIND/CONFIRM:

// Option 1: Direct device stream creation
let (stream, stream_handle) = rodio::OutputStream::try_from_device(&device)?;

// Option 2: Using device config
let (stream, stream_handle) = rodio::OutputStream::try_from_device_config(
    &device, 
    &config
)?;

// Option 3: Using cpal directly with rodio wrapper
// (This might be the correct approach)
```

### 4. Current Working Implementation

Based on our current understanding, here's what works:

```rust
use rodio::{Decoder, OutputStream, Sink, Source};
use cpal::{Device, traits::{DeviceTrait, HostTrait}};

pub struct Player {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
    current_track: Option<PathBuf>,
    volume: f32,
    current_device: Option<Device>,
}

impl Player {
    // Device listing (works)
    pub fn list_output_devices() -> Result<Vec<(String, Device)>, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        
        for device in host.output_devices()? {
            if let Ok(name) = device.name() {
                devices.push((name, device));
            }
        }
        
        Ok(devices)
    }

    // Device selection storage (works)
    pub fn set_device(&mut self, device: Device) -> Result<()> {
        self.current_device = Some(device);
        Ok(())
    }

    // Stream creation (NEEDS CORRECT API)
    pub fn create_stream(&mut self) -> Result<()> {
        // TODO: Replace with correct rodio API
        // This is where we need the right method:
        
        match &self.current_device {
            Some(device) => {
                // NEED: rodio method to create stream from device
                // let (stream, handle) = rodio::OutputStream::from_device(device)?;
                unimplemented!("Need correct rodio API for device-specific streams");
            }
            None => {
                // Default stream creation
                let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
                // Continue with default...
            }
        }
    }
}
```

## Research Needed

To complete the implementation, we need to determine:

### 1. Correct Rodio API

Check the rodio v0.21 documentation for:
- `OutputStream::try_from_device()`
- `OutputStream::from_device_config()`  
- Alternative device-specific stream creation methods

### 2. CPAL Integration

Since rodio uses cpal internally, we might need to:
- Create cpal stream directly
- Wrap it with rodio components
- Use rodio's device configuration methods

### 3. Stream Handle Management

Understand how to:
- Get the stream handle from device-specific streams
- Connect `Sink` to device-specific stream handles
- Maintain proper cleanup

## Next Steps

1. **Research rodio v0.21 docs** for correct device API
2. **Test with simple device selection** example
3. **Integrate into full player** once API is confirmed
4. **Add error handling** for device disconnection
5. **Add device change detection** for robust UX

## Alternative Approaches

If rodio doesn't provide direct device selection:

### 1. CPAL Direct Approach
Use cpal directly for audio output with manual sample conversion

### 2. System Audio Routing
Use system-level audio routing (PulseAudio, ALSA, etc.)

### 3. Multiple Player Instances
Create separate player instances per device

## Testing Checklist

- [ ] List available devices
- [ ] Select device by name
- [ ] Select device by index
- [ ] Handle device not found errors
- [ ] Test audio output to specific device
- [ ] Handle device disconnection
- [ ] Verify audio routing

## Known Issues

1. **API Uncertainty**: Need to confirm correct rodio v0.21 device API
2. **Stream Handle**: Proper management of device-specific stream handles
3. **Error Handling**: Device errors and fallback to default
4. **Platform Differences**: macOS/Windows/Linux device handling variations

This framework provides the structure needed once we identify the correct rodio API calls.
