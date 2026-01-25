<h1 align="center">
  <br>
  <code>RTFM</code>
  <br>
  <sub>Read The F***ing Manual</sub>
</h1>

<p align="center">
  <b>A cross-platform CLI cheatsheet with TUI, full-text search, and system learning</b>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#usage">Usage</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#tech-stack">Tech Stack</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.75+-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-blue?style=flat-square" alt="Platform">
  <img src="https://img.shields.io/badge/license-GPL--3.0-green?style=flat-square" alt="License">
</p>

---

```
┌──────────────────────────────────────────────────────────────────────────────┐
│      _________                  "When all else fails..."                     │
│     /        /|                                                              │
│    /  RTFM  / |                   READ THE F***ING MANUAL                    │
│   /________/  |   < Go on, I DARE you to ask again!                          │
│   |  ~~~~  |  |                                                              │
│   | MANUAL |  /                   Rust-powered CLI Cheatsheet                │
│   |________|/                                                                │
├────────────────────────────────────────────┬─────────────────────────────────┤
│  Search (/ to focus)                       │ [Tab] Switch [^H] Help [Esc] Quit│
│ > docker_                                  │                                 │
├────────────────────────────────────────────┼─────────────────────────────────┤
│  Commands                                  │  docker                         │
│ ┌────────────────────────────────────────┐ │ ─────────────────────────────── │
│ │ ► docker                         [en]  │ │  Manage Docker containers and   │
│ │   docker-compose                 [en]  │ │  images.                        │
│ │   docker-build                   [en]  │ │                                 │
│ │   docker-run                     [en]  │ │  - Start a container:           │
│ │   docker-ps                      [en]  │ │    docker run {{image}}         │
│ │   docker-exec                    [en]  │ │                                 │
│ │   docker-logs                    [zh]  │ │  - List running containers:     │
│ │   docker-pull                    [zh]  │ │    docker ps                    │
│ │                                        │ │                                 │
│ └────────────────────────────────────────┘ │  - Stop a container:            │
│  8 results | Page 1/1                      │    docker stop {{container}}    │
├────────────────────────────────────────────┴─────────────────────────────────┤
│ RTFM v0.1.0 | 3,421 commands | Lang: en | ↑↓/jk: Navigate  Enter: Select     │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Why RTFM?

> *"I've already searched Stack Overflow, read three blog posts, and watched a YouTube video... but I still can't remember how to use `tar`."*

### In the Age of AI Agents

You might ask: *"Why do I need this when I can just ask ChatGPT/Copilot/Claude?"*

Great question. Here's the thing: **AI agents need internet access**. When you're:

- Working in an air-gapped environment (security, compliance, or military)
- On a plane, train, or remote location with no connectivity
- Behind a corporate firewall that blocks AI services
- In a data center with restricted network access
- Simply experiencing an internet outage

...your AI assistant can't help you. But **RTFM works 100% offline**.

Built entirely in **Rust**, RTFM is a single binary with no runtime dependencies. Copy it to any machine and it just works.

### Key Benefits

- **Instant lookup** - `rtfm tar` gets you what you need
- **Learn from YOUR system** - Capture `--help` output from any command
- **Works offline** - All data stored locally, no internet needed
- **Beautiful TUI** - Not just functional, but actually pleasant to use
- **Single binary** - Written in Rust, zero runtime dependencies

## Features

| Feature | Description |
|---------|-------------|
| **Direct Lookup** | `rtfm tar` - instantly show command usage |
| **Learn from System** | `rtfm learn cargo` - capture any `--help` or `man` page |
| **TUI Interface** | Modern terminal UI with vim-style navigation |
| **Full-text Search** | Powered by [Tantivy](https://github.com/quickwit-oss/tantivy) with Chinese tokenization |
| **Multi-language** | English, Chinese, and other languages from tldr-pages |
| **Offline First** | Everything runs locally - your data, your machine |
| **HTTP API** | Optional REST API server for integrations |
| **Backup/Restore** | Export and share your personalized cheatsheets |

## Installation

### From Source

```bash
git clone https://github.com/F2077/rtfm.git
cd rtfm
cargo build --release

# Add to PATH or run directly
./target/release/rtfm
```

### Requirements

- Rust 1.75+

## Quick Start

```bash
# 1. Download cheatsheets from tldr-pages
rtfm update

# 2. Look up commands directly
rtfm tar
rtfm git
rtfm docker

# 3. Or launch the interactive TUI
rtfm
```

## Usage

### Direct Command Lookup

```bash
rtfm tar              # Look up tar
rtfm docker-compose   # Hyphenated commands work too
rtfm -l zh curl       # Specify language (zh = Chinese)
```

### Interactive TUI

```bash
rtfm          # Launch TUI
rtfm --debug  # With debug log panel
```

**Keyboard Shortcuts:**

| Key | Action |
|-----|--------|
| `/` | Focus search box |
| `↑↓` or `jk` | Navigate / scroll |
| `hl` or `←→` | Switch between list and detail |
| `Enter` | Select / confirm |
| `Tab` | Cycle focus (Search → List → Detail) |
| `Esc` | Back / clear search / quit |
| `PgUp/PgDn` | Page up/down |
| `Home/End` | Jump to first/last |
| `g/G` | Jump to first/last (list) / Page up/down (detail) |
| `?` | Toggle help (when not in search) |
| `Ctrl+H` | Toggle help (works everywhere) |
| `Ctrl+L` | Toggle debug logs (requires --debug) |
| `Ctrl+C/Q` | Force quit |

### Learn Commands from Your System

The killer feature - teach RTFM any command installed on your machine:

```bash
# Learn from --help
rtfm learn rustc
rtfm learn cargo
rtfm learn kubectl

# Force re-learn (overwrite existing)
rtfm learn --force git

# Prefer man page over --help
rtfm learn --man grep

# Then query it like any other command
rtfm rustc
```

### Batch Learn

Import entire sections at once:

```bash
# Linux/macOS: Learn all man pages (section 1)
rtfm learn-all

# Learn only git-related commands
rtfm learn-all --prefix git

# Windows: Learn PowerShell cmdlets
rtfm learn-all --source powershell

# Limit to first 100 commands
rtfm learn-all --limit 100 --skip-existing
```

### HTTP Server Mode

```bash
# Start server (default: 127.0.0.1:3030)
rtfm serve

# Custom port and bind address
rtfm serve --port 8080 --bind 0.0.0.0

# Run in background (detached from terminal)
rtfm serve --detach

# Debug mode: logs printed to both file and console
rtfm serve --debug
```

Swagger UI available at: `http://localhost:3030/swagger-ui`

**API Endpoints:**

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/search?q=<query>&lang=<lang>&limit=<n>` | Full-text search |
| GET | `/api/command/{name}?lang=<lang>` | Get command by name |
| GET | `/api/commands?lang=<lang>` | List all commands |
| GET | `/api/metadata` | Database metadata & stats |
| GET | `/api/update/check` | Check for updates |
| POST | `/api/update/download` | Download and apply updates |
| POST | `/api/learn` | Learn a command from system |
| POST | `/api/learn-all` | Batch learn commands |
| GET | `/api/backup/info` | Backup information |
| POST | `/api/import` | Import commands (JSON) |
| POST | `/api/import/file` | Import file (md, zip, tar, tar.gz) |
| POST | `/api/reset` | Factory reset |

### Data Management

```bash
# Update from tldr-pages
rtfm update
rtfm update --force  # Force update

# Import custom cheatsheets (auto-detects format)
rtfm import ./my-commands/        # Directory of .md files
rtfm import ./docker.md           # Single markdown file
rtfm import ./commands.zip        # ZIP archive
rtfm import ./commands.tar.gz     # TAR.GZ archive
rtfm import ./commands.tar        # TAR archive

# Backup your data
rtfm backup -o my-commands.tar.gz

# Restore on another machine
rtfm restore my-commands.tar.gz
rtfm restore --merge backup.tar.gz  # Merge instead of replace

# Factory reset
rtfm reset
```

**Import Format:** Files must follow the [tldr-pages format](https://github.com/tldr-pages/tldr/blob/main/contributing-guides/style-guide.md):

```markdown
# command-name

> Brief description of the command.

- Example description:

`command --option {{arg}}`
```

Files without valid description or examples will be skipped.

## Tech Stack

| Component | Technology |
|-----------|------------|
| TUI | [ratatui](https://github.com/ratatui/ratatui) + crossterm |
| CLI | [clap](https://github.com/clap-rs/clap) |
| Search | [tantivy](https://github.com/quickwit-oss/tantivy) + [jieba-rs](https://github.com/messense/jieba-rs) |
| Storage | [redb](https://github.com/cberner/redb) (embedded KV database) |
| HTTP | [axum](https://github.com/tokio-rs/axum) |
| Logging | [tracing](https://github.com/tokio-rs/tracing) |

## Data Directory

| Platform | Location |
|----------|----------|
| Windows | `%LOCALAPPDATA%\rtfm\` |
| Linux | `~/.local/share/rtfm/` |
| macOS | `~/Library/Application Support/rtfm/` |

```
rtfm/
├── data.redb     # Command database
├── index/        # Search index
└── logs/
    └── rtfm.log.YYYY-MM-DD  # Daily rolling log files
```

## Configuration

RTFM supports optional configuration via TOML files. Configuration is loaded from (in order of priority):

1. `./rtfm.toml` (current directory)
2. `<data_dir>/config.toml`
3. Built-in defaults

You can also set `RTFM_DATA_DIR` environment variable to override the data directory.

### Example Configuration

Create `rtfm.toml` in your working directory or `config.toml` in the data directory:

```toml
[server]
port = 3030
bind = "127.0.0.1"
max_upload_size = 104857600  # 100MB in bytes

[search]
default_limit = 20    # Default search results
max_limit = 100       # Maximum search results
default_lang = "en"

[tui]
poll_timeout_ms = 100
log_buffer_size = 100
scroll_step = 1

[storage]
# data_dir = "/custom/path"  # Override data directory
db_filename = "data.redb"
index_dirname = "index"
log_dirname = "logs"

[logging]
level = "info"
debug_level = "debug,tantivy=info"

[update]
github_api_url = "https://api.github.com/repos/tldr-pages/tldr/releases/latest"
download_url_template = "https://github.com/tldr-pages/tldr/archive/refs/tags/{version}.zip"
fallback_version = "v2.3"
languages = ["en", "zh"]  # Languages to import (empty = all)
```

All fields are optional - unspecified values use built-in defaults.

## Project Structure

```
rtfm/
├── src/
│   ├── main.rs      # Entry point & CLI commands
│   ├── cli/         # Argument parsing
│   ├── tui/         # Terminal UI
│   ├── api/         # HTTP API server
│   ├── storage/     # Database (redb)
│   ├── search/      # Search engine (tantivy)
│   ├── learn/       # Learn from --help/man
│   └── update/      # Data update module
├── docs/
│   ├── SPEC.md              # Product specification
│   ├── DEVELOPER_GUIDE.md   # For experienced devs
│   └── RUST_BEGINNER_GUIDE.md
├── Cargo.toml
└── rustfmt.toml
```

## Development

```bash
cargo run             # Dev mode
cargo test            # Run tests
cargo clippy          # Lint
cargo fmt             # Format

# Cross-compile for Linux (requires Docker)
just build-linux
just build-linux-musl
```

## License

GPL-3.0

---

<p align="center">
  <sub>Built with Rust. Inspired by <a href="https://tldr.sh">tldr</a>. Made for developers who are tired of googling the same commands.</sub>
</p>
