# Installation Guide

This guide covers installing the Virtual Audio Cable library in your Rust project.

## Prerequisites

- Rust 1.70 or later
- Cargo package manager
- For Linux: PipeWire audio daemon
- For Windows: Windows 10/11 (Windows 10 version 1903 or later)

## Adding to Your Project

Add the following to your `Cargo.toml`:

```toml
[dependencies]
virtual_audio = "0.1.0"
```

### Platform-Specific Features

The library uses Cargo features for platform-specific dependencies:

#### Linux
```toml
[dependencies]
virtual_audio = { version = "0.1.0", features = ["linux"] }
```

This enables:
- PipeWire integration
- ALSA support via cpal
- Portal support via ashpd

#### Windows
```toml
[dependencies]
virtual_audio = { version = "0.1.0", features = ["windows"] }
```

This enables:
- WDM/WaveRT driver support
- Windows audio API integration

## System Dependencies

### Linux

On Linux, you need PipeWire installed:

**Fedora:**
```bash
sudo dnf install pipewire pipewire-devel
```

**Ubuntu/Debian:**
```bash
sudo apt install pipewire libpipewire-0.3-dev
```

**Arch Linux:**
```bash
sudo pacman -S pipewire pipewire-devel
```

### Windows

No additional system dependencies are required. The library uses Windows built-in audio APIs.

## Verification

After installation, verify it works:

```rust
use virtual_audio_cable::{CableConfig, VirtualCable, VirtualCableTrait};

fn main() -> anyhow::Result<()> {
    let config = CableConfig::default();
    let cable = VirtualCable::new(config)?;
    println!("Virtual Audio Cable installed successfully!");
    Ok(())
}
```

Run with:
```bash
cargo run
```

## Building from Source

To build the library from source:

```bash
# Clone the repository
git clone https://github.com/nglmercer/virtual_audio.git
cd virtual_audio

# Build the library
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## Development Mode

For development with all features enabled:

```bash
cargo build --all-features
```

## Troubleshooting

### "Failed to connect to PipeWire" (Linux)

Make sure PipeWire is running:
```bash
systemctl --user status pipewire
```

Start it if needed:
```bash
systemctl --user start pipewire
```

### "Audio device not found" (Windows)

Ensure Windows Audio service is running and your audio drivers are up to date.

### Build Errors

If you encounter build errors, try:
```bash
cargo clean
cargo update
cargo build
```

## Next Steps

- [Quick Start Guide](./quick-start.md) - Get started with your first virtual cable
- [API Overview](./api-overview.md) - Learn the main API concepts
- [Examples](../examples) - See working code examples
