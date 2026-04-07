# Geminiscope: Project Roadmap & Capability Map
Geminiscope is a high-performance Rust TUI dashboard for Gemini CLI, providing real-time observability into AI sessions, costs, and system health.
## 🎯 Capability Mapping
| Feature | Status | TUI Implementation Strategy |
| :--- | :--- | :--- |
| **Session Explorer** | ✅ Done | Sidebar + Viewport with `termimad` Markdown rendering. |
| **Real-time Tracking** | ✅ Done | Integrated `notify` events with debounced background parser. |
| **Vim Bindings** | ✅ Done | `j/k` for navigation, `PgUp/PgDn` for detail scrolling. |
| **Global Search** | ✅ Done | `/` to filter sessions by content across Chats and Tools. |
| **Multi-Project** | ✅ Done | Automatic discovery via `.project_root` scanning. |
| **Cost Estimation** | ✅ Done | Pricing engine for Pro/Flash models + token math. |
| **Analytics (Stats)** | ✅ Done | Project-based drill-down; Interactive Sparklines for token trends. |
| **Timeline View** | ✅ Done | Unified chronological view across all project hashes. |
| **Skill Browser** | ✅ Done | Discover and view system prompts and skill definitions. |
| **MCP Monitor** | ✅ Done | Real-time status of local and remote MCP servers. |
| **Config Health** | ✅ Done | Linting rules for `GEMINI.md`, stale sessions, and costs. |
| **Secret Scanning** | ✅ Done | Entropy-aware background scan of `chats/*.json`. |
| **Security Alerts** | ✅ Done | Red banner notification when critical leaks are found. |
| **Context Browser** | ✅ Done | Integrated viewer for Project and Global `GEMINI.md` (Memory). |
| **Plan Viewer**     | ✅ Done | Access and read implementation plans from `plans/*.md`. |
| **Settings Explorer**| ✅ Done | Browser for Gemini CLI `settings.json` configuration. |
| **Mouse Support**   | ✅ Done | Full scroll support for detail panes and sidebar. |
| **Tool Analytics**  | ✅ Done | Extraction and display of tool arguments and results. |
## 🛠️ Engineering Specs & Edge Cases
### 1. Data Ingestion & State Management
Reliable synchronization with Gemini CLI session logs.
* **Primary Case                  : ** The CLI appends a new message to a session JSON. The TUI detects the update and refreshes the view.
* **Edge Cases & Mitigations      : **
* **Torn Writes / Partial Lines   : ** The file watcher triggers while a large payload is being written.
* *Mitigation                     : * Use buffered reading; verify JSON integrity before updating state. If parsing fails, wait for the next write event.
* **Log Rotation & Deletion       : ** User clears history or logs are rotated.
* *Mitigation                     : * Watch the parent directory. Re-initialize handles on `Rename` or `Remove` events to maintain continuity.
* **Massive Backlog Initialization: ** First run on a directory with hundreds of MBs of logs.
* *Mitigation                     : * Load the "tail" of the logs (e.g., last 50 sessions) for immediate interactivity. Parse the full history in a background thread with a UI progress indicator.
* **Malformed CLI Output          : ** CLI crashes leaving garbage or stack traces in logs.
* *Mitigation                     : * Parser returns `Result::Err`. TUI displays a "⚠️ Corrupted turn" indicator instead of panicking.
### 2. TUI Rendering & User Interaction (`ratatui`)
Robust terminal handling for constrained environments.
*   **Primary Case:** standard 80x24+ terminal window.
*   **Edge Cases & Mitigations:**
    *   **Aggressive Resizing:** Terminal window resized to extremely small dimensions.
        *   *Mitigation:* Implement `Constraint::Min()`. If dimensions fall below minimum viable size, render a "Terminal too small" warning.
    *   **Unicode and Emojis:** Complex characters/CJK spanning multiple cells.
        *   *Mitigation:* Use `unicode-width` for layout calculations to prevent wrapping breakages.
    *   **Unbroken Long Strings:** Massive Base64 payloads or long URLs.
        *   *Mitigation:* Fallback to mid-string word-wrapping if content exceeds pane width to prevent UI clipping.
### 3. Analytics, Cost, & Tokens
Accurate financial and token usage reporting.
*   **Primary Case:** Sessions contain `input_tokens` and `output_tokens` multiplied by current pricing.
*   **Edge Cases & Mitigations:**
    *   **Changing Models Mid-Session:** Switching between Flash and Pro in a single thread.
        *   *Mitigation:* Calculate cost per interaction turn using the specific `model_id` logged for that turn.
    *   **Context Caching Discounts:** Significant price drops for cached tokens.
        *   *Mitigation:* Parse `cached_content_token_count` separately. Display a "Savings" badge in the UI.
    *   **Rate Limits and Error Responses:** API errors instead of token counts.
        *   *Mitigation:* Flag turns with an "API Error" tag; treat missing tokens as 0 for cost while notifying the user of the failure.
### 4. Tool Execution & MCP Integration
Monitoring local functions and external protocol servers.
*   **Primary Case:** Model calls a tool (e.g., `read_file`) and the result is logged.
*   **Edge Cases & Mitigations:**
    *   **Massive Tool Payloads:** Tool reads a multi-MB file.
        *   *Mitigation:* Truncate tool output in the UI (e.g., at 5,000 chars) with a "[Output truncated]" notice to prevent memory spikes.
    *   **Hanging/Zombie Tools:** Shell commands that hang or wait for input.
        *   *Mitigation:* Mark turns as `[Status: Hanging/Timeout]` if a tool invocation lacks a result within a reasonable window.
### 5. Config Health & Secret Scanning
Background security and integrity checks.
*   **Primary Case:** Regex detects an AWS or GitHub key and alerts the user.
*   **Edge Cases & Mitigations:**
    *   **False Positives on Hashes:** Commit SHAs triggering secret alerts.
        *   *Mitigation:* Implement Shannon Entropy checks. Only flag high-entropy strings matching known provider prefix patterns.
    *   **Scanner Freezing the UI:** Regex pipeline on massive logs blocking the main thread.
        *   *Mitigation:* Run scanner in a dedicated background thread communicating via `mpsc` channels.
## 📝 Project To-Do
### 1. UI Infrastructure & Navigation
- [x] **Rail-based Navigation:** 10-view side rail for comprehensive observability.
- [x] **Modular Architecture:** Separated models, parser, and UI views.
- [x] **Global Search:** `/` to search across all sessions and filter the sidebar.
### 2. Analytics & Cost
- [x] **Model Pricing Engine:** Dynamic pricing per model turn.
- [x] **Interactive Visuals:** Sparklines for token trends in the Stats view.
- [ ] **Multi-Mode Sorting:** Sort projects by Cost, Tokens, or Name (currently fixed by date).
### 3. Tool & Metadata Visualization
- [x] **Searchable Tool Rail:** Dedicated view for tool discovery.
- [x] **Context Browser:** Viewer for Project and Global `GEMINI.md`.
- [x] **Skill Browser:** View system prompts and skill metadata.
- [x] **MCP Monitor:** Status of configured MCP servers.
- [x] **Plan Viewer:** Access to `plans/*.md`.
- [x] **Settings Explorer:** Visual browser for `settings.json`.
### 4. Security & Health
- [x] **Secret Scanner:** Entropy-aware background scanning for leaked keys.
- [x] **Health Rules:** Automated checks for missing GEMINI.md, high costs, and stale sessions.
- [x] **Real-time Security Alerts:** Visual banners for critical leaks in active sessions.

## 🚀 Next Steps & Structural Improvements

### 1. Architectural Refactoring
The project's core files (`parser.rs` and `ui.rs`) have grown significantly. To maintain high performance and scalability:
- **`src/parser/` Module:** Decompose the parsing logic into specialized handlers (e.g., `session_handler.rs`, `mcp_handler.rs`, `token_handler.rs`).
- **`src/ui/` Module:** Create a dedicated view-based architecture. Move session rendering to `explorer_view.rs`, and statistics to `stats_view.rs`.

### 2. Enhanced Configuration (Theme Support)
- **Theme Engine:** Implement a `Theme` struct allowing users to customize colors via a `themes.json` file.
- **CLI Arguments:** Integrate `clap` to allow users to specify custom log paths, model pricing, or verbosity levels at startup.

### 3. Reporting & Exports
- **Markdown Export:** Allow users to export a specific AI session or a project-wide summary into a professional Markdown report.
- **CSV/JSON Stats:** Provide raw data exports for advanced cost analysis in external tools.

### 4. Interactive UX
- **Advanced Filtering:** Add date-range filters and model-specific toggles to the main explorer.
- **Session Diffing:** Implement a "diff" view to compare two similar AI turns or sessions.

## 🛠️ Tech Stack Notes
- **Language:** Rust
- **TUI:** [ratatui](https://github.com/ratatui-org/ratatui)
- **Async:** [tokio](https://github.com/tokio-rs/tokio)
- **Markdown:** [termimad](https://github.com/Canop/termimad)
- **ANSI Parsing:** [ansi-to-tui](https://github.com/ratatui-org/ansi-to-tui)
- **File Watching:** [notify](https://github.com/notify-rs/notify)
