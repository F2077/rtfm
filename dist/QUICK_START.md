# RTFM Quick Start Guide

Get up and running with RTFM in under 2 minutes.

## Step 1: Install

Extract the archive and optionally add to your PATH:

**Linux/macOS:**
```bash
tar -xzf rtfm-*.tar.gz
sudo mv rtfm /usr/local/bin/
# Or just run from current directory: ./rtfm
```

**Windows:**
```powershell
# Extract the zip file, then either:
# 1. Run from the extracted directory: .\rtfm.exe
# 2. Add the directory to your PATH
```

## Step 2: Download Cheatsheets

```bash
rtfm update
```

This downloads cheatsheets from [tldr-pages](https://tldr.sh) (5000+ commands).

## Step 3: Use It

**Direct lookup:**
```bash
rtfm tar           # Look up tar command
rtfm docker        # Look up docker
rtfm git-commit    # Hyphenated commands work too
rtfm -l zh curl    # Specify language (zh = Chinese)
```

**Interactive TUI:**
```bash
rtfm               # Launch beautiful terminal UI
```

**TUI Keyboard Shortcuts:**
- `/` - Focus search
- `j/k` or `↑/↓` - Navigate
- `Enter` - Select
- `Tab` - Switch panels
- `Esc` - Back / Quit
- `Ctrl+H` - Help

## Learn Your Own Commands

Teach RTFM any command installed on your system:

```bash
rtfm learn rustc       # Learn from --help
rtfm learn --man grep  # Learn from man page
rtfm learn-all         # Learn all man pages (Linux/macOS)
```

## HTTP API (Optional)

```bash
rtfm serve                    # Start API server
rtfm serve --port 8080        # Custom port
rtfm serve --detach           # Run in background
```

Then visit http://localhost:3030/swagger-ui for API docs.

## Configuration (Optional)

Copy `config.example.toml` to `rtfm.toml` in your working directory to customize settings.

## Data Location

Your data is stored at:
- **Windows:** `%LOCALAPPDATA%\rtfm\`
- **Linux:** `~/.local/share/rtfm/`
- **macOS:** `~/Library/Application Support/rtfm/`

## Need Help?

```bash
rtfm --help           # Show all commands
rtfm <command> --help # Help for specific command
```

Visit https://github.com/F2077/rtfm for full documentation.
