# Amni-Code Architecture Map v2.5.0
## Overview
Single-binary AI coding agent. Rust/Axum backend with embedded HTML UI. No Python/Node/npm runtime deps. Per-session cwd for concurrent windows in diff dirs, no max-iters. **v2.5.0 renders Adam's inline widgets** (weather/system/news/stock/git/file/etc from Amni-Ai v6.10+) as themed cards in the assistant message. **v2.4.0 wired Adam (Amni-Ai v6.9.4 GF(17) 2B) as a first-class autonomous brain** via OpenAI `/v1/chat/completions` on `127.0.0.1:7700`. Stale `:8001` / `:8002` saved URLs auto-remap to `:7700` (in addition to the prior `:8787` migration). Same 10 OpenAI tool-call functions, zero wire-shape changes on this side. v2.2.3 fixes Amni default startup by targeting the Amni-AI web server on port 7700, normalizing stale saved Amni base URLs, and auto-starting the local server immediately on initial config load.

## Widget rendering (v2.5.0)
- `agent_loop_stream` extracts `raw_msg.amni_widgets[]` from each LLM response and emits one `widget` SSE event per item before the `message` / `tool_*` events.
- Frontend (`static/index.html`): `case 'widget'` collects into `allWidgets[]`, then `addMsg('assistant', content, tools, widgets)` calls `renderWidget(w)` for each item and appends themed HTML cards to the assistant message bottom.
- Card CSS uses `--accent`, `--accent-dim`, `--accent-glow`, `--border` from the current theme — cards in pink theme are pink, in haven purple, etc. No theme mismatch.
- 12 widget types rendered: weather (temp + meta), system (CPU/MEM/DISK/GPU grid), time, news (clickable list with sources), stock (colored ▲/▼ change), code/file (mono pre), git (branch + commits + dirty/ahead/behind), error/info (generic).

## Adam integration (v2.4.0)
- Provider `amni` → POSTs to `${amni_base_url}/v1/chat/completions` with `messages[]` + `tools[]` + `tool_choice:"auto"`.
- Adam returns `choices[0].message.tool_calls[]` in standard OpenAI shape; the existing agent loop in `agent_loop_stream` handles them unchanged.
- Cross-session learning: each successful (intent → tool sequence) pair is written to Adam's PTEX `CodeAtlas` (cell-address LUT). When Amni-Code sends a similar task later, Adam recalls the prior tool sequence as a hint in the system prompt.
- Privacy gate (Adam side): SSN/email/phone/auth-key patterns in tool arguments block the cell from federating to `__global__`. Personal sessions stay session-local.
## Core Files
src/main.rs: Axum web server + full agent engine
- App state: sessions (HashMap w/ working_dir), config, cwd
- 6 tools: read_file,write_file,edit_file,run_command,list_directory,search_files
- Agent loop: unbounded (soft<100) LLM call->tool->repeat until no tools
- UI header themed cwd display
- LLM integration: OpenAI-compatible API (Ollama/OpenAI/Anthropic/xAI/Google/Amni via provider config)
- Routes: GET / (serve embedded UI), POST /api/chat (agent), GET|POST /api/config, GET /health
- Auto-opens browser on launch
static/index.html: Self-contained single-file UI (HTML+CSS+JS inline)
- Chat interface with markdown rendering
- Code-change diff visualizer (side panel, auto-opens on file changes)
- Settings panel (provider/model/key/url/working_dir/auto_approve toggle + language selector)
- Welcome/onboarding screen with quick-start prompts
- Tool execution badges with expandable details
- i18n system: I18N dict (en/es/fr/de/ja/zh/ko/pt/ar/ru), t(key) function, data-i18n attributes, RTL for Arabic
- Split pane system: panes[] array (max 3), createPane/closePane/paneSend, draggable dividers, independent SSE sessions per pane
Cargo.toml: Rust deps — axum 0.7, tokio, serde, reqwest, uuid, open, futures, tokio-stream
run.bat: One-click launcher — checks Rust, builds if needed, runs binary
install_gui.py: tkinter GUI installer (v2.2.0) — 8-step wizard, API key setup, model picker w/ checkboxes, PATH setup, desktop shortcut
## Packaging
packaging/windows/amni-code.wxs: WiX XML manifest for MSI — perUser install, PATH env, Start Menu + desktop shortcuts
packaging/macos/build-dmg.sh: Creates .app bundle (Info.plist, binary) + DMG via hdiutil
packaging/linux/build-deb.sh: Creates .deb package (DEBIAN/control, /usr/local/bin/amni, .desktop file)
packaging/build-all.sh: OS-detection wrapper — runs cargo build then platform-specific packager
## Rust Installer (installer/)
installer/Cargo.toml: Standalone Rust binary — tao+wry+axum (same stack as main app), reqwest for HTTP model downloads
installer/src/main.rs: Axum server + tao/wry window, routes: GET / (UI), GET /api/prereqs (detect Rust/Git/OS), POST /api/install (SSE progress stream), POST /api/open (launch download URLs)
installer/static/installer.html: WebGL plasma shader background, 5-page wizard (Welcome/Prereqs/Options/Install/Done), glass morphism UI, model picker, PATH setup, two modes (pre-built binary download OR build from source)
Eliminates Python dependency entirely — model downloads use HuggingFace HTTP API via reqwest, no huggingface-cli needed
## Data Flow
User input -> POST /api/chat -> agent_loop -> llm_call (OpenAI-compatible or Google direct) -> tool execution -> repeat until LLM returns text-only -> JSON response with message + tool_calls array
## Config Providers
- ollama: localhost:11434 (default, no key needed, import GGUF via "from" field)
- amni: 127.0.0.1:7700 (Amni-AI web server, exposes /health, /v1/models, /v1/chat/completions)
- openai: api.openai.com (Bearer token)
- anthropic: api.anthropic.com (x-api-key header)
- xai: api.x.ai (Bearer token)
- google: generativelanguage.googleapis.com (Bearer token, direct generateContent API)
## Backups
backups/v1.0.0/: main.rs.bak, index.html.bak, run.bat.bak, Cargo.toml.bak
backups/v2.2.0/: main.rs.bak, index.html.bak, install_gui.py.bak, Cargo.toml.bak
backups/src_main.rs.v2.2.1.bak: Ollama import fix
backups/src_main.rs.v2.2.2.bak: Google provider addition
backups/src_main.rs.v2.2.3_amni_server_port_fix.bak: Amni default startup + port fix
backups/static_index.html.v2.2.3_amni_default_start_fix.bak: Amni immediate auto-start UI fix
