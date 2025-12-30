# Build Issues and Solutions

## Issue: libclang Not Found in Flatpak Environment

### Problem Description
When building the project with default features (Linux/PipeWire support) in a Flatpak environment, you may encounter the following error:

```
error: failed to run custom build command for `libspa-sys v0.8.0`
...
Unable to find libclang: "couldn't find any valid shared libraries matching: ['libclang.so', ...]"
```

This occurs because:
1. The `pipewire` crate (and its dependency `libspa-sys`) uses `bindgen` to generate FFI bindings
2. `bindgen` requires `libclang` to be available on the system
3. Flatpak sandboxes don't include `libclang` in their runtime

### Solutions

#### Solution 1: Build Outside Flatpak (Recommended)

The simplest solution is to build the project outside of the Flatpak sandbox:

```bash
# Exit the Flatpak environment
# Then build normally
cargo build
```

This will use the system's installed `libclang` and `libLLVM` libraries.

#### Solution 2: Install LLVM/Clang Development Package

If you need to build within the Flatpak environment, you must install the LLVM/Clang development package on the host system:

```bash
# On Fedora/RHEL systems
sudo dnf install clang-devel llvm-devel

# On Ubuntu/Debian systems
sudo apt install libclang-dev libllvm-dev
```

Then build with the environment variable pointing to the host libraries:

```bash
LIBCLANG_PATH=/var/run/host/usr/lib64 LD_LIBRARY_PATH=/var/run/host/usr/lib64:/var/run/host/usr/lib cargo build
```

**Note:** This may fail due to ABI incompatibilities between the Flatpak runtime and host libraries.

#### Solution 3: Build Without Default Features

You can build the library without Linux/PipeWire features:

```bash
# Build only the library core
cargo build --lib --no-default-features

# Or build with Windows features only (if on Windows)
cargo build --no-default-features --features windows
```

This will skip the `pipewire` dependencies that require `libclang`.

#### Solution 4: Use the Rust LLVM Bindgen Pre-built Bindings

Some crates provide pre-generated bindings. You can try disabling bindgen at runtime:

```bash
cargo build --features linux LIBSPA_SYS_NO_BINDGEN=1
```

However, this depends on whether the specific crate supports this option.

### Current Status

The project has been fixed to build successfully without default features:

```bash
cargo build --lib --no-default-features
# ✅ Success - library builds correctly
```

### Code Changes Made

The following compilation errors were fixed:

1. **Added `Default` trait bound to `RingBuffer<T>`** (`src/buffer.rs`)
   - Changed `impl<T: Clone + Copy>` to `impl<T: Clone + Copy + Default>`

2. **Made `RingBuffer::write` method accept mutable self** (`src/buffer.rs`)
   - Changed `pub fn write(&self, ...)` to `pub fn write(&mut self, ...)`

3. **Updated `TripleRingBuffer` to use `Mutex` for thread safety** (`src/platform/linux.rs`)
   - Changed `Arc<TripleRingBuffer>` to `Arc<Mutex<TripleRingBuffer>>`
   - Updated all methods to lock the mutex before accessing the buffer

4. **Fixed `AudioFormat` re-export issue** (`src/lib.rs`)
   - Removed incorrect re-export from audio module
   - `AudioFormat` is now only defined in `lib.rs`

### Recommended Development Workflow

For development on Linux:

1. **Outside Flatpak** (preferred):
   ```bash
   cargo build
   cargo test
   cargo run --bin virtual-audio-cable
   ```

2. **Inside Flatpak** (if necessary):
   ```bash
   # Build core library only
   cargo build --lib --no-default-features
   
   # Run tests
   cargo test --lib --no-default-features
   ```

### Testing

To verify the build works:

```bash
# Test library build (no features)
cargo build --lib --no-default-features

# Run library tests
cargo test --lib --no-default-features

# Check if binary can be built (requires tokio feature)
cargo build --no-default-features --bin virtual-audio-cable
# This will fail - binary needs tokio which is optional
```

### Future Improvements

To make the project more Flatpak-friendly, consider:

1. Adding conditional compilation to disable bindgen in sandboxed environments
2. Providing pre-generated bindings as a fallback
3. Using a crate that doesn't depend on bindgen for PipeWire bindings
4. Creating a dedicated Flatpak manifest that includes LLVM/Clang extensions

### Related Issues

- Flatpak sandbox limitations: https://docs.flatpak.org/en/latest/sandbox-permissions.html
- bindgen requirements: https://rust-lang.github.io/rust-bindgen/requirements.html
- pipewire-sys crate documentation: https://docs.rs/pipewire-sys/latest/
