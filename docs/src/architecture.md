# Architecture

Overview of RTFM's internal architecture.

## Project Structure

```
rtfm/
├── src/
│   ├── main.rs         # Entry point, CLI handling
│   ├── cli/            # Command-line interface (clap)
│   ├── config.rs       # Configuration management
│   ├── storage/        # Database layer (redb)
│   ├── search/         # Full-text search (Tantivy)
│   ├── tui/            # Terminal UI (ratatui)
│   │   ├── app.rs      # Application state
│   │   ├── ui.rs       # Rendering logic
│   │   ├── events.rs   # Event handling
│   │   └── mod.rs      # TUI entry point
│   ├── api/            # HTTP API (axum)
│   ├── update/         # Update logic
│   └── learn/          # System command learning
├── docs/               # Documentation (mdBook)
├── tests/              # Integration tests
└── benches/            # Performance benchmarks
```

## Core Components

### Storage Layer (`src/storage/`)

Uses [redb](https://github.com/cberner/redb) - a pure Rust embedded database.

```rust
pub struct Database {
  db: redb::Database,
}

// Tables
- commands: (name, lang) -> Command
- metadata: () -> Metadata
```

### Search Engine (`src/search/`)

Uses [Tantivy](https://github.com/quickwit-oss/tantivy) with jieba tokenization.

```rust
pub struct SearchEngine {
  index: Index,
  reader: IndexReader,
  // Fields: name, description, content, category, lang
}
```

### TUI (`src/tui/`)

Uses [ratatui](https://github.com/ratatui-org/ratatui) with crossterm backend.

```rust
pub struct App {
  // State
  query: String,
  results: Vec<SearchResult>,
  selected: usize,
  focus: Focus,
  ui_style: UiStyle,
  // ...
}
```

### HTTP API (`src/api/`)

Uses [axum](https://github.com/tokio-rs/axum) with utoipa for OpenAPI.

```rust
Router::new()
  .route("/api/search", get(search))
  .route("/api/commands/:name", get(get_command))
  .route("/api/import", post(import))
  // ...
```

## Data Flow

### Search Flow

```
User Input
    ↓
TUI/CLI/API
    ↓
SearchEngine::search()
    ↓
Tantivy Query Parser
    ↓
jieba Tokenization
    ↓
Index Lookup
    ↓
Results (sorted by score)
```

### Update Flow

```
rtfm update
    ↓
Check GitHub Release
    ↓
Download Archive
    ↓
Parse tldr Markdown
    ↓
Store in Database
    ↓
Rebuild Search Index
```

## Key Design Decisions

1. **Offline-first** - All data stored locally
2. **Embedded database** - No external database server
3. **Async runtime** - Tokio for HTTP server
4. **Sync TUI** - Main thread for terminal handling
5. **Chinese support** - First-class jieba integration

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `ratatui` | Terminal UI |
| `crossterm` | Terminal backend |
| `axum` | HTTP framework |
| `tantivy` | Full-text search |
| `jieba-rs` | Chinese tokenization |
| `redb` | Embedded database |
| `tokio` | Async runtime |
| `serde` | Serialization |
