# Contributing to virtual_audio

Thank you for your interest in contributing to `virtual_audio`! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code.

Please report unacceptable behavior to [your email or issue tracker].

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- For Linux: PulseAudio or PipeWire installed
- For Windows development: WDK (Windows Driver Kit)

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/nglmercer/virtual_audio.git
cd virtual_audio

# Install dependencies
cargo build

# Run tests
cargo test

# Run with features
cargo test --all-features
```

### Platform-Specific Setup

#### Linux
```bash
# Install PulseAudio/PipeWire tools
sudo apt-get install pulseaudio-utils pipewire libpipewire-0.3-dev

# Start PulseAudio (if not running)
pulseaudio --start
```

#### Windows
```bash
# Install WDK from Microsoft
# Download from: https://docs.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk

# Install WDK build tools
# Follow WDK documentation for Rust integration
```

## Development Workflow

1. **Fork and Branch**
   - Fork the repository
   - Create a feature branch: `git checkout -b feature/my-feature`

2. **Make Changes**
   - Write code following [Coding Standards](#coding-standards)
   - Add tests following [Testing guidelines](#testing)
   - Update documentation following [Documentation guidelines](#documentation)

3. **Test**
   ```bash
   # Run all tests
   cargo test --all-features
   
   # Run with coverage
   cargo tarpaulin --out Html
   
   # Run clippy
   cargo clippy -- -D warnings
   
   # Format code
   cargo fmt
   ```

4. **Commit**
   ```bash
   git add .
   git commit -m "feat: add my feature"
   ```

5. **Push and PR**
   ```bash
   git push origin feature/my-feature
   # Create Pull Request on GitHub
   ```

## Coding Standards

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy -- -D warnings`
- Prefer `Result<T, E>` over `panic!` for recoverable errors
- Use `thiserror` for custom error types
- Document all public APIs

### Naming Conventions

- Types: `PascalCase`
- Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Example

```rust
/// Creates a new audio processor.
///
/// # Arguments
///
/// * `input_sample_rate` - Input sample rate in Hz
/// * `output_sample_rate` - Output sample rate in Hz
///
/// # Examples
///
/// ```rust
/// use virtual_audio_cable::AudioProcessor;
///
/// let processor = AudioProcessor::new(44100, 48000, 2, AudioFormat::F32LE);
/// ```
///
/// # Errors
///
/// Returns `Error::AudioError` if sample rates are invalid.
pub fn new(input_sample_rate: u32, output_sample_rate: u32, channels: u16, format: AudioFormat) -> Self {
    // Implementation
}
```

## Testing

### Unit Tests

Write unit tests for all public functions and types:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = /* ... */;
        
        // Act
        let result = function_to_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Integration Tests

Add integration tests in the `tests/` directory:

```rust
// tests/integration_test.rs
use virtual_audio_cable::*;

#[test]
fn test_integration_scenario() {
    // Test complete workflows
}
```

### Running Tests

```bash
# All tests
cargo test --all-features

# Specific test
cargo test test_function_name

# Doc tests
cargo test --doc

# With output
cargo test -- --nocapture
```

### Benchmark Tests

Add benchmarks using `criterion`:

```rust
// benches/audio_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_ring_buffer(c: &mut Criterion) {
    c.bench_function("ring_buffer_write", |b| {
        b.iter(|| {
            // Benchmark code
        });
    });
}

criterion_group!(benches, benchmark_ring_buffer);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench
```

## Documentation

### Documentation Requirements

- All public APIs must have documentation
- Use `///` for item documentation
- Use `//!` for module documentation
- Include examples where appropriate
- Document errors and panics
- Document platform-specific behavior

### Documentation Format

```rust
/// Brief description (one sentence).
///
/// Extended description with more details.
///
/// # Arguments
///
/// * `arg1` - Description of argument
///
/// # Returns
///
/// Description of return value.
///
/// # Examples
///
/// ```rust
/// let result = function_call();
/// ```
///
/// # Errors
///
/// Returns `Error::SpecificError` when...
///
/// # Panics
///
/// Panics if...
pub fn function(arg1: Type) -> Result<ReturnType, Error> {
    // Implementation
}
```

### Building Documentation

```bash
# Generate documentation
cargo doc --no-deps --all-features

# Open in browser
cargo doc --open

# Test documentation examples
cargo test --doc
```

## Pull Request Process

### Before Submitting

- [ ] Code follows [coding standards](#coding-standards)
- [ ] All tests pass: `cargo test --all-features`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt -- --check`
- [ ] Documentation is complete: `cargo doc --no-deps`
- [ ] Added tests for new functionality
- [ ] Updated CHANGELOG.md (if applicable)
- [ ] Updated documentation (if applicable)

### PR Description

Use the following PR title format:

```
<type>: <subject>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

Example PR description:

```markdown
## Summary
Brief description of changes.

## Changes
- Add new feature X
- Fix bug Y
- Update documentation

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing performed

## Checklist
- [ ] Follows coding standards
- [ ] Tests added/updated
- [ ] Documentation updated
```

### Review Process

1. Automated checks must pass (CI/CD)
2. At least one maintainer approval required
3. Address all review comments
4. Squash commits if needed
5. Rebase onto target branch if needed

### Merging

Maintainers will:
1. Ensure all checks pass
2. Verify documentation
3. Test changes
4. Merge using "Squash and merge" or "Rebase and merge"

## Release Process

Releases are managed by maintainers following Semantic Versioning.

### Version Bump

- **MAJOR**: Breaking changes
- **MINOR**: New features, backwards compatible
- **PATCH**: Bug fixes, backwards compatible

### Release Checklist

- [ ] All tests pass
- [ ] Documentation is complete
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Git tag created: `git tag v0.x.x`
- [ ] Published to crates.io: `cargo publish`
- [ ] GitHub release created
- [ ] Announcement published

### Release Automation

The CI/CD workflow includes automated steps for:
- Running tests on all platforms
- Verifying documentation
- Publishing dry-run

Maintainers should:
1. Run `cargo publish --dry-run` locally
2. Verify package contents: `cargo package --list`
3. Publish: `cargo publish`
4. Create GitHub release with CHANGELOG content

## Questions?

- Open an issue for questions
- Check existing issues for similar discussions
- Contact maintainers directly for security issues

## Recognition

Contributors will be acknowledged in:
- CHANGELOG.md
- GitHub contributors list
- Release notes

Thank you for contributing!
