# Quick Start

Get up and running with RTFM in 2 minutes.

## 1. Download Cheatsheets

```bash
rtfm update
```

This downloads ~3000+ commands from [tldr-pages](https://github.com/tldr-pages/tldr).

## 2. Search for Commands

### Direct Query

```bash
# Search by command name
rtfm docker

# Search with spaces (uses fuzzy matching)
rtfm "copy files"
```

### Interactive TUI

```bash
# Start TUI mode
rtfm

# With classic style
rtfm --style classic
```

## 3. Basic Navigation

In TUI mode:

| Key | Action |
|-----|--------|
| `↑↓` or `jk` | Navigate results |
| `/` | Focus search box |
| `Enter` | View details |
| `Ctrl+T` | Switch style |
| `?` | Show help |
| `Esc` | Clear/Back/Quit |

## 4. Learn System Commands

Don't have a cheatsheet? Learn from your system:

```bash
# Learn from --help
rtfm learn cargo

# Learn from man page
rtfm learn --man grep

# Learn all commands in PATH
rtfm learn-all --source path --limit 50
```

## 5. Example Workflow

```bash
# Morning: Update cheatsheets
rtfm update

# During work: Quick lookups
rtfm git commit
rtfm docker compose

# New tool? Learn it
rtfm learn kubectl

# Start TUI for browsing
rtfm
```

## Next Steps

- [TUI Mode](./tui.md) - Learn about the terminal interface
- [CLI Usage](./cli.md) - All command-line options
- [Configuration](./configuration.md) - Customize RTFM
