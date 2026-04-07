# 🔭 Geminiscope

**Geminiscope** is a high-performance Rust TUI (Terminal User Interface) dashboard for [Gemini CLI](https://github.com/google/gemini-cli), providing real-time observability into AI sessions, costs, and system health.

Built with Rust and `ratatui`, Geminiscope transforms your CLI interaction history into a structured, searchable, and visually rich analytical environment.

![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/rust-stable-brightgreen)

## ✨ Features

- 🕵️ **Session Explorer:** Browse through chat history with `termimad` Markdown rendering.
- ⚡ **Real-time Tracking:** Instant updates as you chat, powered by `notify` event-based log parsing.
- 💰 **Cost Engine:** Track token usage and financial impact per model (Flash vs Pro).
- 📈 **Interactive Analytics:** Drill down into project-based usage with sparklines and token trends.
- 🔍 **Global Search:** Blazing fast search across all sessions, tools, and project hashes.
- 🔒 **Secret Scanner:** Entropy-aware background scanning for leaked API keys or credentials.
- 📡 **MCP Monitor:** Real-time health and status of your Model Context Protocol (MCP) servers.
- 🧠 **Context Browser:** View your `GEMINI.md` memory and global system prompts.
- 🗺️ **Plan Viewer:** Direct access to your implementation plans from `plans/*.md`.

## 🚀 Quick Start

### Installation

```bash
cargo install geminiscope
```

*Note: Currently requires a pre-existing Gemini CLI installation to monitor.*

### Usage

Simply run:
```bash
geminiscope
```

### Keybindings

- `j/k` or `Arrows`: Navigate lists
- `Enter`: Select item
- `/`: Open Global Search
- `1-9`: Switch views (Explorer, Stats, MCP, etc.)
- `Esc/q`: Exit or go back

## 🛠️ Architecture

Geminiscope is designed for performance and reliability:
- **Async Ingestion:** Background parsing of JSON logs using `tokio` and `notify`.
- **Modular TUI:** Separated models, parser, and rendering layers for high maintainability.
- **Resilient UI:** Handles terminal resizing and massive log files gracefully.

## 📝 Roadmap

- [ ] Multi-Mode Sorting (Cost, Tokens, Date)
- [ ] Exporting Session Summaries (Markdown/PDF)
- [ ] Customizable Themes (`themes.json`)
- [ ] Advanced Date Filtering

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request or open an Issue.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
