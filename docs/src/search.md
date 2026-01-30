# Full-text Search

RTFM uses [Tantivy](https://github.com/quickwit-oss/tantivy) for full-text search with Chinese tokenization support via [jieba-rs](https://github.com/messense/jieba-rs).

## How It Works

### Indexing

When you run `rtfm update` or `rtfm learn`, commands are indexed with:

1. **Command name** - The command name (e.g., `docker`, `git-commit`)
2. **Description** - Brief description of the command
3. **Content** - Full content including examples

All text fields are tokenized using jieba for proper Chinese word segmentation.

### Querying

When you search:

1. Query is tokenized with jieba
2. Tokens are searched across name, description, and content fields
3. Results are ranked by relevance score
4. Top results are returned

## Search Examples

### English Commands

```bash
rtfm docker          # Exact match on name
rtfm "list files"    # Fuzzy match on content
rtfm "compress tar"  # Match description/content
```

### Chinese Search

```bash
rtfm 复制文件         # Search "copy files" in Chinese
rtfm 压缩             # Search "compress"
rtfm 显示目录         # Search "show directory"
```

### Mixed Language

```bash
rtfm "docker 容器"    # Mixed English and Chinese
rtfm "git 提交"       # "git commit" in Chinese
```

## Search Tips

1. **Use keywords** - Short, specific terms work best
2. **Try both languages** - Some commands have both English and Chinese docs
3. **Check spelling** - Typos won't match
4. **Use quotes** - For multi-word phrases

## Technical Details

### Tokenization

- **English**: Split on whitespace and punctuation, lowercase
- **Chinese**: jieba word segmentation with HMM for new words
- **Special characters**: Escaped for Tantivy query syntax

### Scoring

Results are ranked by:
- Term frequency (TF)
- Inverse document frequency (IDF)
- Field boost (name > description > content)

### Index Structure

```
index/
├── meta.json           # Index metadata
├── .managed.json       # Tantivy management file
└── *.idx               # Segment files
```

## Rebuilding Index

If search seems broken:

```bash
# Force rebuild
rtfm update --force
```

This will:
1. Delete existing index
2. Re-download cheatsheets
3. Rebuild index from scratch
