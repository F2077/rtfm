# RTFM - Read The F***ing Manual

> *"When all else fails... READ THE F***ING MANUAL"*

**RTFM** is a Rust-powered CLI cheatsheet tool that brings command-line documentation right to your terminal. It combines the simplicity of [tldr-pages](https://github.com/tldr-pages/tldr) with powerful features like full-text search, a beautiful TUI, and an HTTP API.

## Features

- **TUI Interface** - Beautiful terminal UI with two styles (modern/classic)
- **Full-text Search** - Powered by Tantivy with Chinese tokenization (jieba)
- **Offline First** - All data stored locally, works without internet
- **Learn from System** - Extract docs from `--help` and `man` pages
- **HTTP API** - RESTful API with Swagger UI for integration
- **Cross-platform** - Works on Windows, macOS, and Linux

## Quick Example

```bash
# Search for a command
rtfm docker

# Start interactive TUI
rtfm

# Update cheatsheets from tldr-pages
rtfm update

# Learn a command from your system
rtfm learn git
```

## Why RTFM?

1. **Faster than Google** - Get answers instantly in your terminal
2. **Works offline** - Perfect for servers without internet access
3. **Chinese support** - Full Chinese language support with proper tokenization
4. **Extensible** - Learn new commands, import custom cheatsheets

### In the Age of AI Agents

You might ask: *"Why do I need this when I can just ask ChatGPT/Copilot/Claude?"*

Great question. Here's the thing: **AI agents need internet access**. When you're:

- Working in an air-gapped environment (security, compliance, or military)
- On a plane, train, or remote location with no connectivity
- Behind a corporate firewall that blocks AI services
- In a data center with restricted network access
- Simply experiencing an internet outage

...your AI assistant can't help you. But **RTFM works 100% offline**.

Built entirely in **Rust**, RTFM is a single binary with no runtime dependencies. Copy it to any machine and it just works.

## Screenshot

```
      _________                  "When all else fails..."
     /        /|
    /  RTFM  / |                   READ THE F***ING MANUAL
   /________/  |   < Go on, I DARE you to ask again!
   |  ~~~~  |  |
   | MANUAL |  /                   Rust-powered CLI Cheatsheet
   |________|/

╭─ Search ──────────────────────────────────────────────────────╮
│ > docker                                                       │
╰────────────────────────────────────────────────────────────────╯
╭─ Result [1/8] ─────────────────────────────────────────────────╮
│   docker  [en]                                                 │
│   Manage Docker containers and images.                         │
│                                                                 │
│   → Run a container                                            │
│     docker run {{image}}                                       │
│                                                                 │
│   → List running containers                                    │
│     docker ps                                                  │
╰────────────────────────────────────────────────────────────────╯
```

## License

This project is licensed under the GPL-3.0 License.
