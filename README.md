<p align="center">
  <h1 align="center">⚡ Amni-Code</h1>
  <p align="center">
    <b>Your own AI coding agent. One binary. Any LLM. Full IDE.</b><br>
    <em>Open-source, self-hosted, built in Rust — local-first and fully offline-capable with Adam / Ollama / any local model, plus Anthropic, OpenAI, Google Gemini and xAI when you want the cloud.</em><br>
    <a href="https://amni-scient.com/amni-code">amni-scient.com/amni-code</a> · <a href="https://github.com/Amnibro/Amni-Code">github.com/Amnibro/Amni-Code</a>
  </p>
  <p align="center">
    <a href="https://ko-fi.com/amnibro"><img src="https://img.shields.io/badge/Ko--fi-Support%20the%20project-FF5E5B?logo=ko-fi&logoColor=white" alt="Ko-fi"></a>
    <a href="https://github.com/Amnibro/Amni-Code/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
    <img src="https://img.shields.io/badge/built%20with-Rust-orange?logo=rust" alt="Rust">
    <img src="https://img.shields.io/badge/version-3.0.0-e91e63" alt="Version">
  </p>
</p>

---

> **Think Cursor / Claude Code / GitHub Copilot — but open-source, self-hosted, and zero dependencies.**  
> One Rust binary. No Electron, no Docker, no Node. Just run it.

---

## What is Amni-Code?

Amni-Code is a **self-hosted AI coding agent** with a full embedded IDE. Give an LLM real tools to read, write, edit files, run commands, search the web, and iterate on your code. It runs entirely on your machine against a local model when you want it to — no internet required.

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
| 🧠 **Any LLM Provider** | Adam / Amni-AI (local), Ollama, any OpenAI-compatible local server (llama.cpp, LM Studio), Anthropic, OpenAI, Google Gemini, xAI. Model lists are fetched live from each provider's models endpoint (cached 10 min) with a small confirmed fallback list |
| ✈️ **Offline mode** | Works with no internet: every UI asset (Monaco editor, fonts, icons) is embedded in the binary; local model servers are auto-detected at startup; Offline mode (`--offline`, `AMNI_OFFLINE=1`, `Ctrl+Shift+O`) turns off cloud providers, `web_search`/`web_fetch`, model downloads and update checks with a clear message instead of a timeout |
| 🪶 **Small-model friendly** | Small local models (e.g. Granite 4.2 3B) get a compact tool set and a plain-text `<tool_call>` protocol they can follow; Granite's `<function=…><parameter=…>` format and Hermes/Qwen JSON tool calls are both parsed |
| 🎫 **Your subscriptions, no API key** | Claude Code, Codex CLI (ChatGPT), GitHub Copilot CLI, Cursor CLI, Grok CLI, Antigravity, OpenCode — pick one in Settings, sign in once in that CLI, and it drives your project folder directly |
| 📝 **Rich answers** | Headings, tables, nested lists, task lists, blockquotes, and fenced code with language bar, copy button, and syntax highlight, set in the amni-scient.com type (Archivo, Source Serif 4, Cascadia Code) |
| 📦 **Single Binary** | One Rust binary. No runtime deps. Just `cargo build` and go |
| 🎨 **Full IDE** | Monaco editor (bundled, no CDN) with multi-file tabs, breadcrumbs, syntax highlighting for 30+ languages |
| 🎛 **Model picker** | `Ctrl+K` — local servers found on this machine, suggested local models with a one-click `ollama pull` command, and cloud models, each marked offline-capable or not |
| 🧵 **Live tool timeline** | Each tool call shows up as it runs (name, argument, status, output preview) |
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
git clone https://github.com/Amnibro/Amni-Code.git
cd Amni-Code
install.bat
```

### Option B — Build from Source (all platforms)

```bash
git clone https://github.com/Amnibro/Amni-Code.git
cd Amni-Code
cargo build --release
./target/release/amni        # Linux/macOS
target\release\amni.exe      # Windows
```

### Option C — Just Run (developer mode)

```bash
.\run.bat   # Windows — auto-pulls, builds if changed, launches
```

### Option D — Headless / server only

```bash
amni --serve [project-dir]          # no native window, no browser pop; open http://localhost:3030
amni --serve --offline [project-dir] # same, with Offline mode forced on
```

`AMNI_CODE_PORT` (default 3030) and `AMNI_CODE_HOST` (default 0.0.0.0; use 127.0.0.1 to keep it on this machine) set the listen address. `AMNI_NO_SELF_UPDATE=1` or `--no-update` skips the start-up `git pull` (it is also skipped in `--serve`/`--offline` and always gives up after 5 s).

## Local models (works offline)

Amni-Code probes these at start-up and shows what it finds in the model picker:

| Server | Address | API used |
|--------|---------|----------|
| Adam / Amni-AI | `http://127.0.0.1:7700` | OpenAI `/v1/*` (models via `/v1/models`, Ollama `/api/tags` also read) |
| Ollama | `http://127.0.0.1:11434` | OpenAI-compatible `/v1/chat/completions`, models via `/api/tags` |
| llama.cpp server / LM Studio | `:8080` / `:1234` | OpenAI `/v1/*` |

Suggested local models (released Aug 2026):

| Model | Pull | Context | Tool calling |
|-------|------|---------|--------------|
| IBM Granite 4.2 3B (`ibm-granite/granite-4.2-3b`) | `ollama pull hf.co/ibm-granite/granite-4.2-3b-GGUF` | 128K native (512K extended) | `<tool_call><function=…><parameter=…>`; Amni-Code uses its compact text tool protocol for it |
| Qwen 3.8 27B (`Qwen/Qwen3.8-27B`) | `ollama pull hf.co/unsloth/Qwen3.8-27B-GGUF` | 262K native (1M with YaRN) | native tools; thinking on by default (`<think>` is stripped from answers) |

Set `AMNI_TOOL_MODE=prompt` (or `tool_mode: "prompt"` in `~/.amni/config.json`) to force the text tool protocol for any model, `native` to force OpenAI-style tool calls.

### Global CLI

After `install.bat`, the `amni` command works anywhere:
```bash
cd my-project
amni
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Model / provider picker |
| `Ctrl+Shift+O` | Toggle Offline mode |
| `Ctrl+Shift+L` | Toggle light / dark |
| `Ctrl+Shift+N` | New chat |
| `Ctrl+,` | Settings |
| `?` / `/` | Shortcut list / focus the prompt |
| `Esc` | Close panels, stop generation |
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
│  │  Status Bar │ Language │ Ln/Col │ v3.0.0         ││
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
│  Adam · Ollama · llama.cpp · Anthropic · OpenAI ·    │
│  Gemini · xAI   (src/providers.rs)                   │
└──────────────────────────────────────────────────────┘
```

## Configuration

Click **⚙ Settings** in the UI:

| Setting | Description |
|---------|-------------|
| **Provider** | Adam, xAI, OpenAI, Anthropic, Google, Ollama, custom endpoint, or a subscription CLI (Claude Code, Codex, Copilot, Cursor, Grok, Antigravity, OpenCode) |
| **Model** | Auto-populated from provider |
| **API Key** | Auto-detected from `.env` or env vars |
| **Working Directory** | Where the agent operates |
| **Auto-approve** | Confirm each action vs. fully autonomous |

### API Keys

```bash
# .env file or environment variables
# None are needed for local models (Adam / Ollama), which are the default.
ANTHROPIC_API_KEY=...        # Anthropic  (default model claude-opus-5-5)
OPENAI_API_KEY=...           # OpenAI     (default model gpt-6-astra)
GEMINI_API_KEY=...           # Google     (default model gemini-3.8-flash, OpenAI-compatible endpoint)
XAI_API_KEY=...              # xAI        (default model grok-4.7)
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
- **Rust**: 1.80+ (auto-installed by scripts)
- **Linux build deps**: `libwebkit2gtk-4.1-dev libgtk-3-dev` (for the native window; `--serve` runs without opening one)

## Project Structure

```
Amni-Code/
├── src/main.rs           # Rust backend — server, agent loop, tools
├── src/providers.rs      # Provider table, request shaping (OpenAI / Anthropic), live model lists, local-server detection
├── build.rs              # Embeds static/vendor/** (Monaco) into the binary
├── static/ui.js, ui.css  # 3.0 UI layer: model picker, offline mode, tool timeline, shortcuts
├── static/vendor/monaco/ # Monaco editor 0.44.0 (MIT), served from the binary
├── tests/e2e/            # Offline end-to-end test (stub model servers + Playwright in a loopback-only netns)
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

<a href="https://ko-fi.com/amnibro"><img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Support on Ko-fi"></a>

## License

MIT — see [LICENSE](LICENSE).
