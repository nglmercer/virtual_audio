# Quick Start Guide

Get started with Virtual Audio Cable in just a few minutes!

## Your First Virtual Microphone

This simple example creates a virtual microphone that captures system audio:

```rust
use virtual_audio_cable::{AudioFormat, CableConfig, VirtualCable, VirtualCableTrait};

fn main() -> anyhow::Result<()> {
    // Create default configuration
    let config = CableConfig::default();
    
    // Create virtual cable
    let mut cable = VirtualCable::new(config)?;
    
    // Start the cable
    cable.start()?;
    println!("Virtual microphone is running!");
    
    // Keep running...
    // In real app, add signal handling here
    
    Ok(())
}
```

## Step-by-Step Guide

### 1. Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
virtual_audio = "0.1.0"
tokio = { version = "1.35", features = ["full"] }
anyhow = "1.0"
log = "0.4"
env_logger = "0.11"
```

### 2. Create Configuration

```rust
let config = CableConfig {
    sample_rate: 48000,      // 48 kHz (CD quality)
    channels: 2,            // Stereo
    buffer_size: 1024,       // Buffer size
    format: AudioFormat::F32LE, // 32-bit float
    device_name: "My Virtual Mic".to_string(),
};
```

### 3. Create and Start Cable

```rust
let mut cable = VirtualCable::new(config)?;
cable.start()?;
```

### 4. Monitor Statistics

```rust
use std::time::Duration;

let stats = cable.get_stats();
println!("Samples: {}", stats.samples_processed);
println!("Latency: {:.2}ms", stats.latency_ms);
println!("CPU: {:.1}%", stats.cpu_usage);
```

### 5. Stop When Done

```rust
cable.stop()?;
```

## Complete Working Example

Here's a complete example with signal handling:

```rust
use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio::signal;
use virtual_audio_cable::{CableConfig, VirtualCable, VirtualCableTrait};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    // Create cable
    let cable = Arc::new(std::sync::Mutex::new(
        VirtualCable::new(CableConfig::default())?
    ));
    
    // Start it
    cable.lock().unwrap().start()?;
    info!("Virtual cable started!");
    
    // Wait for Ctrl+C
    signal::ctrl_c().await?;
    info!("Shutting down...");
    
    // Stop it
    cable.lock().unwrap().stop()?;
    info!("Done!");
    
    Ok(())
}
```

## Using Your Virtual Device

Once running, your virtual audio device appears in your system:

### On Linux (PipeWire):
1. Open your audio application (Zoom, OBS, etc.)
2. Go to audio settings
3. Select "Virtual Audio Cable" as microphone

### On Windows:
1. Open Sound Settings
2. Select "Virtual Audio Cable" as input device
3. Use it in any application

## Configuration Tips

### Low Latency (Real-time audio)
```rust
let config = CableConfig {
    sample_rate: 48000,
    channels: 1,           // Mono is faster
    buffer_size: 512,      // Small buffer
    format: AudioFormat::S16LE, // Faster than F32
    device_name: "Low Latency Mic".to_string(),
};
```

### High Quality (Music production)
```rust
let config = CableConfig {
    sample_rate: 96000,    // High sample rate
    channels: 2,           // Stereo
    buffer_size: 4096,     // Large buffer
    format: AudioFormat::F32LE, // Best quality
    device_name: "High Quality Mic".to_string(),
};
```

## Common Patterns

### Async with Monitoring

```rust
let cable = Arc::new(std::sync::Mutex::new(
    VirtualCable::new(CableConfig::default())?
));

// Start monitoring task
let cable_clone = Arc::clone(&cable);
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let stats = cable_clone.lock().unwrap().get_stats();
        println!("Latency: {:.2}ms", stats.latency_ms);
    }
});

// Main loop
signal::ctrl_c().await?;
```

### Error Handling

```rust
match VirtualCable::new(config) {
    Ok(cable) => {
        cable.start()?;
        // Use cable...
    }
    Err(e) => {
        eprintln!("Failed to create cable: {}", e);
        std::process::exit(1);
    }
}
```

## Next Steps

- [API Overview](./api-overview.md) - Learn about the API structure
- [Configuration](./configuration.md) - All configuration options
- [Examples](../examples) - More complete examples
- [Platform-Specific](./platform-linux.md) - Linux/Windows details

## Troubleshooting

### "Device not found"
- Check if PipeWire (Linux) or Windows Audio service is running
- Verify your platform feature is enabled

### High latency
- Reduce `buffer_size`
- Consider mono instead of stereo

### CPU usage too high
- Increase `buffer_size`
- Use lower sample rate
