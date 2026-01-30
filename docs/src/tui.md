# TUI Mode

RTFM features a beautiful terminal user interface with two styles.

## Starting TUI

```bash
# Default (modern style)
rtfm

# Classic style
rtfm --style classic

# With debug logs
rtfm --debug
```

## Styles

### Modern Style (Default)

A clean, minimal layout inspired by modern CLI tools:

```
      _________                  "When all else fails..."
     /        /|
    /  RTFM  / |                   READ THE F***ING MANUAL
   /________/  |
   |  ~~~~  |  |
   | MANUAL |  /                   Rust-powered CLI Cheatsheet
   |________|/

╭─ Search ───────────────────────────────────────────────────────╮
│ > docker                                                        │
╰─────────────────────────────────────────────────────────────────╯
╭─ Result [1/8] ──────────────────────────────────────────────────╮
│   docker  [en]                                                  │
│   Manage Docker containers and images.                          │
│                                                                  │
│   → Run a container                                             │
│     docker run {{image}}                                        │
╰─────────────────────────────────────────────────────────────────╯
```

Features:
- Full-width layout (no horizontal splits)
- Single result view with navigation
- Logo displayed when space allows
- Rounded borders

### Classic Style

A traditional two-panel layout:

```
╭─ Results (8) ──────────╮╭─ Details ─────────────────────────────╮
│ ► docker               ││ docker                                │
│   docker-compose       ││ Manage Docker containers and images.  │
│   docker-run           ││                                       │
│   docker-ps            ││ Run a container:                      │
│                        ││   docker run {{image}}                │
╰────────────────────────╯╰───────────────────────────────────────╯
```

Features:
- Side-by-side list and details
- ASCII art logo at top
- Classic border style

## Keyboard Shortcuts

### Global

| Key | Action |
|-----|--------|
| `Ctrl+T` | Toggle style (modern/classic) |
| `Ctrl+H` | Toggle help popup |
| `Ctrl+L` | Toggle debug logs (requires `--debug`) |
| `Ctrl+C` / `Ctrl+Q` | Force quit |

### Search Box

| Key | Action |
|-----|--------|
| `↑↓` | Navigate to results |
| `Enter` | Go to results |
| `Esc` | Clear search / Quit |
| `Ctrl+U` | Clear all |
| `←→` | Move cursor |
| `Home/End` | Jump to start/end |

### Results Navigation

#### Modern Style

| Key | Action |
|-----|--------|
| `↑↓` / `jk` | Scroll content |
| `←→` / `hl` | Switch between results |
| `PgUp/PgDn` | Page scroll |
| `g` / `G` | Jump to first/last result |
| `/` | Focus search |
| `Esc` / `Tab` | Back to search |

#### Classic Style

| Key | Action |
|-----|--------|
| `↑↓` / `jk` | Navigate list |
| `PgUp/PgDn` | Page navigation |
| `g` / `G` | Jump to first/last |
| `Enter` / `→` / `l` | View details |
| `/` | Focus search |

### Details View (Classic Style)

| Key | Action |
|-----|--------|
| `↑↓` / `jk` | Scroll content |
| `PgUp/PgDn` | Page scroll |
| `Home/End` | Jump to top/bottom |
| `←` / `h` / `Esc` | Back to list |

## Configuration

Set default style in config:

```toml
[tui]
style = "modern"  # or "classic"
poll_timeout_ms = 100
log_buffer_size = 100
```

Or via command line:

```bash
rtfm --style classic
```

## Chinese Input Support

RTFM fully supports Chinese input in the search box:
- Proper cursor positioning for CJK characters
- Unicode-aware text width calculation
- Works with any IME (Input Method Editor)
