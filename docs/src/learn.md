# Learning Commands

RTFM can learn commands from your system by extracting information from `--help` output or man pages.

## Basic Usage

```bash
# Learn from --help (default)
rtfm learn cargo

# Learn from man page
rtfm learn --man grep

# Re-learn existing command
rtfm learn --force git
```

## How It Works

1. **Execute command** - Run `command --help` or `man command`
2. **Parse output** - Extract description and options
3. **Generate examples** - Create usage examples from options
4. **Store locally** - Save to database with `lang: local`
5. **Index for search** - Add to full-text search index

## Batch Learning

### From Man Pages (Linux/macOS)

```bash
# Learn all commands in man section 1 (user commands)
rtfm learn-all --section 1

# Limit to 100 commands
rtfm learn-all --section 1 --limit 100

# Skip already learned
rtfm learn-all --section 1 --skip-existing
```

Man sections:
- 1: User commands
- 2: System calls
- 3: Library functions
- 5: File formats
- 8: System administration

### From PATH (All Platforms)

```bash
# Learn executables in PATH
rtfm learn-all --source path --limit 50

# Filter by prefix
rtfm learn-all --source path --prefix git
```

### From PowerShell (Windows)

```bash
# Learn PowerShell cmdlets
rtfm learn-all --source powershell --limit 100
```

## Learned Command Format

Learned commands are stored with:

```
name: command-name
lang: local
category: learned
platform: current-platform
description: Extracted from help
examples:
  - description: Option description
    code: command --option
```

## Viewing Learned Commands

```bash
# Search for learned commands
rtfm cargo

# In TUI, look for [local] tag
```

## Limitations

1. **Quality varies** - Help output formats differ widely
2. **No translations** - Learned commands are in original language
3. **May miss context** - Automated parsing isn't perfect
4. **Platform specific** - Learned on one platform may not work on another

## Tips

1. **Start with popular tools** - Learn tools you use daily first
2. **Use --force** - Re-learn if initial parse was bad
3. **Check results** - Verify learned content is useful
4. **Contribute upstream** - If you create good cheatsheets, consider contributing to tldr-pages
