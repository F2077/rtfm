# RTFM - Read The F***ing Manual

> A CLI cheatsheet tool inspired by [tldr](https://tldr.sh/).

Rust-based cross-platform offline CLI cheatsheet with TUI interface, full-text search, and multi-language support.

## Features

- **Direct Lookup**: `rtfm tar` - instantly show command usage
- **Learn from System**: `rtfm learn <cmd>` - capture `--help` or `man` pages into the database
- **TUI Interface**: Modern terminal UI built with ratatui
- **Full-text Search**: High-performance search powered by Tantivy with Chinese tokenization (jieba)
- **Multi-language**: Supports Chinese, English, and other languages from tldr-pages
- **Offline First**: Works completely offline
- **Cross-platform**: Windows, Linux, macOS
- **HTTP API**: Optional REST API server mode

## Installation

### From Source

```bash
git clone <repository-url>
cd rtfm

# Build release
cargo build --release

# Run
./target/release/rtfm
```

### Requirements

- Rust 1.75+

## Quick Start

```bash
# First, download cheatsheets from tldr-pages
rtfm update

# Look up a command directly
rtfm tar
rtfm docker
rtfm git

# Look up in specific language
rtfm -l zh tar

# Start interactive TUI
rtfm
```

## Usage

### Direct Command Lookup

```bash
# Look up command (like tldr)
rtfm tar
rtfm docker-compose
rtfm git-rebase

# Specify language
rtfm -l zh curl
rtfm -l en grep
```

### TUI Mode

```bash
# Start TUI (no arguments)
rtfm

# Start with debug log panel
rtfm --debug
```

### Keyboard Shortcuts (TUI)

| Key | Action |
|-----|--------|
| `/` | Focus search box |
| `↑↓` or `jk` | Navigate |
| `Enter` | View details |
| `Tab` | Switch focus |
| `Esc` | Clear search / Back |
| `PgUp/PgDn` | Page up/down |
| `F1` | Toggle help |
| `F12` | Toggle debug log panel |
| `q` | Quit (not in search) |
| `Ctrl+Q/C` | Force quit |

### HTTP Server Mode

```bash
# Start HTTP API server
rtfm serve --port 3030

# Default: 127.0.0.1:3030
```

API Endpoints:
- `GET /api/search?q=<query>&lang=<lang>&limit=<limit>` - Search commands
- `GET /api/command/<name>?lang=<lang>` - Get command details
- `GET /api/stats` - Get statistics

### Data Update

```bash
# Update from tldr-pages
rtfm update

# Force update (ignore version check)
rtfm update --force

# Import from local files
rtfm import --path /path/to/markdown
```

### Learn Commands from System

You can teach it any command installed on your system:

```bash
# Learn from --help output
rtfm learn rustc
rtfm learn cargo
rtfm learn docker

# Force re-learn (overwrite existing)
rtfm learn --force git

# Prefer man page over --help
rtfm learn --man grep

# Then query the learned command
rtfm rustc
```

The `learn` command:
1. Runs `<command> --help` (or `-h`)
2. Falls back to `man <command>` if needed
3. Parses the output to extract description and examples
4. Saves to local database with full-text indexing

This means you can build a personalized cheatsheet database with ANY command on your system!

### Batch Learn from Man Pages (Linux/macOS) or PowerShell Cmdlets (Windows)

Import entire sections of man pages at once:

```bash
# Learn all user commands (section 1)
rtfm learn-all

# Learn system administration commands (section 8)
rtfm learn-all -s 8

# Learn only git-related commands
rtfm learn-all --prefix git

# Limit to first 100 commands
rtfm learn-all --limit 100

# Skip already learned commands
rtfm learn-all --skip-existing
```

Man sections:
- `1` - User commands (default)
- `2` - System calls
- `3` - Library functions
- `5` - File formats
- `8` - System administration

### Backup/Restore Data

Share your learned commands across machines:

```bash
# Backup all data to a portable archive
rtfm backup -o my-commands.tar.gz

# Restore on another machine
rtfm restore my-commands.tar.gz

# Merge with existing data (instead of replacing)
rtfm restore --merge my-commands.tar.gz
```

The backup includes:
- `data.redb` - Command database
- `index/` - Full-text search index
- `config.toml` - Application configuration
- `metadata.json` - Version and stats
- `README.md` - Restore instructions

## Tech Stack

- **TUI**: ratatui + crossterm
- **CLI**: clap
- **Search**: tantivy + jieba-rs
- **Storage**: redb (embedded KV database)
- **HTTP**: axum
- **Logging**: tracing + tracing-appender

## Documentation

- [Product Specification](docs/SPEC.md) - Product requirements and architecture
- [Developer Guide](docs/DEVELOPER_GUIDE.md) - For experienced developers
- [Rust Beginner Guide](docs/RUST_BEGINNER_GUIDE.md) - For Rust newcomers

## Data Directory

Data is stored at:

- **Windows**: `%LOCALAPPDATA%\rtfm\`
- **Linux**: `~/.local/share/rtfm/`
- **macOS**: `~/Library/Application Support/rtfm/`

Structure:
```
rtfm/
├── data.redb        # Command database
├── index/           # Search index
└── logs/
    └── rtfm.log     # Rolling log file
```

## Project Structure

```
rtfm/
├── src/
│   ├── main.rs      # Entry point
│   ├── cli/         # CLI argument parsing
│   ├── tui/         # TUI interface
│   ├── api/         # HTTP API server
│   ├── storage/     # Database (redb)
│   ├── search/      # Search engine (tantivy)
│   ├── learn/       # Learn from --help/man
│   └── update/      # Data update module
├── Cargo.toml
└── rustfmt.toml
```

## Development

```bash
cargo run           # Dev mode
cargo test          # Run tests
cargo clippy        # Lint
cargo fmt           # Format
```

## Cross-Platform Build

### Using `cross` (Recommended for cross-compilation)

[cross](https://github.com/cross-rs/cross) uses Docker to provide consistent build environments.

**Prerequisites**: Docker Desktop must be installed and running.

```bash
# Build for Linux (cross will be auto-installed if needed)
just build-linux

# Build static Linux binary (musl)
just build-linux-musl

# Or manually install cross first (optional)
just install-cross
```

The build output will be in `target/x86_64-unknown-linux-gnu/release/rtfm`.

### Manual Build

```bash
# Install targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl

# Build (native environment only)
cargo build --release --target x86_64-unknown-linux-gnu
```

## License

MIT
