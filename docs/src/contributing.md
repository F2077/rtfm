# Contributing

Thank you for your interest in contributing to RTFM!

## Getting Started

```bash
# Clone repository
git clone https://github.com/F2077/rtfm.git
cd rtfm

# Build
cargo build

# Run tests
cargo test

# Run with debug
cargo run -- --debug
```

## Development Setup

### Requirements

- Rust 1.75+
- Git

### Recommended Tools

```bash
# Install useful cargo tools
cargo install cargo-watch  # Auto-rebuild on changes
cargo install cargo-expand # Macro expansion

# Watch mode
cargo watch -x run
```

## Code Style

- Follow Rust conventions
- Use `cargo fmt` before committing
- Run `cargo clippy` to catch issues
- Write tests for new features

## Project Layout

```
src/
├── main.rs      # Entry point
├── cli/         # CLI definitions
├── tui/         # Terminal UI
├── api/         # HTTP API
├── storage/     # Database
├── search/      # Full-text search
├── update/      # Update logic
└── learn/       # Command learning
```

## Making Changes

### 1. Create a Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

- Keep commits focused and atomic
- Write descriptive commit messages
- Add tests for new functionality

### 3. Test

```bash
cargo test
cargo clippy
cargo fmt --check
```

### 4. Submit PR

- Describe what changed and why
- Reference any related issues
- Be responsive to review feedback

## Releasing

Releases are automated via GitHub Actions. Pushing a version tag triggers the release workflow which builds binaries for all platforms and creates a GitHub Release.

### Steps to Release

1. **Update version** in `Cargo.toml`:
   ```toml
   version = "0.2.0"
   ```

2. **Update changelog** in `docs/src/changelog.md`

3. **Commit changes**:
   ```bash
   git add Cargo.toml docs/src/changelog.md
   git commit -m "Bump version to 0.2.0"
   ```

4. **Create and push tag**:
   ```bash
   # Create annotated tag
   git tag -a v0.2.0 -m "Release v0.2.0"
   
   # Push commit and tag
   git push origin main
   git push origin v0.2.0
   ```

   Or push all tags at once:
   ```bash
   git push origin main --tags
   ```

### What Happens

When a `v*` tag is pushed, the release workflow will:

1. Run tests and clippy checks
2. Build binaries for Linux, macOS, and Windows (x86_64 and ARM64)
3. Create a macOS universal binary
4. Generate SHA256 checksums
5. Create a GitHub Release with all artifacts
6. Publish to crates.io (requires `CARGO_REGISTRY_TOKEN` secret)

### Manual Release

You can also trigger a release manually from GitHub Actions:

1. Go to Actions > Release workflow
2. Click "Run workflow"
3. Enter the tag (e.g., `v0.2.0`)

## Areas for Contribution

- **Docs**: Improve documentation
- **Tests**: Add test coverage
- **Features**: New functionality
- **Bugs**: Fix issues
- **Performance**: Optimization
- **i18n**: Translations

## Code of Conduct

Be respectful and constructive. We're all here to learn and build something useful.

## Questions?

Open an issue for discussion before major changes.
