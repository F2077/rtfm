# RTFM - Product Specification

> Version: 1.0 | Last Updated: 2026-01-23

## Overview

RTFM (Read The F***ing Manual) is a cross-platform offline CLI cheatsheet tool, designed as an enhanced alternative to tldr-pages with additional features like learning from system help and full-text search.

## Product Goals

1. **Offline First**: Work entirely offline without network dependency
2. **Fast Lookup**: Instant command lookup via CLI (`rtfm tar`)
3. **Learnable**: Capture and index any command's `--help` or `man` output
4. **Searchable**: Full-text search with Chinese/English support
5. **Portable**: Cross-platform (Windows/Linux/macOS) with single binary
6. **Extensible**: HTTP API for integration with other tools

## Core Features

### 1. Direct Command Lookup

```bash
rtfm tar          # Show tar usage
rtfm -l zh docker # Chinese language
```

- Query by exact command name
- Multi-language support (en, zh, etc.)
- Fallback chain: specified language -> zh -> en

### 2. TUI Interface

- Modern terminal UI with ratatui
- Three-panel layout: Search / Results List / Details
- Vim-style keyboard navigation (j/k)
- Real-time search as you type
- Debug log panel (F12 in debug mode)

### 3. Learn from System

```bash
rtfm learn rustc       # Learn from --help
rtfm learn --man grep  # Prefer man page
rtfm learn-all         # Batch learn from man section
```

- Parse `--help` output to extract description/examples
- Fall back to `man` pages when `--help` unavailable
- Batch import entire man sections

### 4. Full-text Search

- Powered by Tantivy search engine
- Chinese tokenization via jieba-rs
- Special character escaping for query safety
- Relevance-ranked results

### 5. HTTP API Server

```bash
rtfm serve --port 3030
```

- RESTful API endpoints
- OpenAPI/Swagger documentation at `/swagger-ui`
- CORS support for web integration

### 6. Data Management

```bash
rtfm update              # Sync from tldr-pages
rtfm import ./docs       # Import local markdown
rtfm backup -o data.tar.gz  # Backup all data
rtfm restore data.tar.gz    # Restore from backup
```

## Technical Specifications

### Architecture

```
┌─────────────────────────────────────────────────┐
│                    CLI Layer                     │
│              (clap argument parser)              │
├─────────────────────────────────────────────────┤
│     TUI Layer        │       API Layer          │
│    (ratatui)         │        (axum)            │
├─────────────────────────────────────────────────┤
│                  Core Services                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │ Search  │  │ Storage │  │ Update/Learn    │ │
│  │(tantivy)│  │ (redb)  │  │                 │ │
│  └─────────┘  └─────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────┤
│               Configuration Layer                │
│                (TOML + defaults)                 │
└─────────────────────────────────────────────────┘
```

### Data Storage

| Component | Format | Location |
|-----------|--------|----------|
| Database | redb (embedded KV) | `data.redb` |
| Search Index | Tantivy | `index/` |
| Logs | Rolling text | `logs/rtfm.log` |
| Config | TOML | `config.toml` |

### Command Schema

```rust
struct Command {
    name: String,        // e.g., "docker-compose"
    description: String, // Brief description
    category: String,    // e.g., "common", "linux"
    platform: String,    // e.g., "common", "windows"
    lang: String,        // e.g., "en", "zh"
    examples: Vec<Example>,
    content: String,     // Full raw content
}

struct Example {
    description: String, // What this example does
    code: String,        // The command to run
}
```

### Configuration

All configuration via TOML with sensible defaults:

```toml
[server]
port = 3030
bind = "127.0.0.1"

[search]
default_limit = 20
max_limit = 100

[update]
github_api_url = "https://api.github.com/repos/tldr-pages/tldr/releases/latest"
```

## Non-Functional Requirements

### Performance

- Search latency: < 50ms for 10,000 commands
- TUI render: < 16ms per frame (60fps)
- Startup time: < 500ms cold start

### Size

- Binary size: < 15MB (release build with LTO)
- Minimal runtime dependencies

### Compatibility

- Rust 1.75+
- Windows 10+, Linux (glibc 2.17+), macOS 10.15+

## Code Standards

### Language

- **Output messages**: English
- **Log messages**: English
- **Code comments**: Chinese (for internal documentation)
- **API responses**: English

### Testing

- Unit tests for core logic
- Integration tests for CLI commands
- Benchmark tests for search performance

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/search` | Full-text search |
| GET | `/api/command/{name}` | Get command details |
| GET | `/api/commands` | List all commands |
| GET | `/api/metadata` | Database metadata |
| POST | `/api/import` | Import commands |
| GET | `/api/update/check` | Check for updates |
| POST | `/api/update/download` | Download updates |
| POST | `/api/learn` | Learn single command |
| POST | `/api/learn-all` | Batch learn |
| GET | `/api/backup/info` | Backup info |

## Future Considerations

1. **Plugin System**: Allow custom parsers for different help formats
2. **Fuzzy Search**: Typo-tolerant command matching
3. **Shell Integration**: Auto-completion scripts
4. **Sync Service**: Cloud sync for cross-device access
5. **AI Enhancement**: LLM-powered example generation
