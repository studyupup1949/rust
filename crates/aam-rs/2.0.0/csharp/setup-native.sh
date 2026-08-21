#!/bin/bash
# Script to prepare native libraries for local C# development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CSHARP_DIR="$SCRIPT_DIR"

echo "Preparing native libraries for C# development..."

# Build the Rust library
echo "Building Rust library with FFI..."
cd "$PROJECT_ROOT"
cargo build --release --features ffi

# Determine OS and architecture
OS_TYPE="$(uname -s)"
ARCH="$(uname -m)"

case "$OS_TYPE" in
    Linux)
        LIB_FILE="target/release/libaam_rs.so"
        RUNTIME_DIR="$CSHARP_DIR/runtimes/linux-x64/native"
        ;;
    Darwin)
        LIB_FILE="target/release/libaam_rs.dylib"
        if [ "$ARCH" = "arm64" ]; then
            RUNTIME_DIR="$CSHARP_DIR/runtimes/osx-arm64/native"
        else
            RUNTIME_DIR="$CSHARP_DIR/runtimes/osx-x64/native"
        fi
        ;;
    MINGW*|MSYS*|CYGWIN*)
        LIB_FILE="target/release/aam_rs.dll"
        RUNTIME_DIR="$CSHARP_DIR/runtimes/win-x64/native"
        ;;
    *)
        echo "Unsupported OS: $OS_TYPE"
        exit 1
        ;;
esac

# Create runtime directory if it doesn't exist
mkdir -p "$RUNTIME_DIR"

# Copy the library
if [ -f "$LIB_FILE" ]; then
    echo "Copying $LIB_FILE to $RUNTIME_DIR"
    cp "$LIB_FILE" "$RUNTIME_DIR/"
    echo "✓ Native library copied successfully"
else
    echo "✗ Native library not found at $LIB_FILE"
    exit 1
fi

echo ""
echo "Setup complete! You can now run C# tests and examples:"
echo ""
echo "  cd csharp"
echo "  dotnet test"
echo "  dotnet run --project examples/Basic/AamCsharp.Basic.csproj"

