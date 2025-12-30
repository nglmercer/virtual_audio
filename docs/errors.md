# Error Handling

Complete guide to handling errors in Virtual Audio Cable.

## Error Types

```rust
pub enum Error {
    BufferError(String),
    AudioError(String),
    PlatformError(String),
    IoError(std::io::Error),
    Other(String),
}
```

## Basic Error Handling

### Using `?` Operator

```rust
use anyhow::Result;

fn create_cable() -> Result<VirtualCable> {
    let config = CableConfig::default();
    let cable = VirtualCable::new(config)?;  // Returns Result
    Ok(cable)
}
```

### Match on Errors

```rust
use virtual_audio_cable::Error;

match VirtualCable::new(config) {
    Ok(cable) => {
        println!("Cable created successfully");
    }
    Err(Error::PlatformError(msg)) => {
        eprintln!("Platform error: {}", msg);
    }
    Err(Error::BufferError(msg)) => {
        eprintln!("Buffer error: {}", msg);
    }
    Err(Error::AudioError(msg)) => {
        eprintln!("Audio error: {}", msg);
    }
    Err(Error::IoError(e)) => {
        eprintln!("IO error: {}", e);
    }
    Err(Error::Other(msg)) => {
        eprintln!("Other error: {}", msg);
    }
}
```

## Error Variants

### BufferError

Buffer-related errors (ring buffer, triple buffer).

**Examples:**
- Buffer allocation failed
- Buffer size invalid
- Buffer access violation

```rust
Err(Error::BufferError(msg)) => {
    eprintln!("Buffer error: {}", msg);
    // Try increasing buffer size or checking memory
}
```

### AudioError

Audio processing errors (format conversion, resampling).

**Examples:**
- Unsupported audio format
- Sample rate mismatch
- Resampling failed

```rust
Err(Error::AudioError(msg)) => {
    eprintln!("Audio error: {}", msg);
    // Check format and sample rate configuration
}
```

### PlatformError

Platform-specific errors (PipeWire, Windows Audio).

**Examples:**
- PipeWire connection failed (Linux)
- WDM driver not found (Windows)
- Audio device not available

```rust
Err(Error::PlatformError(msg)) => {
    eprintln!("Platform error: {}", msg);
    // Check audio daemon (PipeWire/Windows Audio) is running
}
```

### IoError

Standard I/O errors from the system.

**Examples:**
- File system errors
- Device access denied
- Permission errors

```rust
Err(Error::IoError(e)) => {
    eprintln!("IO error: {}", e);
    // Check permissions and file access
}
```

### Other

Generic errors that don't fit other categories.

```rust
Err(Error::Other(msg)) => {
    eprintln!("Error: {}", msg);
    // Handle other unexpected errors
}
```

## Error Handling Patterns

### Retry Pattern

```rust
use std::time::Duration;

fn create_cable_with_retry(max_retries: usize) -> Result<VirtualCable> {
    let config = CableConfig::default();
    
    for attempt in 1..=max_retries {
        match VirtualCable::new(config.clone()) {
            Ok(cable) => {
                println!("Cable created on attempt {}", attempt);
                return Ok(cable);
            }
            Err(e) if attempt < max_retries => {
                println!("Attempt {} failed: {}. Retrying...", attempt, e);
                std::thread::sleep(Duration::from_secs(1));
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    
    unreachable!()
}
```

### Fallback Pattern

```rust
fn create_cable_with_fallback() -> VirtualCable {
    // Try high quality config first
    let high_quality = CableConfig {
        sample_rate: 96000,
        format: AudioFormat::F32LE,
        ..Default::default()
    };
    
    match VirtualCable::new(high_quality) {
        Ok(cable) => {
            println!("Using high quality configuration");
            cable
        }
        Err(_) => {
            println!("High quality failed, falling back to default");
            VirtualCable::new(CableConfig::default())
                .expect("Default config should work")
        }
    }
}
```

### Error Recovery

```rust
async fn run_cable_with_recovery(
    config: CableConfig
) -> anyhow::Result<()> {
    let cable = Arc::new(std::sync::Mutex::new(
        VirtualCable::new(config)?
    ));
    
    loop {
        // Try to start
        match cable.lock().unwrap().start() {
            Ok(_) => {
                println!("Cable started successfully");
                
                // Run until failure
                match run_until_failure(&cable).await {
                    Ok(_) => break,
                    Err(e) => {
                        println!("Cable failed: {}", e);
                        // Continue to recovery
                    }
                }
            }
            Err(e) => {
                println!("Failed to start: {}", e);
            }
        }
        
        // Recovery strategy
        println!("Attempting recovery...");
        cable.lock().unwrap().stop()?;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    
    Ok(())
}
```

## Common Errors and Solutions

### "Failed to connect to PipeWire"

**Linux only. PipeWire audio daemon is not running or not accessible.**

**Solutions:**
```bash
# Check if PipeWire is running
systemctl --user status pipewire

# Start PipeWire if needed
systemctl --user start pipewire

# Enable auto-start
systemctl --user enable pipewire
```

**Code workaround:**
```rust
Err(Error::PlatformError(msg)) if msg.contains("PipeWire") => {
    eprintln!("PipeWire error: {}", msg);
    eprintln!("Please start PipeWire: systemctl --user start pipewire");
    std::process::exit(1);
}
```

### "Device not found"

**Virtual audio device cannot be created or accessed.**

**Solutions:**
1. Check audio service is running
2. Verify permissions
3. Try different device name

```rust
Err(Error::PlatformError(msg)) if msg.contains("not found") => {
    eprintln!("Device not found: {}", msg);
    eprintln!("Try: 1. Restart audio service");
    eprintln!("      2. Check device name is unique");
    eprintln!("      3. Verify audio permissions");
}
```

### "Buffer size invalid"

**Buffer size is outside valid range.**

**Valid range:** 256 to 8192 samples

**Solution:**
```rust
let buffer_size = 1024.min(8192).max(256);  // Clamp to valid range

let config = CableConfig {
    buffer_size,
    ..Default::default()
};
```

### "Unsupported audio format"

**Audio format is not supported by the platform.**

**Solution:**
```rust
// Try F32LE (most supported)
let format = AudioFormat::F32LE;

match VirtualCable::new(CableConfig {
    format,
    ..Default::default()
}) {
    Ok(cable) => cable,
    Err(_) => {
        // Fallback to S16LE
        VirtualCable::new(CableConfig {
            format: AudioFormat::S16LE,
            ..Default::default()
        }).unwrap()
    }
}
```

## Error Logging

### Structured Error Logging

```rust
use log::error;

match VirtualCable::new(config) {
    Ok(cable) => Ok(cable),
    Err(e) => {
        error!(
            error_type = match &e {
                Error::PlatformError(_) => "platform",
                Error::BufferError(_) => "buffer",
                Error::AudioError(_) => "audio",
                Error::IoError(_) => "io",
                Error::Other(_) => "other",
            },
            error_message = %e,
            "Failed to create cable"
        );
        Err(e.into())
    }
}
```

### Error Context with anyhow

```rust
use anyhow::Context;

let cable = VirtualCable::new(config)
    .context("Failed to create virtual cable")?;

cable.start()
    .context("Failed to start cable")?;
```

### Custom Error Display

```rust
fn format_error(err: &Error) -> String {
    match err {
        Error::PlatformError(msg) => {
            format!("Platform error: {}. Check audio daemon is running.", msg)
        }
        Error::BufferError(msg) => {
            format!("Buffer error: {}. Try different buffer size.", msg)
        }
        Error::AudioError(msg) => {
            format!("Audio error: {}. Check format and sample rate.", msg)
        }
        Error::IoError(e) => {
            format!("IO error: {}. Check permissions.", e)
        }
        Error::Other(msg) => {
            format!("Error: {}", msg)
        }
    }
}
```

## Testing Error Handling

### Unit Tests

```rust
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_invalid_buffer_size() {
        let config = CableConfig {
            buffer_size: 100,  // Too small
            ..Default::default()
        };
        
        match VirtualCable::new(config) {
            Ok(_) => panic!("Should fail with invalid buffer size"),
            Err(_) => {},  // Expected
        }
    }

    #[test]
    fn test_invalid_sample_rate() {
        let config = CableConfig {
            sample_rate: 0,  // Invalid
            ..Default::default()
        };
        
        assert!(VirtualCable::new(config).is_err());
    }
}
```

### Error Propagation Tests

```rust
#[test]
fn test_error_propagation() -> anyhow::Result<()> {
    let config = CableConfig {
        device_name: "".to_string(),  // Invalid
        ..Default::default()
    };
    
    let result: anyhow::Result<VirtualCable> = VirtualCable::new(config)
        .map_err(|e| anyhow::anyhow!("Failed to create: {}", e));
    
    assert!(result.is_err());
    Ok(())
}
```

## Best Practices

1. **Always handle errors explicitly**
   ```rust
   // Good
   match VirtualCable::new(config) {
       Ok(cable) => /* use cable */,
       Err(e) => eprintln!("Error: {}", e),
   }
   
   // Bad
   let cable = VirtualCable::new(config).unwrap();  // May panic!
   ```

2. **Provide helpful error messages**
   ```rust
   // Good
   Err(e) => eprintln!("Failed to create cable: {}. Check audio daemon.", e),
   
   // Bad
   Err(e) => eprintln!("Error: {}", e),
   ```

3. **Use context with anyhow**
   ```rust
   VirtualCable::new(config)
       .context("Failed to initialize virtual microphone")?;
   ```

4. **Log errors for debugging**
   ```rust
   match VirtualCable::new(config) {
       Ok(cable) => Ok(cable),
       Err(e) => {
           log::error!("Failed to create cable: {}", e);
           Err(e.into())
       }
   }
   ```

5. **Implement recovery strategies**
   ```rust
   loop {
       match cable.start() {
           Ok(_) => break,
           Err(e) => {
               log::warn!("Failed to start: {}. Retrying...", e);
               tokio::time::sleep(Duration::from_secs(1)).await;
           }
       }
   }
   ```

## Next Steps

- [API Overview](./api-overview.md) - Core API reference
- [Monitoring](./monitoring.md) - Track performance
- [Configuration](./configuration.md) - Avoid configuration errors
- [Example Microphone](./example-microphone.md) - See error handling in action
