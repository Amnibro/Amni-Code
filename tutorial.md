# Amni-Code Tutorial

Amni-Code is an open-source AI coding agent built in Rust. It gives an LLM real tools to read, write, and edit files, run shell commands, and search your codebase — all from a browser UI at `localhost:3030`.

## One-Click Install (Windows)

The fastest way to get started:

```bash
# Download and run the quickstart script
curl -L -o quickstart.bat https://raw.githubusercontent.com/Amnibro/Amni-Code/main/quickstart.bat
quickstart.bat
```

This will:
1. Clone the repository
2. Prompt you for API keys (xAI Grok, OpenAI, Anthropic — press Enter to skip any)
3. Install Rust if needed
4. Build the application
5. Launch it at http://localhost:3030

### Already cloned? Just run:
```bash
cd Amni-Code
quickstart.bat
```

## GUI Installer

For a guided experience with progress bars and hardware detection:

```bash
git clone https://github.com/Amnibro/Amni-Code.git
cd Amni-Code
install.bat
```

The GUI installer provides:
- Real-time progress tracking
- GPU detection (NVIDIA CUDA / AMD HIP/ROCm)
- API key entry fields for xAI, OpenAI, and Anthropic
- Desktop shortcut creation
- One-click launch after install

## Manual Build

```bash
git clone https://github.com/Amnibro/Amni-Code.git
cd Amni-Code
cargo build --release
target\release\amni.exe   (Linux/macOS: ./target/release/amni)
```

## Setting Up API Keys

Amni-Code defaults to **Adam / Amni-AI on your own machine** (`http://127.0.0.1:7700`) and also finds Ollama (`:11434`), llama.cpp (`:8080`) and LM Studio (`:1234`) automatically. **No API key is needed for local models.** Keys are only for the optional cloud providers.

### Fully offline
1. Start a local model server, e.g. `ollama serve`, then `ollama pull hf.co/ibm-granite/granite-4.2-3b-GGUF` (small, 128K context) or `ollama pull hf.co/unsloth/Qwen3.8-27B-GGUF` (stronger, 262K context). Do the pull while you still have internet.
2. Launch `amni --offline` (or press `Ctrl+Shift+O` in the app, or tick **Offline mode** in Settings).
3. Press `Ctrl+K` and pick a model under **Local · works offline**.

In Offline mode cloud providers, `web_search`, `web_fetch`, Hugging Face downloads and the start-up update check are switched off and say so straight away instead of timing out. The whole UI, including the Monaco editor, is served from the binary.

### During Installation
The quickstart script and GUI installer both prompt for API keys. They're saved to a `.env` file (gitignored).

### After Installation
1. Open http://localhost:3030
2. Click the **⚙ Settings** button (top right)
3. Select your **Provider** (Adam, Ollama, custom local endpoint, Anthropic, OpenAI, Google Gemini, xAI) — or press `Ctrl+K` for the quick picker
4. Enter your **API Key** (cloud providers only)
5. Close the settings panel — changes save automatically

### Manual `.env` File
Create a `.env` file in the Amni-Code directory:
```
ANTHROPIC_API_KEY=your-key-here
OPENAI_API_KEY=your-key-here
GEMINI_API_KEY=your-key-here
XAI_API_KEY=your-key-here
```

## Using Amni-Code

### Chat Interface
Type anything in the text box and press Enter. The agent will:
- Read files to understand your codebase
- Write or edit files to implement changes
- Run shell commands (build, test, install)
- Search across your project for relevant code
- Iterate until the task is done (cap: `AMNI_CODE_MAX_ITERS`), showing each tool call live as it runs

### Example Prompts

**Build something:**
```
Create a Python Flask API with user authentication and a SQLite database.
```

**Debug code:**
```
Find and fix the bug in main.py — the server crashes on startup.
```

**Explore a codebase:**
```
Explain how this project is structured and what each module does.
```

**Scaffold a project:**
```
Set up a new React project with TypeScript, ESLint, and Tailwind CSS.
```

### Code Diff Panel
Click the **diff icon** (top right) to see every file change the agent makes. Shows added/removed lines for each file.

### Auto-Approve Mode
In Settings, toggle **Auto-approve all agent actions**:
- **Off** (default): You see what the agent wants to do before it executes
- **On**: The agent acts fully autonomously

### Working Directory
The agent operates in the directory shown in Settings. Change it to point at any project on your machine.

## Supported Providers

| Provider | Default / suggested models | Key | Offline |
|----------|-----------------------------|-----|---------|
| **Adam / Amni-AI** (default) | `adam:3b`, `adam:8b` (live list from `/v1/models`) | none | yes |
| **Ollama** | live list from `/api/tags`; suggested Granite 4.2 3B, Qwen 3.8 27B | none | yes |
| **Custom local** | live list from `/v1/models` (llama.cpp, LM Studio, vLLM…) | optional | yes |
| **Anthropic** | `claude-opus-5-5`, `claude-sonnet-5-5`, `claude-haiku-5-5`, `claude-fable-5-2` | `ANTHROPIC_API_KEY` | no |
| **OpenAI** | `gpt-6-astra`, `gpt-6.1-sol`, `gpt-6-luna`, `gpt-6.1-terra` | `OPENAI_API_KEY` | no |
| **Google Gemini** | `gemini-3.8-flash`, `gemini-3.6-flash`, `gemini-3.5-flash-lite`, `gemini-3.1-pro-preview` | `GEMINI_API_KEY` | no |
| **xAI** | `grok-4.7`, `grok-4.6`, `grok-4.5`, `grok-build-0.1` | `XAI_API_KEY` | no |

With a key set, the model list comes live from the provider's own models endpoint; the ids above are the fallback when it cannot be reached.

## Troubleshooting

**Port 3030 busy:** Another app is using port 3030. Kill it, or set `AMNI_CODE_PORT` to a free port and relaunch. (Haven uses **3000** — Amni-Code intentionally does not.)

**Build fails:** Make sure Rust is installed: `rustup --version`. If not, run `quickstart.bat` or install from https://rustup.rs.

**"Request failed" error:** Your API key may be invalid or the provider is unreachable. Check Settings and verify your key.

**"Offline mode is on…" message:** a cloud provider is selected while Offline mode is on. Press `Ctrl+K` and pick a local model, or turn Offline mode off.

**Headless / remote box:** run `amni --serve` and open `http://localhost:3030` in any browser.

**Ollama not responding:** Make sure Ollama is running (`ollama serve`) and accessible at `http://localhost:11434`.

## Contributing

See [README.md](README.md#contributing) for contribution guidelines.

## License

MIT — see [LICENSE](LICENSE).
