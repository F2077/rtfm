# Changelog

All notable changes to RTFM.

## [0.1.0] - 2025-01-20

### Added
- Initial release
- **Modern TUI style**: New minimal layout with full-width result view
- **Style switching**: `Ctrl+T` to toggle between modern/classic styles
- **`--style` flag**: Command-line option to set UI style
- **Chinese input support**: Fixed UTF-8 character handling in search box
- TUI interface with classic layout
- Full-text search with Tantivy
- Chinese tokenization with jieba
- HTTP API with Swagger UI
- Command learning from `--help` and man pages
- Batch learning (`learn-all`)
- Import from local files/archives
- Backup and restore functionality
- Cross-platform support (Windows/Linux/macOS)

### Technical
- Database: redb
- Search: Tantivy
- TUI: ratatui + crossterm
- HTTP: axum + tower
- Async: tokio

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.0 | 2025-01-20 | Initial release |
