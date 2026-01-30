# Installation

## From crates.io (Recommended)

The easiest way to install RTFM:

```bash
cargo install rtfm
```

This will download, compile, and install the latest version to `~/.cargo/bin/`.

## Pre-built Binaries

Download pre-compiled binaries from [GitHub Releases](https://github.com/F2077/rtfm/releases):

| Platform | Architecture | File |
|----------|--------------|------|
| Linux | x86_64 (glibc) | `rtfm-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | x86_64 (musl/static) | `rtfm-x86_64-unknown-linux-musl.tar.gz` |
| Linux | ARM64 (glibc) | `rtfm-aarch64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 (musl/static) | `rtfm-aarch64-unknown-linux-musl.tar.gz` |
| macOS | Intel | `rtfm-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `rtfm-aarch64-apple-darwin.tar.gz` |
| macOS | Universal | `rtfm-universal-apple-darwin.tar.gz` |
| Windows | x86_64 | `rtfm-x86_64-pc-windows-msvc.zip` |
| Windows | ARM64 | `rtfm-aarch64-pc-windows-msvc.zip` |

Extract and add to your PATH:

```bash
# Linux/macOS
tar -xzf rtfm-*.tar.gz
sudo mv rtfm /usr/local/bin/

# Windows (PowerShell)
Expand-Archive rtfm-*.zip -DestinationPath .
# Move rtfm.exe to a directory in your PATH
```

## From Source

Build from source for the latest features:

```bash
# Clone the repository
git clone https://github.com/F2077/rtfm.git
cd rtfm

# Build release version
cargo build --release

# The binary will be at target/release/rtfm
```

## Requirements

- **Rust 1.75+** - Install from [rustup.rs](https://rustup.rs/)
- **C compiler** - Required for some dependencies
  - Windows: Install Visual Studio Build Tools
  - macOS: `xcode-select --install`
  - Linux: `apt install build-essential` or equivalent

## Post-installation

After installation, download the cheatsheet database:

```bash
# Download tldr-pages cheatsheets
rtfm update
```

This will:
1. Download the latest tldr-pages release from GitHub
2. Parse and store commands in local database
3. Build the full-text search index

## Verifying Installation

```bash
# Check version
rtfm --version

# Should output: rtfm 0.1.0

# Test a search
rtfm tar
```

## Data Location

RTFM stores its data in platform-specific directories:

| Platform | Location |
|----------|----------|
| Linux    | `~/.local/share/rtfm/` |
| macOS    | `~/Library/Application Support/rtfm/` |
| Windows  | `%APPDATA%\rtfm\` |

Contents:
- `data.redb` - Command database
- `index/` - Full-text search index
- `logs/` - Application logs
- `config.toml` - Local configuration (optional)
