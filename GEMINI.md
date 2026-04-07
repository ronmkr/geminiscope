# Geminiscope: Project Roadmap & Capability Map

Geminiscope is a high-performance Rust TUI dashboard for Gemini CLI, providing real-time observability into AI sessions, costs, and system health.

## 🎯 Capability Mapping

| Feature | Status | Implementation Strategy |
| :--- | :--- | :--- |
| **Session Explorer** | ✅ Done | High-density custom renderer with syntax highlighting. |
| **Real-time Tracking** | ✅ Done | Integrated `notify` events with debounced background parser. |
| **Vim Bindings** | ✅ Done | `j/k` for navigation, `J/K` or `Alt+Arrows` for detail scrolling. |
| **Global Search** | ✅ Done | `/` to filter sessions by content across Chats and Tools. |
| **Multi-Mode Sorting**| ✅ Done | Sort projects by Date, Cost, Tokens, or Name. |
| **Cost Estimation** | ✅ Done | Pricing engine for Pro/Flash models + token math. |
| **Analytics (Stats)** | ✅ Done | Interactive Sparklines for token trends. |
| **Secret Scanning** | ✅ Done | Entropy-aware background scan of `chats/*.json`. |
| **Session Export** | ✅ Done | `e` key to export raw un-truncated JSON for large logs. |
| **Notification System**| ✅ Done | Immediate feedback banner for file operations. |

## 🛠️ Engineering Specs & Performance

### 1. Custom Render Pipeline
To eliminate the vertical padding issues inherent in standard markdown libraries, Geminiscope uses a specialized line-buffer renderer. This allows for:
- **Maximum Density**: Multiple blocks of content (JSON, Headers, Text) are collapsed into a single cohesive view.
- **Dynamic Syntax Highlighting**: Real-time parsing of JSON structures to provide color-coded keys and values without blocking the UI thread.

### 2. High-Volume Data Strategy
AI logs can grow to several megabytes. Geminiscope maintains performance through:
- **Intelligent Truncation**: Strings exceeding 1000 characters are truncated in the UI preview to maintain 60FPS rendering.
- **Off-Thread Parsing**: All JSON deserialization happens outside the main event loop.
- **On-Demand Export**: Users can bypass UI limits by exporting the full raw payload to disk for external viewing.

## 🚀 Next Steps & Structural Improvements

### 1. Architectural Refactoring
- [x] **Modular UI**: Decoupled UI into specialized sub-modules (`explorer`, `stats`, `infrastructure`).
- [ ] **Parser Decomposition**: Move specialized handlers (MCP, Session, Tokens) into a dedicated `src/parser/` module.

### 2. Advanced Features
- [ ] **Theme Engine**: Support for `themes.json` to allow user-defined color schemes.
- [ ] **Session Diffing**: Compare two AI turns to see how prompts or parameters changed.

## 🛠️ Tech Stack Notes
- **Language**: Rust
- **TUI**: [ratatui](https://github.com/ratatui-org/ratatui)
- **Async**: [tokio](https://github.com/tokio-rs/tokio)
- **File Watching**: [notify](https://github.com/notify-rs/notify)
- **Security**: Shannon Entropy scanning for credentials.
