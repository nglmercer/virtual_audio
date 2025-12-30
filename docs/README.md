# Virtual Audio Cable Documentation

Welcome to the Virtual Audio Cable documentation. This library provides cross-platform virtual audio routing capabilities in Rust.

## 📚 Documentation Index

### Getting Started
- [Installation Guide](./installation.md) - How to install and set up the library
- [Quick Start](./quick-start.md) - Get up and running in minutes
- [API Overview](./api-overview.md) - High-level API concepts

### Core Concepts
- [Architecture](./architecture.md) - Library architecture and design
- [Buffer Management](./buffers.md) - Ring buffers and triple buffer architecture
- [Audio Processing](./audio-processing.md) - Audio format conversion and resampling

### Platform-Specific
- [Linux Support](./platform-linux.md) - PipeWire integration on Linux
- [Windows Support](./platform-windows.md) - WDM/WaveRT driver on Windows

### Examples
- [Basic Virtual Microphone](./example-microphone.md) - Create a virtual microphone
- [Audio Routing](./example-routing.md) - Route audio between applications
- [Advanced Configuration](./example-config.md) - Custom cable configuration

### Reference
- [Configuration Options](./configuration.md) - All CableConfig parameters
- [Error Handling](./errors.md) - Error types and handling
- [Statistics & Monitoring](./monitoring.md) - Performance metrics

### Contributing
- [Development Guide](./development.md) - Setup for contributing
- [Testing](./testing.md) - Run and write tests
- [Building](./building.md) - Build from source

## 🔍 Quick Links

- [GitHub Repository](https://github.com/yourusername/virtual-audio-cable)
- [API Documentation](https://docs.rs/virtual-audio-cable)
- [Examples](../examples)
- [Release Notes](../CHANGELOG.md)

## 📝 License

MIT OR Apache-2.0 - See [LICENSE](../LICENSE) for details.

---

**Need help?** Check the [examples](../examples) or open an issue on GitHub.
