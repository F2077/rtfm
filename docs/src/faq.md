# FAQ

Frequently asked questions about RTFM.

## General

### What does RTFM stand for?

"Read The F***ing Manual" - a classic programmer's response when someone asks a question that's clearly documented.

### How is this different from tldr?

RTFM is inspired by tldr-pages but adds:
- Beautiful TUI interface
- Full-text search with Chinese support
- Learn commands from your system
- HTTP API for integration
- Offline-first design

### Is it really offline?

Yes! After initial `rtfm update`, everything works offline. Data is stored locally.

## Installation

### Why does compilation take so long?

RTFM has many dependencies (Tantivy, axum, etc.). First build downloads and compiles everything. Subsequent builds are faster.

### How do I update RTFM itself?

```bash
cd rtfm
git pull
cargo build --release
```

## Usage

### How do I update cheatsheets?

```bash
rtfm update
```

### Search isn't finding my command?

1. Try `rtfm update` to get latest cheatsheets
2. Try different search terms
3. Use `rtfm learn <command>` to add it locally

### How do I add custom commands?

Option 1: Learn from system
```bash
rtfm learn mycommand
```

Option 2: Import markdown files
```bash
rtfm import ./my-cheatsheets/
```

### TUI looks broken?

- Ensure terminal supports UTF-8
- Try a different terminal emulator
- Check terminal width (minimum ~80 columns)
- Try `--style classic` for simpler layout

## Troubleshooting

### "Database not found"

Run `rtfm update` first to initialize the database.

### Search returns no results

1. Check if database has data: `rtfm --version` shows command count
2. Try `rtfm update --force` to rebuild

### Chinese characters display incorrectly

- Use a terminal with CJK font support
- On Windows, try Windows Terminal
- Check locale settings

### HTTP server won't start

- Port 8080 might be in use: `rtfm serve --port 3000`
- Check firewall settings
- Try `--bind 127.0.0.1` (localhost only)

## Data

### Where is data stored?

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/rtfm/` |
| macOS | `~/Library/Application Support/rtfm/` |
| Windows | `%APPDATA%\rtfm\` |

### How do I reset everything?

```bash
rtfm reset --yes
```

### Can I sync between machines?

Use backup/restore:
```bash
# Machine A
rtfm backup sync.tar.gz

# Machine B
rtfm restore sync.tar.gz
```

## Contributing

### How can I contribute?

See [Contributing Guide](./contributing.md). Areas include:
- Documentation
- Bug fixes
- New features
- Translations
