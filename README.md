# 🔭 Geminiscope

**Geminiscope** is a high-performance, real-time observability dashboard for [Gemini CLI](https://github.com/google/gemini-cli). Built with Rust and `ratatui`, it provides deep insights into your AI-assisted development sessions, including cost analysis, token usage, and system health.

![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/rust-1.80+-brightgreen)
![Build](https://img.shields.io/badge/build-passing-brightgreen)

## 🚀 Key Capabilities

- ⚡ **Real-time Log Ingestion**: Uses `notify` for zero-latency synchronization with Gemini CLI logs.
- 💰 **Financial Intelligence**: Dynamic pricing engine for Gemini Pro/Flash models with historical token tracking.
- 🔍 **High-Density Custom Rendering**: A hand-optimized TUI renderer designed for maximum information density without external markdown overhead.
- 🔒 **Entropy-Aware Security**: Background scanning for API keys and credentials using Shannon entropy analysis.
- 📡 **MCP Observability**: Real-time status monitoring for Model Context Protocol (MCP) servers.
- 📦 **Session Archiving**: Instant export of raw session data to JSON (`e` key) for offline analysis.

## 🛠️ Engineering Architecture

Geminiscope is designed for engineers who value performance and reliability:

- **Async Core**: Powered by `tokio`, log parsing and security scanning run in non-blocking background threads.
- **Custom Render Pipeline**: Replaced generic markdown libraries with a specialized line-buffer renderer to eliminate vertical padding and maximize vertical space.
- **Resilient Data Handling**: Intelligent truncation of massive JSON payloads (up to 500KB) ensures the UI remains responsive even during heavy file-reading operations.
- **Modular View-Based Design**: Clean separation between state management (`app.rs`), data modeling (`models.rs`), and modular UI components (`src/ui/`).

## 📥 Installation

```bash
cargo install geminiscope
```

*Prerequisite: An active installation of [Gemini CLI](https://github.com/google/gemini-cli).*

## ⌨️ Controls

| Key | Action |
| :--- | :--- |
| `1-9` | Switch between Views (Chats, Stats, MCP, etc.) |
| `j/k`, `Arrows` | Navigate Sidebar |
| `J/K`, `Alt+Arrows` | Scroll Detail Pane |
| `/` | Global Search / Filter |
| `s` | Cycle Sort Modes (Date, Cost, Tokens, Name) |
| `e` | Export Current Session to JSON |
| `q`, `Esc` | Quit / Back |

## 🗺️ Roadmap

- [x] Multi-mode sorting architecture.
- [x] High-performance custom TUI rendering.
- [x] Real-time security and health auditing.
- [ ] Customizable theme engine (`themes.json`).
- [ ] Cross-session diffing tools.

## 📄 License

MIT © [Raunak Jyotishi](https://github.com/ronmkr)
