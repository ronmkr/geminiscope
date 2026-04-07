# Geminiscope: High-Performance Rust TUI Dashboard for Gemini CLI

Geminiscope is an observability dashboard for your AI development workflow. It provides real-time insights into your session history, costs, and token usage through a highly interactive terminal interface.

![Geminiscope Preview](screenshots/preview.png)

## 🚀 Key Features

*   **Session Explorer**: High-density custom renderer with syntax highlighting for JSON and Markdown.
*   **Real-time Observability**: Integrated file-system watcher (`notify`) for instant UI updates as you interact with Gemini CLI.
*   **Advanced Cost Tracking**: Pricing engine for Gemini Pro/Flash models with historical token usage trends.
*   **Interactive Theme Engine**: Support for custom color schemes via `~/.gemini/themes.json`.
*   **Smart Navigation**: Vim-like bindings (`j/k`), global search (`/`), and precise scrolling (`J/K`).
*   **Health & Security Scanning**: Shannon Entropy scanning to detect leaked secrets or API keys in your chat history.
*   **Project Context**: Automatic discovery of `GEMINI.md` files and development plans across your local workspaces.

## 🛠️ Keybindings

| Key | Action |
| :--- | :--- |
| `?` / `h` | **Open Help Menu** (Detailed documentation) |
| `1-9` | Switch between Views (Chats, Stats, Tools, Health, etc.) |
| `0` | **Settings View** (Interactive configuration editing) |
| `j` / `k` | Move cursor Up/Down |
| `J` / `K` | Scroll detail view Up/Down |
| `/` | Search/Filter current view |
| `s` | Cycle sort modes (Date, Cost, Tokens, Name) |
| `e` | Export raw session/view to JSON |
| `q` | Quit Application |

## 📦 Installation

```bash
cargo install --path .
```

## 🎨 Configuration

Geminiscope stores its configuration in `~/.gemini/settings.json`. You can also define custom themes in `~/.gemini/themes.json`.

---
*Built with Rust, Ratatui, and Tokio.*
