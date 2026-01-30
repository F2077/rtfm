# Rust Beginner Guide

New to Rust? This guide helps you understand RTFM's codebase.

## Learning Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

## Key Concepts in RTFM

### Error Handling

RTFM uses `anyhow` for error handling:

```rust
use anyhow::Result;

fn do_something() -> Result<()> {
    let data = std::fs::read("file.txt")?;  // ? propagates errors
    Ok(())
}
```

### Async/Await

HTTP server uses async Rust:

```rust
async fn search(query: &str) -> Result<Vec<Result>> {
    let results = engine.search(query).await?;
    Ok(results)
}
```

### Traits

Common traits used:

```rust
// Serialization
#[derive(Serialize, Deserialize)]
struct Command { ... }

// Debug output
#[derive(Debug)]
struct App { ... }

// Cloning
#[derive(Clone)]
struct Config { ... }
```

### Pattern Matching

Used extensively:

```rust
match app.focus {
    Focus::Search => handle_search(key),
    Focus::List => handle_list(key),
    Focus::Detail => handle_detail(key),
}
```

### Option and Result

```rust
// Option: might be None
let cmd: Option<Command> = db.get_command("docker");
if let Some(cmd) = cmd {
    println!("{}", cmd.name);
}

// Result: might be error
let result: Result<Command> = db.get_command("docker")?;
```

## Building and Testing

```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test test_search

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Debugging Tips

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Run with backtrace
RUST_BACKTRACE=1 cargo run
```

## Common Patterns in RTFM

### Reading Files

```rust
let content = std::fs::read_to_string("file.md")?;
```

### Iterators

```rust
let names: Vec<String> = commands
    .iter()
    .map(|cmd| cmd.name.clone())
    .collect();
```

### String Handling

```rust
// UTF-8 aware
let chars: usize = text.chars().count();
let bytes: usize = text.len();
```

## Getting Help

- Check compiler error messages (they're helpful!)
- Search [docs.rs](https://docs.rs) for crate documentation
- Ask in Rust community forums
