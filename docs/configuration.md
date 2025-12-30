# Configuration Options

Complete reference for `CableConfig` parameters and audio formats.

## CableConfig Structure

```rust
pub struct CableConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub format: AudioFormat,
    pub device_name: String,
}
```

### sample_rate (u32)

Sample rate in Hertz. Determines audio quality and CPU usage.

**Common values:**
- `44100` - CD quality, good balance
- `48000` - Professional audio, widely supported
- `96000` - High-resolution audio

**Trade-offs:**
| Sample Rate | Quality | CPU Usage | Latency |
|-------------|----------|------------|----------|
| 44100       | Good     | Low        | Medium   |
| 48000       | Better   | Medium     | Medium   |
| 96000       | Best     | High       | Low      |

```rust
let config = CableConfig {
    sample_rate: 48000,  // Most common
    ..Default::default()
};
```

### channels (u16)

Number of audio channels.

- `1` - Mono (single channel)
- `2` - Stereo (left/right channels)

**When to use:**
- **Mono**: Podcasts, voice chat, games
- **Stereo**: Music, movies, professional audio

```rust
// For voice applications
let config = CableConfig {
    channels: 1,
    ..Default::default()
};

// For music
let config = CableConfig {
    channels: 2,
    ..Default::default()
};
```

### buffer_size (usize)

Buffer size in audio samples. Affects latency and stability.

**Typical values:**
- `256-512` - Ultra-low latency (gaming, real-time)
- `1024-2048` - Balanced (most applications)
- `4096` - High stability, high latency

**Calculate latency:**
```rust
let latency_ms = (buffer_size as f64 * 1000.0) / sample_rate as f64;

// Example: 1024 samples @ 48000 Hz
// latency = 1024 * 1000 / 48000 = 21.33 ms
```

**Trade-offs:**
| Buffer Size | Latency | Stability | CPU Usage |
|-------------|----------|------------|------------|
| 256         | ~5ms     | Low        | High       |
| 512         | ~11ms    | Medium     | Medium     |
| 1024        | ~21ms    | Good       | Low        |
| 2048        | ~43ms    | Better     | Very Low   |
| 4096        | ~85ms    | Best       | Very Low   |

```rust
// Low latency (for gaming/real-time)
let config = CableConfig {
    buffer_size: 512,
    ..Default::default()
};

// High stability (for recording)
let config = CableConfig {
    buffer_size: 2048,
    ..Default::default()
};
```

### format (AudioFormat)

Audio data format. Affects quality and processing overhead.

```rust
pub enum AudioFormat {
    F32LE,  // 32-bit float, little-endian
    S16LE,  // 16-bit integer, little-endian
    S24LE,  // 24-bit integer, little-endian
    S32LE,  // 32-bit integer, little-endian
}
```

**Comparison:**

| Format | Bytes/Sample | Quality | Processing | Use Case |
|---------|---------------|----------|-------------|-----------|
| S16LE   | 2             | Good     | Fastest     | Games, VoIP |
| S24LE   | 3             | Better   | Fast        | Professional |
| S32LE   | 4             | Better   | Medium      | High-end audio |
| F32LE   | 4             | Best     | Slower      | Music, production |

```rust
// Best quality (music production)
let config = CableConfig {
    format: AudioFormat::F32LE,
    ..Default::default()
};

// Fastest processing (real-time)
let config = CableConfig {
    format: AudioFormat::S16LE,
    ..Default::default()
};
```

### device_name (String)

Display name for the virtual audio device.

**Guidelines:**
- Keep it short and descriptive
- Avoid special characters
- Make it unique if multiple devices

```rust
let config = CableConfig {
    device_name: "My Virtual Mic".to_string(),
    ..Default::default()
};
```

## Predefined Configurations

### Low Latency (Real-time)

```rust
let config = CableConfig {
    sample_rate: 48000,
    channels: 1,
    buffer_size: 512,
    format: AudioFormat::S16LE,
    device_name: "Low Latency Mic".to_string(),
};
```
**Use for:** Gaming, video conferencing, real-time monitoring

### High Quality

```rust
let config = CableConfig {
    sample_rate: 96000,
    channels: 2,
    buffer_size: 4096,
    format: AudioFormat::F32LE,
    device_name: "High Quality Mic".to_string(),
};
```
**Use for:** Music production, audio recording, mastering

### Balanced

```rust
let config = CableConfig {
    sample_rate: 48000,
    channels: 2,
    buffer_size: 1024,
    format: AudioFormat::F32LE,
    device_name: "Balanced Mic".to_string(),
};
```
**Use for:** Most applications, general purpose

### Voice Chat

```rust
let config = CableConfig {
    sample_rate: 48000,
    channels: 1,
    buffer_size: 2048,
    format: AudioFormat::S16LE,
    device_name: "Voice Chat Mic".to_string(),
};
```
**Use for:** Discord, Zoom, Teams, Skype

## Default Configuration

```rust
impl Default for CableConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 1024,
            format: AudioFormat::F32LE,
            device_name: "Virtual Audio Cable".to_string(),
        }
    }
}
```

Use it as a starting point:

```rust
let mut config = CableConfig::default();
config.device_name = "My Mic".to_string();
```

## Configuration Presets Helper

Create a helper function for common presets:

```rust
fn create_config(preset: ConfigPreset) -> CableConfig {
    match preset {
        ConfigPreset::LowLatency => CableConfig {
            sample_rate: 48000,
            channels: 1,
            buffer_size: 512,
            format: AudioFormat::S16LE,
            device_name: "Low Latency".to_string(),
        },
        ConfigPreset::HighQuality => CableConfig {
            sample_rate: 96000,
            channels: 2,
            buffer_size: 4096,
            format: AudioFormat::F32LE,
            device_name: "High Quality".to_string(),
        },
        ConfigPreset::Balanced => CableConfig::default(),
    }
}

enum ConfigPreset {
    LowLatency,
    HighQuality,
    Balanced,
}
```

## Optimization Tips

1. **Start with defaults**
   ```rust
   let config = CableConfig::default();
   ```

2. **Monitor statistics**
   ```rust
   let stats = cable.get_stats();
   if stats.underruns > 10 {
       // Increase buffer_size
   }
   if stats.latency_ms > 50.0 {
       // Decrease buffer_size
   }
   ```

3. **Adjust incrementally**
   - Change one parameter at a time
   - Test thoroughly
   - Monitor for issues

4. **Consider use case**
   - Real-time → low buffer, fast format
   - Recording → high sample rate, large buffer
   - Voice → mono, medium settings

## Platform Considerations

### Linux (PipeWire)
- Lower overhead with `S16LE`
- Works well with 48000 Hz
- PipeWire handles format conversion

### Windows (WDM/WaveRT)
- `F32LE` is native format
- Supports all sample rates well
- Larger buffers more stable

## Next Steps

- [Quick Start](./quick-start.md) - Get started with config
- [API Overview](./api-overview.md) - Core API types
- [Monitoring](./monitoring.md) - Track performance
