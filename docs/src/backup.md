# Backup & Restore

RTFM supports full data backup and restore for migration or disaster recovery.

## Backup

```bash
rtfm backup rtfm-backup.tar.gz
```

### What's Included

- `data.redb` - Command database
- `index/` - Full-text search index
- `config.toml` - Configuration (if exists)
- `metadata.json` - Backup metadata

### Backup Output

```
Backing up data from /home/user/.local/share/rtfm...
  Adding data.redb...
  Adding index/...
  Adding config.toml (current config)...
  Adding metadata.json...

Backup complete!
  Output: rtfm-backup.tar.gz
  Size:   2048000 bytes (1.95 MB)
```

## Restore

```bash
# Replace existing data
rtfm restore rtfm-backup.tar.gz

# Merge with existing data
rtfm restore --merge rtfm-backup.tar.gz
```

### Restore Modes

| Mode | Behavior |
|------|----------|
| Default | Backup existing data, then replace |
| `--merge` | Add to existing data without replacing |

## Use Cases

### Migration to New Machine

```bash
# On old machine
rtfm backup rtfm-backup.tar.gz
# Transfer file to new machine

# On new machine
rtfm restore rtfm-backup.tar.gz
```

### Before Major Changes

```bash
# Backup before reset
rtfm backup pre-reset-backup.tar.gz
rtfm reset --yes

# If something goes wrong
rtfm restore pre-reset-backup.tar.gz
```

### Sharing with Team

```bash
# Create team backup with learned commands
rtfm backup team-commands.tar.gz
# Share via file server or git

# Team members restore
rtfm restore --merge team-commands.tar.gz
```

## Archive Format

The backup is a gzip-compressed tar archive:

```
rtfm-backup.tar.gz
├── data.redb          # redb database
├── index/             # Tantivy index
│   ├── meta.json
│   └── *.idx
├── config.toml        # Configuration
├── metadata.json      # Backup info
└── README.md          # Instructions
```

## Compatibility

- Backups are cross-platform (Windows/Linux/macOS)
- Database format (redb) is portable
- Search index (Tantivy) is portable
