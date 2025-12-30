# Changelog

All notable changes to the `virtual_audio` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Integration with rubato for high-quality resampling
- Support for CPAL for audio capture/playback
- Improved routing API
- Audio effects (gain, EQ, etc.)
- Complete Windows WDM/WaveRT driver implementation

## [0.1.0] - 2024-XX-XX

### Added
- Initial release of virtual_audio
- Cross-platform virtual audio cable implementation
- **Linux Support:**
  - Full implementation using PulseAudio/PipeWire
  - Dynamic audio routing per application
  - System-wide audio routing
  - Application listing and discovery
  - Output duplication capabilities
- **Windows Support:**
  - Placeholder implementation for WDM/WaveRT driver
  - Structured for future kernel-mode driver
- **Core Features:**
  - Lock-free ring buffers for audio data
  - Triple ring buffer architecture
  - Audio format conversion (F32, S16, S24, S32)
  - Basic resampling with linear interpolation
  - Real-time safe audio processing
- **Configuration:**
  - Flexible `CableConfig` for cable parameters
  - Support for multiple sample rates
  - Configurable buffer sizes
  - Multiple audio formats
- **Error Handling:**
  - Comprehensive error types
  - Platform-specific error messages
  - Error propagation through `anyhow::Result`

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Security
- N/A (initial release)

## Known Issues

### v0.1.0
- **S24LE Format:** Known sign-extension bug in `bytes_to_samples()` method. Converting from S24LE to f32 may produce incorrect values for negative samples. Workaround: Use F32LE, S16LE, or S32LE formats.
- **Windows Driver:** The Windows implementation is a placeholder. Actual driver functionality requires WDK and kernel-mode development.
- **Linux Dependencies:** Requires `pactl` to be installed and PulseAudio/PipeWire running.
- **Resampling Quality:** Uses linear interpolation. For high-quality resampling, integrate `rubato` library (planned for v0.2.0).

## Migration Guide

### From 0.1.0 to Future Versions

When upgrading to future versions, be aware of:

**v0.2.0 (Planned):**
- Resampling API will change when `rubato` is integrated
- New `AudioProcessor` methods for high-quality conversion
- Possible breaking changes in resampler interface

**v1.0.0 (Planned):**
- Windows API will change when driver is fully implemented
- Additional audio effect methods may be added
- Configuration structure may be enhanced

## Contributors

- Virtual Audio Cable Team

## License

This project is licensed under MIT OR Apache-2.0. See LICENSE-MIT and LICENSE-APACHE for details.
