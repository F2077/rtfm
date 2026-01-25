# RTFM - Cross-platform Build Tasks
# Install just: cargo install just
#
# Cross-compilation notes:
# - Requires Docker or Podman to run cross tool
# - cross automatically downloads the appropriate toolchain images
# - macOS targets can only be built on macOS (Apple restriction)
# - Windows MSVC targets can only be built on Windows
# - Windows GNU targets can be cross-compiled via cross on other platforms
#
# Using Podman instead of Docker:
#   export CROSS_CONTAINER_ENGINE=podman
#   # Or add to ~/.bashrc / ~/.zshrc for permanent config

# Default: show help
default:
    @just --list

# ============ Development ============

# Debug build
build:
    cargo build

# Release build
release:
    cargo build --release

# Run (development mode)
run *args:
    cargo run -- {{args}}

# Run tests
test:
    cargo test

# Run benchmarks
bench:
    cargo bench

# Lint check
check:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Format check
fmt-check:
    cargo fmt -- --check

# Clean build artifacts
clean:
    cargo clean

# Full CI check
ci: fmt-check check test

# ============ Local Build (current platform) ============

# Executable extension
exe := if os() == "windows" { ".exe" } else { "" }

# Local release build to dist/
dist: release
    @echo "Creating distribution..."
    @mkdir -p dist
    @cp target/release/rtfm{{exe}} dist/
    @echo "Done: dist/rtfm{{exe}}"

# ============ Cross-compilation Toolchain ============

# Install cross tool (requires Docker or Podman)
install-cross:
    @echo "Installing cross..."
    @echo "Note: Requires Docker or Podman"
    @echo "For Podman: export CROSS_CONTAINER_ENGINE=podman"
    cargo install cross --git https://github.com/cross-rs/cross
    @echo "cross installed successfully!"

# Check if cross is available
check-cross:
    @cross --version
    @echo "Container engine: $${CROSS_CONTAINER_ENGINE:-docker}"

# Install all Rust targets (without cross)
install-targets:
    rustup target add x86_64-unknown-linux-gnu
    rustup target add x86_64-unknown-linux-musl
    rustup target add aarch64-unknown-linux-gnu
    rustup target add aarch64-unknown-linux-musl
    rustup target add x86_64-pc-windows-msvc
    rustup target add x86_64-pc-windows-gnu
    rustup target add aarch64-pc-windows-msvc
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin

# ============ Linux Targets ============

# Linux x86_64 (glibc) - via cross
build-linux-x64:
    cross build --release --target x86_64-unknown-linux-gnu

# Linux x86_64 (musl/static) - via cross
build-linux-x64-musl:
    cross build --release --target x86_64-unknown-linux-musl

# Linux ARM64 (glibc) - via cross
build-linux-arm64:
    cross build --release --target aarch64-unknown-linux-gnu

# Linux ARM64 (musl/static) - via cross
build-linux-arm64-musl:
    cross build --release --target aarch64-unknown-linux-musl

# Linux native build (current architecture)
build-linux-native:
    cargo build --release

# ============ Windows Targets ============

# Windows x86_64 (MSVC) - Windows only
[windows]
build-windows-x64-msvc:
    cargo build --release --target x86_64-pc-windows-msvc

# Windows x86_64 (GNU) - cross-compilable via cross
build-windows-x64-gnu:
    cross build --release --target x86_64-pc-windows-gnu

# Windows ARM64 (MSVC) - Windows only
[windows]
build-windows-arm64-msvc:
    cargo build --release --target aarch64-pc-windows-msvc

# Windows native build
[windows]
build-windows-native:
    cargo build --release

# ============ macOS Targets ============
# Note: macOS targets can only be built on macOS (Apple licensing restriction)

# macOS x86_64 (Intel)
[macos]
build-macos-x64:
    cargo build --release --target x86_64-apple-darwin

# macOS ARM64 (Apple Silicon)
[macos]
build-macos-arm64:
    cargo build --release --target aarch64-apple-darwin

# macOS Universal Binary (x64 + ARM64)
[macos]
build-macos-universal: build-macos-x64 build-macos-arm64
    @mkdir -p target/universal-apple-darwin/release
    lipo -create \
        target/x86_64-apple-darwin/release/rtfm \
        target/aarch64-apple-darwin/release/rtfm \
        -output target/universal-apple-darwin/release/rtfm
    @echo "Created universal binary: target/universal-apple-darwin/release/rtfm"

# macOS native build
[macos]
build-macos-native:
    cargo build --release

# ============ Build All Targets ============

# Build all Linux targets
build-all-linux: build-linux-x64 build-linux-x64-musl build-linux-arm64 build-linux-arm64-musl
    @echo "All Linux targets built!"

# Build all cross-compilable targets (excluding macOS)
build-all-cross: build-all-linux build-windows-x64-gnu
    @echo "All cross-compilable targets built!"

# [macOS] Build all macOS + cross-compilable targets
[macos]
build-all: build-macos-universal build-all-cross
    @echo "All targets built!"

# [Linux] Build all cross-compilable targets
[linux]
build-all: build-all-cross
    @echo "All targets built!"
    @echo "Note: macOS targets can only be built on macOS."

# [Windows] Build Windows + cross-compilable targets
[windows]
build-all: build-windows-x64-msvc build-windows-arm64-msvc build-all-linux
    @echo "All targets built!"
    @echo "Note: macOS targets can only be built on macOS."
    @echo "Note: Windows GNU targets built with cross."

# ============ Packaging ============

# Distribution directory
dist_dir := "dist"

# Files to include in release packages
release_files := "README.md LICENSE dist/QUICK_START.md dist/config.example.toml"

# Create dist directory
_mkdir-dist:
    @mkdir -p {{dist_dir}}

# Package Linux x64
package-linux-x64: build-linux-x64 _mkdir-dist
    @mkdir -p {{dist_dir}}/staging
    @cp target/x86_64-unknown-linux-gnu/release/rtfm {{dist_dir}}/staging/
    @cp {{release_files}} {{dist_dir}}/staging/
    @tar -czvf {{dist_dir}}/rtfm-x86_64-unknown-linux-gnu.tar.gz -C {{dist_dir}}/staging .
    @rm -rf {{dist_dir}}/staging
    @echo "Created: {{dist_dir}}/rtfm-x86_64-unknown-linux-gnu.tar.gz"

# Package Linux x64 (musl)
package-linux-x64-musl: build-linux-x64-musl _mkdir-dist
    @mkdir -p {{dist_dir}}/staging
    @cp target/x86_64-unknown-linux-musl/release/rtfm {{dist_dir}}/staging/
    @cp {{release_files}} {{dist_dir}}/staging/
    @tar -czvf {{dist_dir}}/rtfm-x86_64-unknown-linux-musl.tar.gz -C {{dist_dir}}/staging .
    @rm -rf {{dist_dir}}/staging
    @echo "Created: {{dist_dir}}/rtfm-x86_64-unknown-linux-musl.tar.gz"

# Package Linux ARM64
package-linux-arm64: build-linux-arm64 _mkdir-dist
    @mkdir -p {{dist_dir}}/staging
    @cp target/aarch64-unknown-linux-gnu/release/rtfm {{dist_dir}}/staging/
    @cp {{release_files}} {{dist_dir}}/staging/
    @tar -czvf {{dist_dir}}/rtfm-aarch64-unknown-linux-gnu.tar.gz -C {{dist_dir}}/staging .
    @rm -rf {{dist_dir}}/staging
    @echo "Created: {{dist_dir}}/rtfm-aarch64-unknown-linux-gnu.tar.gz"

# Package Linux ARM64 (musl)
package-linux-arm64-musl: build-linux-arm64-musl _mkdir-dist
    @mkdir -p {{dist_dir}}/staging
    @cp target/aarch64-unknown-linux-musl/release/rtfm {{dist_dir}}/staging/
    @cp {{release_files}} {{dist_dir}}/staging/
    @tar -czvf {{dist_dir}}/rtfm-aarch64-unknown-linux-musl.tar.gz -C {{dist_dir}}/staging .
    @rm -rf {{dist_dir}}/staging
    @echo "Created: {{dist_dir}}/rtfm-aarch64-unknown-linux-musl.tar.gz"

# Package Windows x64 (GNU)
package-windows-x64-gnu: build-windows-x64-gnu _mkdir-dist
    @mkdir -p {{dist_dir}}/staging
    @cp target/x86_64-pc-windows-gnu/release/rtfm.exe {{dist_dir}}/staging/
    @cp {{release_files}} {{dist_dir}}/staging/
    @cd {{dist_dir}}/staging && zip -r ../rtfm-x86_64-pc-windows-gnu.zip .
    @rm -rf {{dist_dir}}/staging
    @echo "Created: {{dist_dir}}/rtfm-x86_64-pc-windows-gnu.zip"

# [Windows] Package Windows x64 (MSVC)
[windows]
package-windows-x64-msvc: build-windows-x64-msvc _mkdir-dist
    @mkdir -p {{dist_dir}}/staging
    @cp target/x86_64-pc-windows-msvc/release/rtfm.exe {{dist_dir}}/staging/
    @cp {{release_files}} {{dist_dir}}/staging/
    @cd {{dist_dir}}/staging && 7z a ../rtfm-x86_64-pc-windows-msvc.zip .
    @rm -rf {{dist_dir}}/staging
    @echo "Created: {{dist_dir}}/rtfm-x86_64-pc-windows-msvc.zip"

# [macOS] Package macOS Universal
[macos]
package-macos-universal: build-macos-universal _mkdir-dist
    @mkdir -p {{dist_dir}}/staging
    @cp target/universal-apple-darwin/release/rtfm {{dist_dir}}/staging/
    @cp {{release_files}} {{dist_dir}}/staging/
    @tar -czvf {{dist_dir}}/rtfm-universal-apple-darwin.tar.gz -C {{dist_dir}}/staging .
    @rm -rf {{dist_dir}}/staging
    @echo "Created: {{dist_dir}}/rtfm-universal-apple-darwin.tar.gz"

# [macOS] Package macOS x64
[macos]
package-macos-x64: build-macos-x64 _mkdir-dist
    @mkdir -p {{dist_dir}}/staging
    @cp target/x86_64-apple-darwin/release/rtfm {{dist_dir}}/staging/
    @cp {{release_files}} {{dist_dir}}/staging/
    @tar -czvf {{dist_dir}}/rtfm-x86_64-apple-darwin.tar.gz -C {{dist_dir}}/staging .
    @rm -rf {{dist_dir}}/staging
    @echo "Created: {{dist_dir}}/rtfm-x86_64-apple-darwin.tar.gz"

# [macOS] Package macOS ARM64
[macos]
package-macos-arm64: build-macos-arm64 _mkdir-dist
    @mkdir -p {{dist_dir}}/staging
    @cp target/aarch64-apple-darwin/release/rtfm {{dist_dir}}/staging/
    @cp {{release_files}} {{dist_dir}}/staging/
    @tar -czvf {{dist_dir}}/rtfm-aarch64-apple-darwin.tar.gz -C {{dist_dir}}/staging .
    @rm -rf {{dist_dir}}/staging
    @echo "Created: {{dist_dir}}/rtfm-aarch64-apple-darwin.tar.gz"

# Package all cross-compilable targets
package-all-cross: package-linux-x64 package-linux-x64-musl package-linux-arm64 package-linux-arm64-musl package-windows-x64-gnu
    @echo "All cross-platform packages created in {{dist_dir}}/"

# [macOS] Package all targets
[macos]
package-all: package-all-cross package-macos-universal
    @echo "All packages created in {{dist_dir}}/"

# [Linux] Package all buildable targets
[linux]
package-all: package-all-cross
    @echo "All packages created in {{dist_dir}}/"

# [Windows] Package all buildable targets
[windows]
package-all: package-all-cross package-windows-x64-msvc
    @echo "All packages created in {{dist_dir}}/"

# ============ Utilities ============

# Update dependencies
update:
    cargo update

# Show dependency tree
deps:
    cargo tree

# Show binary size
size: release
    @ls -lh target/release/rtfm{{exe}}

# [Unix] Install to system
[unix]
install: release
    @cp target/release/rtfm /usr/local/bin/
    @echo "Installed to /usr/local/bin/rtfm"

# Download command data
fetch-data:
    cargo run --release -- update

# Show cross-compilation support matrix
cross-matrix:
    @echo "=== Cross Compilation Support Matrix ==="
    @echo ""
    @echo "From Linux:"
    @echo "  -> Linux (x64, arm64, musl): OK (cross)"
    @echo "  -> Windows (GNU):            OK (cross)"
    @echo "  -> Windows (MSVC):           NO (requires Windows)"
    @echo "  -> macOS:                    NO (requires macOS)"
    @echo ""
    @echo "From macOS:"
    @echo "  -> macOS (x64, arm64):       OK (cargo native)"
    @echo "  -> Linux (x64, arm64, musl): OK (cross)"
    @echo "  -> Windows (GNU):            OK (cross)"
    @echo "  -> Windows (MSVC):           NO (requires Windows)"
    @echo ""
    @echo "From Windows:"
    @echo "  -> Windows (MSVC):           OK (cargo native)"
    @echo "  -> Linux (via cross):        OK (cross + Docker/Podman)"
    @echo "  -> Windows (GNU):            OK (cross)"
    @echo "  -> macOS:                    NO (requires macOS)"
    @echo ""
    @echo "Prerequisites:"
    @echo "  - Docker or Podman (for cross)"
    @echo "  - For Podman: export CROSS_CONTAINER_ENGINE=podman"
    @echo "  - just install-cross"
    @echo "  - just install-targets"

# Show help
help:
    @echo "RTFM Build System"
    @echo ""
    @echo "Quick Start:"
    @echo "  just build          # Debug build"
    @echo "  just release        # Release build"
    @echo "  just test           # Run tests"
    @echo "  just run tar        # Run with args"
    @echo ""
    @echo "Cross Compilation:"
    @echo "  just install-cross  # Install cross tool"
    @echo "  just cross-matrix   # Show support matrix"
    @echo "  just build-all      # Build all targets"
    @echo "  just package-all    # Package all targets"
    @echo ""
    @echo "Container Engine (for cross):"
    @echo "  Docker: default, no config needed"
    @echo "  Podman: export CROSS_CONTAINER_ENGINE=podman"
    @echo ""
    @echo "Individual Targets:"
    @echo "  just build-linux-x64       # Linux x86_64 (glibc)"
    @echo "  just build-linux-x64-musl  # Linux x86_64 (static)"
    @echo "  just build-linux-arm64     # Linux ARM64"
    @echo "  just build-windows-x64-gnu # Windows x64 (cross)"
    @echo "  just build-macos-universal # macOS Universal (macOS only)"
    @echo ""
    @echo "Run 'just --list' for full command list."
