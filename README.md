<p align="center">
  <h1 align="center">⚡ Amni-Code</h1>
  <p align="center">
    <b>Your own AI coding agent. One binary. Any LLM. Full IDE.</b><br>
    <em>Open-source, self-hosted, built in Rust — works with xAI, OpenAI, Anthropic, Ollama, or any local model.</em>
  </p>
  <p align="center">
    <a href="https://ko-fi.com/anmire"><img src="https://img.shields.io/badge/Ko--fi-Support%20the%20project-FF5E5B?logo=ko-fi&logoColor=white" alt="Ko-fi"></a>
    <a href="https://github.com/anmire/Amni-Code/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
    <img src="https://img.shields.io/badge/built%20with-Rust-orange?logo=rust" alt="Rust">
    <img src="https://img.shields.io/badge/version-2.8.0-e91e63" alt="Version">
  </p>
</p>

---

> **Think Cursor / Claude Code / GitHub Copilot — but open-source, self-hosted, and zero dependencies.**  
> One Rust binary. No Electron, no Docker, no Node. Just run it.

---

## What is Amni-Code?

Amni-Code is a **self-hosted AI coding agent** with a full embedded IDE. Give an LLM real tools to read, write, edit files, run commands, search the web, and iterate on your code — up to 15 autonomous iterations per request.

### Core Features

| Feature | Description |
|---------|-------------|
| 🔁 **Agentic Tool Loop** | 19 tools — read/write/edit/**multi-edit** files, run commands, search, web fetch/search, memory, **git** (status/diff/add/commit/log), **format/lint/test** |
| 🧭 **Operating Modes** | **Plan** (propose, never mutate) · **Edit** · **Autonomous** — Claude-Code-style control |
| ✅ **Quality Gate** | `run_format` → `run_lint` → `run_tests` (auto-detected cargo/go/py/npm) — write→test→fix loop |
| 🌿 **Git Built-in** | First-class `git_status/diff/add/commit/log` tools — not just shell |
| ⌨️ **Slash Commands** | `/test` `/commit` `/diff` `/review` `/plan` `/edit` `/auto` `/help` |
| 📎 **@-File Mentions** | Type `@path/to/file` to drop its contents straight into context |
| ⏪ **Session Checkpoints** | Snapshot + restore a whole session — conversation *and* files |
| ✂️ **Atomic Multi-Edit** | Apply many find/replace edits to one file all-or-nothing |
| 🧠 **Any LLM Provider** | xAI Grok, OpenAI, Anthropic, Ollama, or any OpenAI-compatible endpoint |
| 📦 **Single Binary** | One Rust binary. No runtime deps. Just `cargo build` and go |
| 🎨 **Full IDE** | Monaco editor with multi-file tabs, breadcrumbs, syntax highlighting for 30+ languages |
| 🎯 **Command Palette** | `Ctrl+Shift+P` — fuzzy search 16 commands, VS Code-style |
| 📂 **Quick Open** | `Ctrl+P` — instant fuzzy file picker |
| 🔍 **Live Diff Panel** | See every change the agent makes — accept or undo with one click |
| 🌐 **Web Tools** | Agent can fetch URLs and search the web (SSRF-protected) |
| 💾 **Persistent Memory** | Agent remembers notes across sessions via memory_read/memory_write |
| 📋 **Per-Project Instructions** | Drop a `.amni-instructions.md` in your project — the agent reads it automatically |
| 👁 **File Watcher** | Sidebar auto-refreshes when files change on disk |
| 🎭 **8 Accent Themes** | Code (pink), Crypt (blue), Haven (purple), AI (amber), Core (red), Explore (cyan), Calc (orange), Green (mint) — dark & light modes |
| ⚡ **Status Bar** | Language, cursor position, version — just like VS Code |
| 🔔 **Toast Notifications** | Non-intrusive popups for saves, actions, and feedback |
| 🛑 **Interrupt & Steer** | Stop the agent mid-generation or inject context while it's thinking |
| 📥 **HuggingFace Downloader** | Search and download GGUF models directly from the UI |
| 🌍 **10 Languages** | EN, ES, FR, DE, JA, ZH, KO, PT, AR, RU — switchable in Settings, RTL support for Arabic |
| 🪟 **Split Panes** | Up to 3 independent agent sessions side-by-side with draggable dividers |
| 📦 **GUI Installer** | WebGL installer (Rust binary) — zero-dependency `install.bat` bootstraps everything from scratch |

## Quick Start

### Option A — One-Click Installer (Windows, zero setup required)

Download and double-click `install.bat`. That's it.

```
install.bat
```

The script auto-installs (if missing):
- **Rust** toolchain via `rustup`
- **Git** via `winget`
- **WebView2 Runtime** (Microsoft Edge component)

Then it compiles and launches the **GUI installer** — a native window with a WebGL interface that guides you through model selection, PATH setup, and desktop shortcut creation. No Python, no Java, no Node required.

```bash
# Or clone first, then run
git clone https://github.com/anmire/Amni-Code.git
cd Amni-Code
install.bat
```

### Option B — Build from Source (all platforms)

```bash
git clone https://github.com/anmire/Amni-Code.git
cd Amni-Code
cargo build --release
./target/release/amni        # Linux/macOS
target\release\amni.exe      # Windows
```

### Option C — Just Run (developer mode)

```bash
.\run.bat   # Windows — auto-pulls, builds if changed, launches
```

### Global CLI

After `install.bat`, the `amni` command works anywhere:
```bash
cd my-project
amni
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+P` | Command Palette |
| `Ctrl+P` | Quick Open (file picker) |
| `Ctrl+S` | Save file (in editor) |
| `Ctrl+Space` | AI suggestion (in editor) |
| `Enter` | Send message (in chat) |
| `Shift+Enter` | New line (in chat) |

## Architecture

```
┌──────────────────────────────────────────────────────┐
│  Native Window (WebView2/wry)                        │
│  ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │Sidebar  │ │  Chat    │ │  Monaco  │ │  Diff    │  │
│  │Explorer │ │  + Tools │ │  Editor  │ │  Panel   │  │
│  │         │ │          │ │  + Tabs  │ │          │  │
│  └─────────┘ └──────────┘ └──────────┘ └──────────┘  │
│  ┌──────────────────────────────────────────────────┐│
│  │  Status Bar │ Language │ Ln/Col │ v2.2.0         ││
│  └──────────────────────────────────────────────────┘│
└───────────────────────┬──────────────────────────────┘
                        │ SSE streaming
┌───────────────────────▼──────────────────────────────┐
│  Rust Backend (Axum + Tokio)                         │
│  ┌─────────────────────────────────────────────────┐ │
│  │  Agent Loop — up to 15 iterations               │ │
│  │  Tools: read/write/edit_file, run_command,      │ │
│  │  list_directory, search_files, web_fetch,       │ │
│  │  web_search, memory_read, memory_write          │ │
│  └─────────────────────────────────────────────────┘ │
│  Per-project instructions · File watcher · Memory    │
└───────────────────────┬──────────────────────────────┘
                        │ OpenAI-compatible API
┌───────────────────────▼──────────────────────────────┐
│  LLM Provider                                        │
│  xAI · OpenAI · Anthropic · Ollama · Any local server│
└──────────────────────────────────────────────────────┘
```

## Configuration

Click **⚙ Settings** in the UI:

| Setting | Description |
|---------|-------------|
| **Provider** | xAI, OpenAI, Anthropic, Ollama, or custom endpoint |
| **Model** | Auto-populated from provider |
| **API Key** | Auto-detected from `.env` or env vars |
| **Working Directory** | Where the agent operates |
| **Auto-approve** | Confirm each action vs. fully autonomous |

### API Keys

```bash
# .env file or environment variables
XAI_API_KEY=xai-...          # xAI Grok (default)
OPENAI_API_KEY=sk-...        # OpenAI
ANTHROPIC_API_KEY=sk-ant-... # Anthropic
```

### Per-Project Instructions

Drop any of these in your project root — they're automatically loaded into the agent's system prompt:

- `.amni-instructions.md`
- `.github/copilot-instructions.md`
- `AGENTS.md`

## Hardware

| GPU | Framework | Notes |
|-----|-----------|-------|
| **NVIDIA** | CUDA 12.0+ | Full acceleration |
| **AMD** | HIP/ROCm 7.1+ | 7000-series supported |
| **CPU** | — | 16GB+ RAM recommended for local models |

## Requirements

- **OS**: Windows 10/11, Linux, macOS (build from source)
- **RAM**: 8GB min, 16GB+ for local models
- **Rust**: 1.70+ (auto-installed by scripts)

## Project Structure

```
Amni-Code/
├── src/main.rs           # Rust backend — server, agent loop, 12 tools, LLM routing
├── static/index.html     # Embedded UI — chat, Monaco, diff, i18n, split panes
├── Cargo.toml            # Dependencies
├── install.bat           # Zero-dependency installer — bootstraps Rust/Git/WebView2/GUI
├── run.bat               # Developer launcher — auto-pull, incremental build, launch
├── installer/            # Rust GUI installer crate (tao+wry+axum, WebGL UI)
│   ├── src/main.rs       # Axum server + wry window + prereq detection + HF downloads
│   └── static/installer.html  # WebGL 5-page install wizard
├── packaging/
│   ├── windows/amni-code.wxs  # WiX MSI manifest
│   ├── macos/build-dmg.sh     # macOS .app + DMG
│   └── linux/build-deb.sh     # Linux .deb package
└── changelog.md          # Version history
```

## Contributing

1. Fork → branch → PR. All contributions welcome.
2. Ideas: edit-mode approval UI, MCP client, LSP/inline diagnostics, semantic codebase index, Linux/macOS installers, multimodal.

## Support

<a href="https://ko-fi.com/anmire"><img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Support on Ko-fi"></a>

## License

MIT — see [LICENSE](LICENSE).
