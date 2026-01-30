# CLI Usage

Complete reference for RTFM command-line interface.

## Basic Usage

```bash
rtfm [OPTIONS] [QUERY] [COMMAND]
```

## Options

| Option | Description |
|--------|-------------|
| `--lang <LANG>` | Preferred language (default: zh) |
| `--style <STYLE>` | UI style: modern or classic |
| `--debug` | Enable debug mode |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Commands

### `rtfm` (no arguments)

Start interactive TUI mode.

```bash
rtfm
rtfm --style classic
rtfm --debug
```

### `rtfm <query>`

Search and display command information.

```bash
rtfm docker
rtfm "git commit"
rtfm tar
```

### `rtfm update`

Update cheatsheets from tldr-pages.

```bash
rtfm update          # Normal update
rtfm update --force  # Force re-download
```

### `rtfm import <path>`

Import cheatsheets from local files.

```bash
rtfm import ./my-commands/
rtfm import ./cheatsheet.md
rtfm import ./tldr-archive.zip
```

### `rtfm learn <command>`

Learn a command from system help.

```bash
rtfm learn cargo           # From --help
rtfm learn --man grep      # Prefer man page
rtfm learn --force git     # Re-learn existing
```

### `rtfm learn-all`

Batch learn commands.

```bash
# Learn from man pages (Linux/macOS)
rtfm learn-all --section 1 --limit 100

# Learn from PATH (all platforms)
rtfm learn-all --source path --limit 50

# Learn PowerShell cmdlets (Windows)
rtfm learn-all --source powershell

# Filter by prefix
rtfm learn-all --prefix git --source path
```

Options:
- `--section <N>` - Man section (1-8)
- `--limit <N>` - Maximum commands to learn
- `--skip-existing` - Skip already learned commands
- `--prefix <PREFIX>` - Filter by command prefix
- `--source <SOURCE>` - Source: auto, man, path, powershell

### `rtfm serve`

Start HTTP API server.

```bash
rtfm serve                        # Default port 8080
rtfm serve --port 3000            # Custom port
rtfm serve --bind 0.0.0.0         # Bind to all interfaces
rtfm serve --detach               # Run in background
rtfm serve --debug                # With debug logging
```

### `rtfm backup <output>`

Backup all data to archive.

```bash
rtfm backup rtfm-backup.tar.gz
```

### `rtfm restore <path>`

Restore from backup.

```bash
rtfm restore rtfm-backup.tar.gz
rtfm restore --merge backup.tar.gz  # Merge with existing
```

### `rtfm reset`

Delete all data (factory reset).

```bash
rtfm reset       # Interactive confirmation
rtfm reset --yes # Skip confirmation
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (command not found, network error, etc.) |

## Examples

```bash
# Daily workflow
rtfm update && rtfm

# Quick lookup
rtfm docker run

# Learn new tools
rtfm learn kubectl
rtfm learn helm

# Backup before reinstall
rtfm backup ~/rtfm-backup.tar.gz

# Start API server for team
rtfm serve --bind 0.0.0.0 --port 8080 --detach
```
