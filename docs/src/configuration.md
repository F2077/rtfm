# Configuration

RTFM can be configured via a TOML file.

## Configuration File Location

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/rtfm/config.toml` |
| macOS | `~/Library/Application Support/rtfm/config.toml` |
| Windows | `%APPDATA%\rtfm\config.toml` |

## Default Configuration

```toml
[storage]
db_filename = "data.redb"
index_dirname = "index"
log_dirname = "logs"

[update]
github_repo = "tldr-pages/tldr"
languages = ["en", "zh"]

[server]
default_port = 8080
default_bind = "127.0.0.1"
max_upload_size = 10485760  # 10MB

[search]
default_limit = 20
min_score = 0.1

[logging]
level = "info"
debug_level = "debug"

[tui]
style = "modern"
poll_timeout_ms = 100
log_buffer_size = 100
scroll_step = 1
```

## Configuration Sections

### `[storage]`

| Key | Type | Description |
|-----|------|-------------|
| `db_filename` | string | Database file name |
| `index_dirname` | string | Search index directory |
| `log_dirname` | string | Log files directory |

### `[update]`

| Key | Type | Description |
|-----|------|-------------|
| `github_repo` | string | GitHub repository for updates |
| `languages` | array | Languages to download (e.g., `["en", "zh"]`) |

### `[server]`

| Key | Type | Description |
|-----|------|-------------|
| `default_port` | integer | Default HTTP server port |
| `default_bind` | string | Default bind address |
| `max_upload_size` | integer | Max upload size in bytes |

### `[search]`

| Key | Type | Description |
|-----|------|-------------|
| `default_limit` | integer | Default result limit |
| `min_score` | float | Minimum relevance score |

### `[logging]`

| Key | Type | Description |
|-----|------|-------------|
| `level` | string | Normal log level |
| `debug_level` | string | Debug mode log level |

### `[tui]`

| Key | Type | Description |
|-----|------|-------------|
| `style` | string | UI style: "modern" or "classic" |
| `poll_timeout_ms` | integer | Event poll timeout |
| `log_buffer_size` | integer | Debug log buffer size |
| `scroll_step` | integer | Scroll step size |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Override log level (e.g., `debug`, `rtfm=trace`) |

## Examples

### Chinese-only Setup

```toml
[update]
languages = ["zh"]
```

### Multi-language Setup

```toml
[update]
languages = ["en", "zh", "ja", "ko"]
```

### Classic Style Default

```toml
[tui]
style = "classic"
```

### Custom Data Directory

Currently not configurable via config file. Use symlinks if needed.
