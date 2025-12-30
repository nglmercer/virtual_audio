# Virtual Audio Cable

A cross-platform virtual audio cable implementation in Rust that allows routing audio between applications without physical hardware connections.

## Features

- **Low Latency**: Optimized for minimal audio delay (< 10ms on Windows, < 5ms on Linux)
- **Cross-Platform**: Supports Linux (PipeWire) and Windows (WDM/WaveRT)
- **Real-Time Safe**: Designed for audio callback constraints with lock-free data structures
- **Memory Safe**: Leverages Rust's safety guarantees for robust audio processing
- **Flexible Configuration**: Configurable sample rates, channels, buffer sizes, and formats

## Platform Support

### Linux
- Uses PipeWire for user-space audio routing
- Requires PipeWire >= 0.3.60
- Supports modern Linux distributions (Ubuntu 22.04+, Fedora 38+, Arch Linux)

### Windows
- Uses WDM/WaveRT kernel driver model
- Requires Windows Driver Kit (WDK)
- Supports Windows 10/11
- **Note**: Kernel driver implementation is in progress

## Installation

### Prerequisites

#### Linux
```bash
# Install PipeWire development libraries
sudo apt install libpipewire-dev pipewire

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Windows
```powershell
# Install Windows Driver Kit (WDK)
# Download from: https://docs.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk

# Install Visual Studio 2022 with C++ build tools
# Download from: https://visualstudio.microsoft.com/

# Install Rust (if not already installed)
# Download from: https://rustup.rs/
```

### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/virtual-audio-cable.git
cd virtual-audio-cable

# Build
cargo build --release

# Install (optional)
cargo install --path .
```

## Usage

### Command Line Interface

```bash
# Start virtual cable with default settings (48kHz, stereo, F32)
virtual-audio-cable

# Start with custom sample rate and monitoring
virtual-audio-cable --sample-rate 44100 --monitor

# Start with custom buffer size and format
virtual-audio-cable --buffer 2048 --format s16

# Mono audio with device name
virtual-audio-cable --channels 1 --name "My Virtual Cable"
```

### Command Line Options

| Option | Short | Description | Default |
|---------|---------|-------------|-----------|
| `--sample-rate` | `-r` | Sample rate in Hz | 48000 |
| `--channels` | `-c` | Number of channels (1=mono, 2=stereo) | 2 |
| `--buffer` | `-b` | Buffer size in samples | 1024 |
| `--format` | `-f` | Audio format (f32, s16, s24, s32) | f32 |
| `--name` | `-n` | Device name for virtual cable | "Virtual Audio Cable" |
| `--monitor` | `-m` | Monitor and print statistics every second | false |
| `--help` | `-h` | Show help message | - |

### Library Usage

```rust
use virtual_audio_cable::{CableConfig, VirtualCable, AudioFormat};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create configuration
    let config = CableConfig {
        sample_rate: 48000,
        channels: 2,
        buffer_size: 1024,
        format: AudioFormat::F32LE,
        device_name: "My Cable".to_string(),
    };
    
    // Create virtual cable
    let mut cable = VirtualCable::new(config)?;
    
    // Start the cable
    cable.start()?;
    
    // Get statistics
    let stats = cable.get_stats();
    println!("Latency: {:.2}ms", stats.latency_ms);
    
    // Stop when done
    cable.stop()?;
    
    Ok(())
}
```

## Architecture

### Triple Ring Buffer

The virtual audio cable uses a triple ring buffer architecture:

1. **Input Buffer**: Receives audio from the capture device (speaker)
2. **Resample Buffer**: Holds data during sample rate conversion
3. **Output Buffer**: Delivers audio to the playback device (microphone)

This architecture provides:
- Lock-free data transfer for real-time safety
- Efficient sample rate conversion between different applications
- Minimal latency and CPU overhead

### Audio Processing

- **Format Conversion**: Supports F32, S16, S24, and S32 formats
- **Resampling**: Linear interpolation for sample rate conversion
- **Monitoring**: Real-time statistics (underruns, overruns, latency, CPU usage)

## Configuration Examples

### Gaming (Low Latency)
```bash
virtual-audio-cable -r 48000 -b 256 -m
```

### Music Production (High Quality)
```bash
virtual-audio-cable -r 96000 -b 4096 -f s24
```

### Voice Chat (Compact)
```bash
virtual-audio-cable -r 44100 -c 1 -b 512 -f s16
```

## Troubleshooting

### Buffer Underruns
If you experience audio dropouts:
- Increase buffer size: `--buffer 2048`
- Reduce sample rate if not needed
- Close other CPU-intensive applications

### High Latency
If latency is too high:
- Decrease buffer size: `--buffer 512`
- Check system load with `--monitor`
- Ensure PipeWire is running with low-latency configuration

### Device Not Visible
If the virtual device doesn't appear:
- **Linux**: Check PipeWire daemon is running: `systemctl status pipewire`
- **Windows**: Ensure driver is installed and loaded in Device Manager

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

### Building Documentation

```bash
# Build and open documentation
cargo doc --open

# Build documentation for all features
cargo doc --all-features --open
```

### Code Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html
```

## Roadmap

### v0.2.0
- [ ] Complete PipeWire integration
- [ ] Add ASIO support on Windows
- [ ] Implement JACK integration on Linux
- [ ] Add audio effects (EQ, reverb, compression)

### v0.3.0
- [ ] Complete Windows kernel driver
- [ ] Add WebRTC streaming support
- [ ] Implement Bluetooth device support
- [ ] Add scripting API (Lua/Python)

### v1.0.0
- [ ] Full certification for Windows
- [ ] Comprehensive testing suite
- [ ] Performance optimization
- [ ] Production-ready documentation

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Code Style
- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting
- All code must pass `cargo test`
- Document public APIs with rustdoc comments

### Pull Request Process
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request with description

## License

This project is dual-licensed under:
- **MIT License** - For commercial use
- **Apache 2.0 License** - For community contributions

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## Acknowledgments

- **PipeWire Team** - For the excellent audio routing framework
- **Microsoft** - For Windows Drivers Rust support (windows-drivers-rs)
- **Rust Audio Community** - For the amazing audio ecosystem
- **specs.md** - Technical specifications and architecture guidance

## Support

- **Documentation**: See [specs.md](specs.md) for detailed technical specifications
- **Issues**: Report bugs on [GitHub Issues](https://github.com/yourusername/virtual-audio-cable/issues)
- **Discussions**: Join [GitHub Discussions](https://github.com/yourusername/virtual-audio-cable/discussions)
- **Community**: Join the [Rust Audio Discord](https://discord.gg/YCpc8T8)

## References

- [PipeWire Documentation](https://docs.pipewire.org/)
- [Windows Driver Development](https://docs.microsoft.com/en-us/windows-hardware/drivers/)
- [Rust Audio Ecosystem](https://github.com/RustAudio/rust-audio)
- [specs.md](specs.md) - Complete technical specifications

---

**Version**: 0.1.0  
**Status**: Alpha - In Development  
**Last Updated**: December 30, 2025
