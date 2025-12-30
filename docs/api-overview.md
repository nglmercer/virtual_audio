# API Overview

This document provides a high-level overview of the Virtual Audio Cable API.

## Core Types

### VirtualCable

The main type representing a virtual audio cable.

```rust
use virtual_audio_cable::{VirtualCable, VirtualCableTrait};

// Create cable
let mut cable = VirtualCable::new(config)?;

// Start/stop
cable.start()?;
cable.stop()?;

// Check status
if cable.is_running() {
    println!("Cable is active");
}

// Get statistics
let stats = cable.get_stats();
```

### VirtualCableTrait

Trait defining the virtual cable interface.

```rust
pub trait VirtualCableTrait: Send + Sync {
    fn new(config: CableConfig) -> Result<Self, Error>;
    fn start(&mut self) -> Result<(), Error>;
    fn stop(&mut self) -> Result<(), Error>;
    fn is_running(&self) -> bool;
    fn get_stats(&self) -> CableStats;
}
```

### CableConfig

Configuration for creating a virtual cable.

```rust
pub struct CableConfig {
    pub sample_rate: u32,        // Hz: 44100, 48000, 96000
    pub channels: u16,            // 1 = mono, 2 = stereo
    pub buffer_size: usize,        // 256-4096 samples
    pub format: AudioFormat,       // F32LE, S16LE, S24LE, S32LE
    pub device_name: String,       // Device display name
}
```

### AudioFormat

Audio format specification.

```rust
pub enum AudioFormat {
    F32LE,  // 32-bit float, little-endian (4 bytes/sample)
    S16LE,  // 16-bit int, little-endian (2 bytes/sample)
    S24LE,  // 24-bit int, little-endian (3 bytes/sample)
    S32LE,  // 32-bit int, little-endian (4 bytes/sample)
}
```

### CableStats

Real-time statistics about cable operation.

```rust
pub struct CableStats {
    pub is_running: bool,          // Is cable active?
    pub samples_processed: u64,    // Total samples processed
    pub underruns: u64,          // Buffer underflow events
    pub overruns: u64,           // Buffer overflow events
    pub latency_ms: f64,         // Current latency in ms
    pub cpu_usage: f64,          // CPU usage percentage
}
```

## Typical Workflow

```rust
use virtual_audio_cable::{CableConfig, VirtualCable, VirtualCableTrait};

fn main() -> anyhow::Result<()> {
    // 1. Configure
    let config = CableConfig {
        sample_rate: 48000,
        channels: 2,
        buffer_size: 1024,
        format: AudioFormat::F32LE,
        device_name: "My Cable".to_string(),
    };
    
    // 2. Create
    let mut cable = VirtualCable::new(config)?;
    
    // 3. Start
    cable.start()?;
    
    // 4. Monitor/Use
    loop {
        let stats = cable.get_stats();
        println!("Latency: {:.2}ms", stats.latency_ms);
        // ... your application logic
    }
    
    // 5. Stop
    cable.stop()?;
    
    Ok(())
}
```

## Platform-Specific Types

The library provides platform-specific implementations:

- **Linux**: `LinuxVirtualCable` - Uses PipeWire
- **Windows**: `WindowsVirtualCable` - Uses WDM/WaveRT

These are aliased as `VirtualCable` via conditional compilation.

## Error Handling

All operations return `Result<T, Error>`:

```rust
use virtual_audio_cable::Error;

match VirtualCable::new(config) {
    Ok(cable) => {
        cable.start()?;
    }
    Err(Error::PlatformError(msg)) => {
        eprintln!("Platform error: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Async Usage

For async applications, use `Arc<Mutex<>>`:

```rust
use std::sync::Arc;

let cable = Arc::new(std::sync::Mutex::new(
    VirtualCable::new(config)?
));

// Clone for different tasks
let cable_clone = Arc::clone(&cable);

tokio::spawn(async move {
    cable_clone.lock().unwrap().start().unwrap();
});
```

## Thread Safety

`VirtualCable` implements `Send + Sync`, allowing concurrent access:

```rust
use std::thread;

let cable = Arc::new(std::sync::Mutex::new(
    VirtualCable::new(config)?
));

// Monitor in one thread
let cable_monitor = Arc::clone(&cable);
thread::spawn(move || {
    loop {
        let stats = cable_monitor.lock().unwrap().get_stats();
        println!("Samples: {}", stats.samples_processed);
        thread::sleep(Duration::from_secs(1));
    }
});

// Control in main thread
cable.lock().unwrap().start()?;
```

## Lifecycle

```
   [Create] → [Start] → [Running] → [Stop] → [Stopped]
                    ↓                   ↓
               Error (if any)      Error (if any)
```

- **Create**: Allocate resources, validate config
- **Start**: Connect to audio system, begin processing
- **Running**: Actively routing audio
- **Stop**: Disconnect, clean up resources
- **Stopped**: Ready for restart or cleanup

## Best Practices

1. **Always check is_running() before operations**
   ```rust
   if cable.is_running() {
       // Safe to use
   }
   ```

2. **Monitor statistics for health**
   ```rust
   if stats.underruns > 10 {
       // Buffer too small
   }
   if stats.latency_ms > 100.0 {
       // High latency
   }
   ```

3. **Handle errors gracefully**
   ```rust
   match cable.start() {
       Ok(_) => {},
       Err(e) => log::error!("Failed to start: {}", e),
   }
   ```

4. **Stop before drop** (optional but recommended)
   ```rust
   cable.stop()?;  // Explicit stop
   // cable is dropped here
   ```

## Next Steps

- [Configuration](./configuration.md) - Detailed configuration options
- [Error Handling](./errors.md) - All error types
- [Monitoring](./monitoring.md) - Statistics in depth
- [Examples](../examples) - Working code examples
