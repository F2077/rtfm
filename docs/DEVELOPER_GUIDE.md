# RTFM Developer Guide

This guide covers the architecture, design patterns, and code organization of RTFM for experienced developers.

## Architecture Overview

RTFM follows a layered architecture with clear separation of concerns:

```
┌────────────────────────────────────────────────────────────────┐
│                      Presentation Layer                         │
│  ┌──────────────────────┐  ┌──────────────────────────────────┐│
│  │   TUI (ratatui)      │  │      HTTP API (axum)             ││
│  │  - app.rs            │  │  - search.rs, data.rs            ││
│  │  - ui.rs             │  │  - update.rs, learn.rs           ││
│  │  - events.rs         │  │  - OpenAPI/Swagger               ││
│  └──────────────────────┘  └──────────────────────────────────┘│
├────────────────────────────────────────────────────────────────┤
│                       CLI Layer (clap)                          │
│                        src/cli/mod.rs                           │
├────────────────────────────────────────────────────────────────┤
│                      Business Logic                             │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────────┐ │
│  │ search/mod.rs  │  │ learn/mod.rs   │  │ update/mod.rs    │ │
│  │ - Tantivy      │  │ - --help parse │  │ - tldr sync      │ │
│  │ - jieba token  │  │ - man parse    │  │ - markdown parse │ │
│  └────────────────┘  └────────────────┘  └──────────────────┘ │
├────────────────────────────────────────────────────────────────┤
│                       Data Layer                                │
│  ┌────────────────────────┐  ┌───────────────────────────────┐│
│  │   storage/mod.rs       │  │      config.rs                ││
│  │   - redb database      │  │      - TOML configuration     ││
│  │   - Command/Metadata   │  │      - defaults               ││
│  └────────────────────────┘  └───────────────────────────────┘│
└────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs          # Entry point, CLI dispatch, run_* functions
├── cli/
│   └── mod.rs       # clap argument definitions
├── tui/
│   ├── mod.rs       # TUI entry, event loop, logging setup
│   ├── app.rs       # Application state, business logic
│   ├── ui.rs        # UI rendering (ratatui widgets)
│   └── events.rs    # Keyboard event handling
├── api/
│   ├── mod.rs       # Router setup, OpenAPI definition
│   ├── search.rs    # GET /api/search
│   ├── data.rs      # GET/POST /api/command(s), /api/import
│   ├── update.rs    # GET/POST /api/update/*
│   └── learn.rs     # POST /api/learn*, GET /api/backup/info
├── storage/
│   └── mod.rs       # redb database wrapper, Command/Example types
├── search/
│   └── mod.rs       # Tantivy index, jieba tokenization
├── learn/
│   └── mod.rs       # --help/man parsing, batch learn
├── update/
│   └── mod.rs       # tldr-pages download, markdown parsing
└── config.rs        # AppConfig with TOML support
```

## Key Design Decisions

### 1. Embedded Database (redb)

We use redb instead of SQLite for:
- Zero configuration (no external dependencies)
- Cross-platform single-file storage
- Native Rust with excellent performance
- ACID transactions

```rust
// Storage pattern
pub struct Database {
    db: Arc<RedbDatabase>,
}

impl Database {
    pub fn save_command(&self, cmd: &Command) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(COMMANDS_TABLE)?;
            let key = format!("{}:{}", cmd.name, cmd.lang);
            table.insert(&*key, &serde_json::to_vec(cmd)?)?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
```

### 2. Full-text Search (Tantivy)

Tantivy provides Elasticsearch-like features in a Rust library:
- Inverted index for fast search
- Custom tokenizer support (jieba for Chinese)
- Query parsing with escaping

```rust
// Search with special character escaping
pub fn search(&self, query: &str, lang: Option<&str>, limit: usize) 
    -> Result<SearchResponse> {
    let tokenized = self.tokenize_query(query);
    let escaped = escape_special_chars(&tokenized);
    let query = self.parser.parse_query(&escaped)?;
    // ... execute search
}
```

### 3. Async Runtime (Tokio)

The project uses Tokio for:
- HTTP server (axum)
- Network requests (reqwest)
- TUI event loop (with `block_on` bridges)

```rust
// Mixed sync/async in TUI
impl App {
    pub async fn search(&mut self) {
        let search = self.search.read().await;
        match search.search(&self.query, None, 20) {
            Ok(response) => self.results = response.results,
            Err(e) => self.status = format!("Error: {}", e),
        }
    }
}
```

### 4. Configuration (TOML with Defaults)

All configuration has sensible defaults for zero-config operation:

```rust
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3030,
            bind: "127.0.0.1".to_string(),
        }
    }
}
```

Load priority: `rtfm.toml` (cwd) → `config.toml` (data dir) → defaults

### 5. OpenAPI Documentation (utoipa)

API documentation is generated from code annotations:

```rust
#[utoipa::path(
    get,
    path = "/api/search",
    params(SearchQuery),
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
    ),
    tag = "Search"
)]
pub async fn search(...) -> Result<Json<SearchResponse>, ...> { }
```

Swagger UI available at `/swagger-ui` when server is running.

## Data Flow

### TUI Search Flow

```
User Input → events.rs → App::input_char()
                            ↓
                      App::search()
                            ↓
                  SearchEngine::search()
                            ↓
               Tantivy Query Execution
                            ↓
                     Update results
                            ↓
                ui.rs renders new state
```

### Learn Command Flow

```
rtfm learn docker
        ↓
  run_learn() in main.rs
        ↓
  learn::get_help_output("docker")
        ↓
  Execute: docker --help
        ↓
  learn::parse_help_content()
        ↓
  Database::save_command()
        ↓
  SearchEngine::index_single_command()
```

## Error Handling

We use `anyhow` for CLI operations and `thiserror` for library errors:

```rust
// Library error types
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Tantivy error: {0}")]
    Tantivy(#[from] tantivy::TantivyError),
    #[error("Query parse error: {0}")]
    QueryParser(#[from] tantivy::query::QueryParserError),
}

// CLI uses anyhow::Result for ergonomic error handling
async fn run_update(force: bool, config: &AppConfig) -> anyhow::Result<()> {
    // ...
}
```

## Testing Strategy

### Unit Tests

Located alongside source files:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_special_chars() {
        assert_eq!(escape_special_chars("docker -a"), "docker \\-a");
    }
}
```

### Integration Tests

In `tests/` directory:

```rust
// tests/integration_test.rs
#[test]
fn test_command_name_normalization() {
    // Test CLI behavior
}
```

### Benchmarks

Using Criterion:

```rust
// benches/search_bench.rs
fn search_benchmark(c: &mut Criterion) {
    c.bench_function("search_simple", |b| {
        b.iter(|| search.search("docker", None, 20))
    });
}
```

Run with: `cargo bench`

## Building

### Development

```bash
cargo run              # Debug build
cargo run -- tar       # With args
cargo run -- --debug   # TUI with debug panel
```

### Release

```bash
# Single platform
cargo build --release

# Cross-platform (via justfile)
just build-all
```

### Profile Settings

Release profile in `Cargo.toml` optimizes for size:

```toml
[profile.release]
opt-level = "z"    # Size optimization
lto = true         # Link-time optimization
codegen-units = 1  # Better optimization
strip = true       # Strip symbols
panic = "abort"    # Smaller panic handling
```

## Adding New Features

### Adding a New CLI Command

1. Add to `src/cli/mod.rs`:
```rust
#[derive(Subcommand)]
pub enum Commands {
    // ...
    NewCommand {
        #[arg(short, long)]
        option: String,
    },
}
```

2. Handle in `main.rs`:
```rust
Some(Commands::NewCommand { option }) => {
    run_new_command(&option, &config).await
}
```

3. Implement `run_new_command()` function

### Adding a New API Endpoint

1. Add handler in `src/api/` module:
```rust
#[utoipa::path(...)]
pub async fn new_endpoint(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Response>, Json<ErrorResponse>> {
    // ...
}
```

2. Register in `src/api/mod.rs`:
```rust
.route("/new-endpoint", get(module::new_endpoint))
```

3. Add to OpenAPI paths and schemas

### Adding Configuration Options

1. Add field to appropriate config struct in `src/config.rs`
2. Add default value
3. Update `rtfm.example.toml`
4. Use in code via `config.section.field`

## Debugging

### TUI Debug Mode

```bash
rtfm --debug
```

Press Ctrl+L to toggle log panel in TUI (requires `--debug` flag).

### Logging

Set `RUST_LOG` environment variable:

```bash
RUST_LOG=debug rtfm serve
RUST_LOG=rtfm=trace,tantivy=info rtfm
```

### API Testing

```bash
# Start server
rtfm serve

# Test endpoints
curl http://localhost:3030/api/health
curl "http://localhost:3030/api/search?q=docker"

# View Swagger UI
open http://localhost:3030/swagger-ui
```

## Code Style

- Format: `cargo fmt`
- Lint: `cargo clippy`
- Comments: Chinese for internal documentation
- Messages/Logs: English

## Performance Considerations

1. **Search Index**: Tantivy index is memory-mapped, efficient for large datasets
2. **Database Transactions**: Batch writes when possible
3. **TUI Rendering**: Only redraw on changes, use efficient diff
4. **Async I/O**: Network operations don't block UI

## Common Patterns

### State Sharing in TUI

```rust
pub struct App {
    pub search: Arc<RwLock<SearchEngine>>,  // Async-safe sharing
    pub db: Database,                        // Sync, cloneable
}
```

### API State

```rust
pub struct AppState {
    pub db: Database,
    pub search: RwLock<SearchEngine>,
    pub config: AppConfig,
}

// Used as axum state
State(state): State<Arc<AppState>>
```

### Error Response Pattern

```rust
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

// Handler returns Result
pub async fn handler() -> Result<Json<Data>, Json<ErrorResponse>> {
    match operation() {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err(Json(ErrorResponse { error: e.to_string() })),
    }
}
```
