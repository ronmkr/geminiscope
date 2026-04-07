# Geminiscope: High-Performance Rust TUI Dashboard for Gemini CLI

Geminiscope is an observability dashboard for your AI development workflow. It provides real-time insights into your session history, costs, and token usage through a highly interactive terminal interface.

![Geminiscope Preview](screenshots/preview.png)

## 🚀 Key Features

*   **Session Explorer**: High-density custom renderer with syntax highlighting for JSON and Markdown.
*   **Real-time Observability**: Integrated file-system watcher (`notify`) for instant UI updates.
*   **Advanced Cost Tracking**: Pricing engine for Gemini Pro/Flash models with historical token usage trends.
*   **Session Diffing**: Side-by-side comparison of AI turns to track prompt/parameter changes.
*   **Smart Navigation**: Vim-bindings, mouse-click support for sidebar icons, and precise scrolling.
*   **Security First**: Shannon Entropy secret scanning and automatic UI redaction of sensitive tokens.
*   **Clean Architecture**: Fully modular Rust implementation following DRY principles and strict Clippy guidelines.

## 🛠️ Keybindings

| Key | Action |
| :--- | :--- |
| `?` / `h` | **Open Help Menu** (Detailed documentation) |
| `1-9` | Switch between primary views (Chats, Stats, Tools, etc.) |
| `0` | **Settings View** (Interactive configuration editing) |
| `d` | **Session Diff**: Press on 1st session, then 2nd to compare. |
| `Ctrl+R` | **Toggle Secret Redaction** (Mask/Unmask sensitive keys) |
| `o` | **Open in Editor** (Launch preferred editor for Memory/Plans) |
| `j` / `k` | Move cursor Up/Down (Click sidebar icons to jump) |
| `J` / `K` | Scroll detail view Up/Down (Mouse wheel supported) |
| `/` | Search/Filter current view |
| `s` | Cycle sort modes (Date, Cost, Tokens, Name) |
| `e` | Export raw session/view to JSON (Secured with 0600 permissions) |
| `q` | Quit Application |

## 📦 Installation

```bash
cargo install --path .
```

## 🎨 Configuration

Geminiscope stores its configuration in `~/.gemini/settings.json`. You can also define custom themes in `~/.gemini/themes.json` and select them live in the Settings view.

---
*Built with Rust, Ratatui, and Tokio.*
