#!/bin/bash
# Setup script for Virtual Audio Cable project
# This script configures the environment and verifies the project

echo "=== Virtual Audio Cable Project Setup ==="
echo ""

# Check if Rust is installed
echo "[1/5] Checking Rust installation..."
if [ -f "$HOME/.cargo/bin/rustc" ]; then
    echo "✅ Rust found at: $HOME/.cargo/bin/rustc"
    RUSTC_VERSION=$($HOME/.cargo/bin/rustc --version 2>/dev/null)
    echo "   Version: $RUSTC_VERSION"
else
    echo "❌ Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi
echo ""

# Add Cargo to PATH
echo "[2/5] Configuring PATH..."
if ! grep -q "$HOME/.cargo/bin" "$HOME/.bashrc" 2>/dev/null; then
    echo "Adding ~/.cargo/bin to PATH..."
    echo "" >> "$HOME/.bashrc"
    echo "# Rust and Cargo" >> "$HOME/.bashrc"
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.bashrc"
    echo "✅ PATH configured in ~/.bashrc"
    echo ""
    echo "Please run: source ~/.bashrc"
    echo "Or restart your terminal"
else
    echo "✅ PATH already configured"
fi
echo ""

# Export PATH for current session
export PATH="$HOME/.cargo/bin:$PATH"

# Verify cargo works
echo "[3/5] Verifying Cargo installation..."
if command -v cargo >/dev/null 2>&1; then
    echo "✅ Cargo is available"
    CARGO_VERSION=$(cargo --version)
    echo "   $CARGO_VERSION"
else
    echo "❌ Cargo not found in PATH"
    echo "   Please run: source ~/.bashrc"
fi
echo ""

# Check for platform dependencies
echo "[4/5] Checking platform dependencies..."
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Linux detected"
    
    if command -v pipewire >/dev/null 2>&1; then
        echo "✅ PipeWire installed"
        PW_VERSION=$(pipewire --version 2>/dev/null || echo "unknown")
        echo "   $PW_VERSION"
    else
        echo "⚠️  PipeWire not found"
        echo "   Install with: sudo apt install pipewire libpipewire-dev"
    fi
    
    if [ -f "/usr/lib/libpipewire.so" ] || [ -f "/usr/lib/x86_64-linux-gnu/libpipewire.so" ]; then
        echo "✅ PipeWire development libraries found"
    else
        echo "⚠️  PipeWire dev libraries not found"
        echo "   Install with: sudo apt install libpipewire-dev"
    fi
elif [[ "$OSTYPE" == "msys" ]]; then
    echo "Windows detected"
    echo "⚠️  Windows support requires WDK installation"
    echo "   See specs.md section 2 for details"
else
    echo "Unknown OS: $OSTYPE"
fi
echo ""

# Verify project structure
echo "[5/5] Verifying project structure..."
if [ -f "Cargo.toml" ]; then
    echo "✅ Cargo.toml found"
else
    echo "❌ Cargo.toml not found"
fi

if [ -d "src" ]; then
    echo "✅ src/ directory found"
else
    echo "❌ src/ directory not found"
fi

if [ -f "specs.md" ]; then
    echo "✅ specs.md found"
else
    echo "❌ specs.md not found"
fi

if [ -f "README.md" ]; then
    echo "✅ README.md found"
else
    echo "❌ README.md not found"
fi
echo ""

# Try to build project
echo "=== Building Project ==="
echo ""
export PATH="$HOME/.cargo/bin:$PATH"

if command -v cargo >/dev/null 2>&1; then
    echo "Running: cargo check"
    cargo check
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ Project checks passed!"
        echo ""
        echo "=== Next Steps ==="
        echo ""
        echo "1. To use cargo in current session:"
        echo "   export PATH=\"$HOME/.cargo/bin:\$PATH\""
        echo ""
        echo "2. Or restart your terminal (recommended)"
        echo ""
        echo "3. Build the project:"
        echo "   cargo build --release"
        echo ""
        echo "4. Run tests:"
        echo "   cargo test"
        echo ""
        echo "5. Run the application:"
        echo "   cargo run --release -- --help"
        echo ""
    else
        echo ""
        echo "❌ Build failed. Check errors above."
    fi
else
    echo "❌ Cargo not available. Please run: source ~/.bashrc"
fi
