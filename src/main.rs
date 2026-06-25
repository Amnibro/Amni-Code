use axum::{extract::{Query,State},response::{sse::{Event as SseEvent,KeepAlive,Sse},Html,Json},routing::{get,post},Router};use futures::StreamExt;use serde::{Deserialize,Serialize};use std::{collections::HashMap,convert::Infallible,path::{Path,PathBuf},process::Stdio,sync::{Arc,atomic::AtomicBool}};use tokio::sync::{Mutex,broadcast};use tokio_stream::wrappers::ReceiverStream;use tower_http::cors::CorsLayer;
#[derive(Clone, Default, Serialize)]
struct DownloadProgress {
    repo: String,
    file: String,
    downloaded: u64,
    total: u64,
    done: bool,
    error: String,
}
#[derive(Clone)]struct App{sessions:Arc<Mutex<HashMap<String,Session>>>,config:Arc<Mutex<Config>>,cwd:Arc<Mutex<PathBuf>>,dl_progress:Arc<Mutex<DownloadProgress>>,backups:Arc<Mutex<HashMap<String,Vec<(String,Option<String>)>>>>,interrupts:Arc<Mutex<HashMap<String,Arc<AtomicBool>>>>,amni_proc:Arc<Mutex<Option<tokio::process::Child>>>,server_log_tx:Arc<broadcast::Sender<String>>,memory:Arc<Mutex<HashMap<String,Vec<String>>>>,fs_tx:Arc<broadcast::Sender<String>>}
#[derive(Clone,Default)]struct Session{messages:Vec<serde_json::Value>,working_dir:String}
#[derive(Clone, Serialize, Deserialize)]
struct Config {
    provider: String,
    model: String,
    api_key: String,
    base_url: String,
    auto_approve: bool,
    #[serde(default)]
    mode: String,
    working_dir: String,
    model_dir: String,
}
impl Default for Config {
    fn default() -> Self {
        let xai_key = [
            "XAI_API_KEY",
            "GROK_key",
            "xAI_key",
            "GROK_API_KEY",
            "XAI_KEY",
        ]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
        .unwrap_or_default();
        let openai_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        let google_key = ["GOOGLE_AI_API_KEY", "GEMINI_API_KEY", "GOOGLE_API_KEY"]
            .iter()
            .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
            .unwrap_or_default();
        let deepseek_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
        let groq_key = std::env::var("GROQ_API_KEY").unwrap_or_default();
        let mistral_key = ["MISTRAL_API_KEY", "MISTRAL_KEY"]
            .iter()
            .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
            .unwrap_or_default();
        let cohere_key = std::env::var("COHERE_API_KEY").unwrap_or_default();
        let together_key = std::env::var("TOGETHER_API_KEY").unwrap_or_default();
        let fireworks_key = std::env::var("FIREWORKS_API_KEY").unwrap_or_default();
        let perplexity_key = std::env::var("PERPLEXITY_API_KEY").unwrap_or_default();
        let openrouter_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
        let forced = std::env::var("AMNI_CODE_PROVIDER").ok().filter(|v| !v.is_empty());
        let (provider, key): (&str, String) = match forced.as_deref() {
            Some("xai") => ("xai", xai_key),
            Some("openai") => ("openai", openai_key),
            Some("anthropic") => ("anthropic", anthropic_key),
            Some("google") => ("google", google_key),
            Some("deepseek") => ("deepseek", deepseek_key),
            Some("groq") => ("groq", groq_key),
            Some("mistral") => ("mistral", mistral_key),
            Some("cohere") => ("cohere", cohere_key),
            Some("together") => ("together", together_key),
            Some("fireworks") => ("fireworks", fireworks_key),
            Some("perplexity") => ("perplexity", perplexity_key),
            Some("openrouter") => ("openrouter", openrouter_key),
            Some("ollama") => ("ollama", String::new()),
            Some("local") => ("local", String::new()),
            _ => ("amni", String::new()),
        };
        let (model, base_url) = match provider {
            "openai" => ("gpt-4o".to_string(), "https://api.openai.com".to_string()),
            "anthropic" => (
                "claude-sonnet-4-20250514".to_string(),
                "https://api.anthropic.com".to_string(),
            ),
            "google" => (
                "gemini-2.5-flash".to_string(),
                "https://generativelanguage.googleapis.com".to_string(),
            ),
            "deepseek" => (
                "deepseek-chat".to_string(),
                "https://api.deepseek.com".to_string(),
            ),
            "groq" => (
                "llama-3.3-70b-versatile".to_string(),
                "https://api.groq.com".to_string(),
            ),
            "mistral" => (
                "mistral-large-latest".to_string(),
                "https://api.mistral.ai".to_string(),
            ),
            "cohere" => (
                "command-r-plus".to_string(),
                "https://api.cohere.com".to_string(),
            ),
            "together" => (
                "meta-llama/Llama-3.3-70B-Instruct-Turbo".to_string(),
                "https://api.together.xyz".to_string(),
            ),
            "fireworks" => (
                "accounts/fireworks/models/llama-v3p3-70b-instruct".to_string(),
                "https://api.fireworks.ai".to_string(),
            ),
            "perplexity" => (
                "sonar-pro".to_string(),
                "https://api.perplexity.ai".to_string(),
            ),
            "openrouter" => (
                "anthropic/claude-sonnet-4".to_string(),
                "https://openrouter.ai".to_string(),
            ),
            "amni" => ("adam:granite-gf17".to_string(), "http://127.0.0.1:7700".to_string()),
            "ollama" => (String::new(), "http://localhost:11434".to_string()),
            "local" => (String::new(), "http://localhost:11434".to_string()),
            _ => (
                "grok-4-1-fast-reasoning".to_string(),
                "https://api.x.ai".to_string(),
            ),
        };
        let model_dir = std::env::current_dir()
            .unwrap_or_default()
            .parent()
            .unwrap_or(&std::env::current_dir().unwrap_or_default())
            .join("Amni-Ai")
            .join("models")
            .to_string_lossy()
            .to_string();
        let model_dir = if std::fs::metadata(&model_dir).is_ok() {
            model_dir
        } else {
            String::new()
        };
        Self {
            provider: provider.into(),
            model,
            api_key: key,
            base_url,
            auto_approve: false,
            mode: "edit".into(),
            working_dir: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            model_dir,
        }
    }
}
fn amni_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.is_empty() || trimmed.contains(":8787") || trimmed.contains(":8001") || trimmed.contains(":8002") {
        "http://127.0.0.1:7700".to_string()
    } else {
        trimmed.to_string()
    }
}
fn normalize_config(cfg: &mut Config) {
    cfg.base_url = if cfg.provider == "amni" {
        amni_base_url(&cfg.base_url)
    } else {
        cfg.base_url.trim_end_matches('/').to_string()
    };
}
fn persist_config(cfg: &Config) {
    let config_path = dirs::home_dir()
        .map(|h| h.join(".amni").join("config.json"))
        .unwrap_or_default();
    if let Ok(json) = serde_json::to_string(cfg) {
        let _ = std::fs::create_dir_all(config_path.parent().unwrap());
        let _ = std::fs::write(&config_path, json);
    }
}
const TOOLS_JSON: &str = r#"[
  {"type":"function","function":{"name":"read_file","description":"Read a file's contents","parameters":{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}}},
  {"type":"function","function":{"name":"write_file","description":"Write content to a file (creates or overwrites)","parameters":{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}}},
  {"type":"function","function":{"name":"edit_file","description":"Replace a specific string in a file","parameters":{"type":"object","properties":{"path":{"type":"string"},"old_string":{"type":"string"},"new_string":{"type":"string"}},"required":["path","old_string","new_string"]}}},
  {"type":"function","function":{"name":"run_command","description":"Run a shell command","parameters":{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}}},
  {"type":"function","function":{"name":"list_directory","description":"List files in a directory","parameters":{"type":"object","properties":{"path":{"type":"string"}}}}},
  {"type":"function","function":{"name":"search_files","description":"Search for text across files","parameters":{"type":"object","properties":{"query":{"type":"string"},"path":{"type":"string"}},"required":["query"]}}},
  {"type":"function","function":{"name":"web_fetch","description":"Fetch a webpage URL and return its text content (HTML stripped). Use for documentation, GitHub repos, articles.","parameters":{"type":"object","properties":{"url":{"type":"string","description":"Full URL (http/https)"},"max_chars":{"type":"integer","description":"Max chars to return (default 6000)"}},"required":["url"]}}},
  {"type":"function","function":{"name":"web_search","description":"Search the web via DuckDuckGo. Returns titles, URLs, snippets.","parameters":{"type":"object","properties":{"query":{"type":"string","description":"Search query"},"max_results":{"type":"integer","description":"Max results (default 5, max 10)"}},"required":["query"]}}},
  {"type":"function","function":{"name":"memory_read","description":"Read persistent memory notes. Returns stored facts/preferences for a key.","parameters":{"type":"object","properties":{"key":{"type":"string","description":"Memory key to read (e.g. 'project_notes', 'preferences')"}},"required":["key"]}}},
  {"type":"function","function":{"name":"memory_write","description":"Write to persistent memory. Stores facts/preferences that persist across sessions.","parameters":{"type":"object","properties":{"key":{"type":"string","description":"Memory key"},"content":{"type":"string","description":"Content to store (appended to existing)"}},"required":["key","content"]}}},
  {"type":"function","function":{"name":"git_status","description":"Show git status: current branch plus changed/staged files (short format). Use this to see what you've modified.","parameters":{"type":"object","properties":{}}}},
  {"type":"function","function":{"name":"git_diff","description":"Show the git diff of unstaged changes, optionally for a single path. Review changes before committing.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"Optional file or dir to diff"}}}}},
  {"type":"function","function":{"name":"git_add","description":"Stage changes for the next commit. 'path' defaults to '.' (everything).","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File or dir to stage (default '.')"}}}}},
  {"type":"function","function":{"name":"git_commit","description":"Commit staged changes with a message. Run git_add first to stage. Requires a non-empty message.","parameters":{"type":"object","properties":{"message":{"type":"string","description":"Commit message"}},"required":["message"]}}},
  {"type":"function","function":{"name":"git_log","description":"Show recent commits in oneline format.","parameters":{"type":"object","properties":{"count":{"type":"integer","description":"How many commits to show (default 15)"}}}}},
  {"type":"function","function":{"name":"run_tests","description":"Run the project's test suite (auto-detects cargo/go/pytest/npm from the working dir; pass 'command' to override). Returns pass/fail + output. ALWAYS run after changing code and fix failures before finishing.","parameters":{"type":"object","properties":{"command":{"type":"string","description":"Optional explicit test command instead of auto-detect"}}}}},
  {"type":"function","function":{"name":"run_lint","description":"Run the project's linter (auto-detects clippy/go vet/ruff/eslint from the working dir; pass 'command' to override). Surfaces code-quality and correctness diagnostics. Run after edits alongside run_tests.","parameters":{"type":"object","properties":{"command":{"type":"string","description":"Optional explicit lint command instead of auto-detect"}}}}}
]"#;
const SYSTEM_PROMPT:&str="You are Amni-Code,an expert AI coding agent.Your working dir is:{CWD}\n\nCRITICAL:chain actions,NEVER stop after 1 tool,explore w/tools first,read key files before edits,verify after changes,concise,base paths on cwd,fix errors+retry.\n\nTools:read_file,write_file,edit_file,run_command,list_directory,search_files,web_fetch,web_search,memory_read,memory_write,git_status,git_diff,git_add,git_commit,git_log,run_tests,run_lint\n\nVerify:after editing code,call run_tests (and run_lint for code quality) and fix failures before reporting done (write->test->fix->retest until green;never claim success on a red suite).\n\nGit:prefer the first-class git_* tools over raw run_command for version control;after a meaningful set of edits offer to git_add+git_commit with a clear message;use git_status/git_diff to review before committing.\n\nSupport /interrupt (stop gen) & steering (append mid-gen context).\n\n{CUSTOM_INSTRUCTIONS}";
#[derive(Deserialize)]struct ChatReq{message:String,session_id:Option<String>,working_dir:Option<String>}
#[derive(Serialize)]
struct ChatRes {
    session_id: String,
    message: String,
    tool_calls: Vec<ToolCallResult>,
    done: bool,
}
#[derive(Serialize, Clone)]
struct ToolCallResult {
    tool: String,
    input: serde_json::Value,
    output: String,
    status: String,
}
#[derive(Deserialize)]
struct ConfigReq {
    provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    auto_approve: Option<bool>,
    mode: Option<String>,
    working_dir: Option<String>,
    model_dir: Option<String>,
}
#[derive(Serialize)]
struct ModelsRes {
    models: Vec<String>,
}
#[derive(Serialize)]
struct DirsRes {
    dirs: Vec<String>,
}
#[derive(Serialize)]
struct LsRes {
    dirs: Vec<String>,
    files: Vec<String>,
}
#[derive(Serialize)]
struct ReadFileRes {
    content: String,
}
#[derive(Deserialize)]
struct WriteFileReq {
    path: String,
    content: String,
}
#[derive(Deserialize)]
struct DirQuery {
    path: Option<String>,
}
async fn exec_tool(name: &str, args: &serde_json::Value, cwd: &PathBuf) -> (String, String) {
    let resolve = |p: &str| -> PathBuf {
        if PathBuf::from(p).is_absolute() {
            PathBuf::from(p)
        } else {
            cwd.join(p)
        }
    };
    match name {
        "read_file" => {
            let full = resolve(args["path"].as_str().unwrap_or(""));
            match tokio::fs::read_to_string(&full).await {
                Ok(c) => (c, "success".into()),
                Err(e) => (format!("Error: {}", e), "error".into()),
            }
        }
        "write_file" => {
            let full = resolve(args["path"].as_str().unwrap_or(""));
            let content = args["content"].as_str().unwrap_or("");
            if let Some(p) = full.parent() {
                let _ = tokio::fs::create_dir_all(p).await;
            }
            match tokio::fs::write(&full, content).await {
                Ok(_) => (
                    format!("Written {} bytes to {}", content.len(), full.display()),
                    "success".into(),
                ),
                Err(e) => (format!("Error: {}", e), "error".into()),
            }
        }
        "edit_file" => {
            let full = resolve(args["path"].as_str().unwrap_or(""));
            let old = args["old_string"].as_str().unwrap_or("");
            let new = args["new_string"].as_str().unwrap_or("");
            match tokio::fs::read_to_string(&full).await {
                Ok(c) if c.contains(old) => {
                    match tokio::fs::write(&full, c.replacen(old, new, 1)).await {
                        Ok(_) => (format!("Edited {}", full.display()), "success".into()),
                        Err(e) => (format!("Error: {}", e), "error".into()),
                    }
                }
                Ok(_) => ("String not found in file".into(), "error".into()),
                Err(e) => (format!("Error: {}", e), "error".into()),
            }
        }
        "run_command" => {
            let cmd = args["command"].as_str().unwrap_or("");
            let shell = if cfg!(windows) {
                ("cmd", vec!["/C", cmd])
            } else {
                ("sh", vec!["-c", cmd])
            };
            match tokio::process::Command::new(shell.0)
                .args(&shell.1)
                .current_dir(cwd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
            {
                Ok(o) => {
                    let out = format!(
                        "{}{}",
                        String::from_utf8_lossy(&o.stdout),
                        if o.stderr.is_empty() {
                            "".into()
                        } else {
                            format!("\nstderr: {}", String::from_utf8_lossy(&o.stderr))
                        }
                    );
                    let trimmed = if out.len() > 10000 {
                        format!("{}...(truncated)", &out[..10000])
                    } else {
                        out
                    };
                    (
                        trimmed,
                        if o.status.success() {
                            "success"
                        } else {
                            "error"
                        }
                        .into(),
                    )
                }
                Err(e) => (format!("Failed: {}", e), "error".into()),
            }
        }
        "list_directory" => {
            let full = resolve(args.get("path").and_then(|p| p.as_str()).unwrap_or("."));
            match tokio::fs::read_dir(&full).await {
                Ok(mut rd) => {
                    let mut items = Vec::new();
                    while let Ok(Some(e)) = rd.next_entry().await {
                        let n = e.file_name().to_string_lossy().to_string();
                        items.push(
                            if e.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                                format!("{}/", n)
                            } else {
                                n
                            },
                        );
                    }
                    items.sort();
                    (items.join("\n"), "success".into())
                }
                Err(e) => (format!("Error: {}", e), "error".into()),
            }
        }
        "search_files" => {
            let query = args["query"].as_str().unwrap_or("");
            let full = resolve(args.get("path").and_then(|p| p.as_str()).unwrap_or("."));
            let cmd = if cfg!(windows) {
                format!("findstr /s /n /i \"{}\" \"{}\\*\"", query, full.display())
            } else {
                format!("grep -rn -i \"{}\" \"{}\"", query, full.display())
            };
            let shell = if cfg!(windows) {
                ("cmd", vec!["/C".to_string(), cmd])
            } else {
                ("sh", vec!["-c".to_string(), cmd])
            };
            match tokio::process::Command::new(shell.0)
                .args(&shell.1)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
            {
                Ok(o) => {
                    let r = String::from_utf8_lossy(&o.stdout).to_string();
                    (
                        if r.is_empty() {
                            "No matches".into()
                        } else if r.len() > 10000 {
                            format!("{}...(truncated)", &r[..10000])
                        } else {
                            r
                        },
                        "success".into(),
                    )
                }
                Err(e) => (format!("Error: {}", e), "error".into()),
            }
        }
        "web_fetch" => exec_web_fetch(args).await,
        "web_search" => exec_web_search(args).await,
        "git_status" => git_run(cwd, &["status", "--short", "--branch"]).await,
        "git_diff" => match args.get("path").and_then(|p| p.as_str()).filter(|s| !s.is_empty()) {
            Some(p) => git_run(cwd, &["diff", "--", p]).await,
            None => git_run(cwd, &["diff"]).await,
        },
        "git_add" => {
            let path = args.get("path").and_then(|p| p.as_str()).filter(|s| !s.is_empty()).unwrap_or(".");
            git_run(cwd, &["add", "--", path]).await
        }
        "git_commit" => {
            let msg = args.get("message").and_then(|m| m.as_str()).unwrap_or("");
            if msg.trim().is_empty() {
                ("Error: git_commit requires a non-empty 'message'".into(), "error".into())
            } else {
                git_run(cwd, &["commit", "-m", msg]).await
            }
        }
        "git_log" => {
            let n = args.get("count").and_then(|c| c.as_u64()).unwrap_or(15).clamp(1, 100).to_string();
            git_run(cwd, &["log", "--oneline", "-n", n.as_str()]).await
        }
        "run_tests" => {
            let cmd = args.get("command").and_then(|c| c.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string())
                .or_else(|| detect_test_command(cwd).map(|s| s.to_string()));
            match cmd {
                None => ("No test command detected (looked for Cargo.toml, go.mod, pyproject.toml/pytest.ini/setup.py, package.json). Pass an explicit 'command'.".into(), "error".into()),
                Some(c) => {
                    let (out, status) = run_shell(cwd, &c).await;
                    (format!("$ {}\n{}", c, out), status)
                }
            }
        }
        "run_lint" => {
            let cmd = args.get("command").and_then(|c| c.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string())
                .or_else(|| detect_lint_command(cwd).map(|s| s.to_string()));
            match cmd {
                None => ("No linter detected (looked for Cargo.toml, go.mod, ruff.toml/pyproject.toml, eslint config, package.json). Pass an explicit 'command'.".into(), "error".into()),
                Some(c) => {
                    let (out, status) = run_shell(cwd, &c).await;
                    (format!("$ {}\n{}", c, out), status)
                }
            }
        }
        _ => (format!("Unknown tool: {}", name), "error".into()),
    }
}
async fn git_run(cwd: &PathBuf, gitargs: &[&str]) -> (String, String) {
    match tokio::process::Command::new("git")
        .args(gitargs)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
    {
        Ok(o) => {
            let mut out = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                if o.stderr.is_empty() { String::new() } else { format!("\n{}", String::from_utf8_lossy(&o.stderr)) }
            );
            out = out.trim().to_string();
            if out.len() > 10000 {
                out = format!("{}...(truncated)", &out[..10000]);
            } else if out.is_empty() {
                out = "(no output)".into();
            }
            (out, if o.status.success() { "success".into() } else { "error".into() })
        }
        Err(e) => (format!("git not available: {}", e), "error".into()),
    }
}
fn is_mutating_tool(name: &str) -> bool {
    matches!(name, "write_file" | "edit_file" | "run_command" | "git_add" | "git_commit" | "memory_write")
}
fn detect_test_command(cwd: &PathBuf) -> Option<&'static str> {
    if cwd.join("Cargo.toml").exists() {
        Some("cargo test")
    } else if cwd.join("go.mod").exists() {
        Some("go test ./...")
    } else if cwd.join("pyproject.toml").exists() || cwd.join("pytest.ini").exists() || cwd.join("setup.py").exists() || cwd.join("tox.ini").exists() {
        Some("python -m pytest -q")
    } else if cwd.join("package.json").exists() {
        Some("npm test")
    } else {
        None
    }
}
fn detect_lint_command(cwd: &PathBuf) -> Option<&'static str> {
    if cwd.join("Cargo.toml").exists() {
        Some("cargo clippy --quiet --all-targets")
    } else if cwd.join("go.mod").exists() {
        Some("go vet ./...")
    } else if cwd.join("ruff.toml").exists() || cwd.join("pyproject.toml").exists() {
        Some("ruff check .")
    } else if cwd.join(".eslintrc.json").exists() || cwd.join(".eslintrc.js").exists() || cwd.join(".eslintrc.cjs").exists() || cwd.join("eslint.config.js").exists() || cwd.join("eslint.config.mjs").exists() {
        Some("npx --no-install eslint .")
    } else if cwd.join("package.json").exists() {
        Some("npm run lint --if-present")
    } else {
        None
    }
}
async fn run_shell(cwd: &PathBuf, cmd: &str) -> (String, String) {
    let shell = if cfg!(windows) {
        ("cmd", vec!["/C".to_string(), cmd.to_string()])
    } else {
        ("sh", vec!["-c".to_string(), cmd.to_string()])
    };
    match tokio::process::Command::new(shell.0)
        .args(&shell.1)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
    {
        Ok(o) => {
            let mut out = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                if o.stderr.is_empty() { String::new() } else { format!("\n{}", String::from_utf8_lossy(&o.stderr)) }
            );
            if out.len() > 10000 {
                out = format!("{}...(truncated)", &out[..10000]);
            }
            (out, if o.status.success() { "success".into() } else { "error".into() })
        }
        Err(e) => (format!("Failed: {}", e), "error".into()),
    }
}
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    let mut in_script = false;
    let mut tag_buf = String::new();
    for ch in html.chars() {
        if ch == '<' { in_tag = true; tag_buf.clear(); continue; }
        if ch == '>' && in_tag {
            in_tag = false;
            let tag_lower = tag_buf.to_lowercase();
            if tag_lower.starts_with("script") || tag_lower.starts_with("style") || tag_lower.starts_with("nav") || tag_lower.starts_with("footer") { in_script = true; }
            if tag_lower.starts_with("/script") || tag_lower.starts_with("/style") || tag_lower.starts_with("/nav") || tag_lower.starts_with("/footer") { in_script = false; }
            continue;
        }
        if in_tag { tag_buf.push(ch); continue; }
        if !in_script { out.push(ch); }
    }
    let cleaned: String = out.split_whitespace().collect::<Vec<&str>>().join(" ");
    cleaned
}
fn is_private_url(url: &str) -> bool {
    if let Ok(parsed) = url::Url::parse(url) {
        let host = parsed.host_str().unwrap_or("");
        if host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "0.0.0.0" || host.is_empty() { return true; }
        if let Ok(addr) = host.parse::<std::net::IpAddr>() {
            return match addr {
                std::net::IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
                std::net::IpAddr::V6(v6) => v6.is_loopback(),
            };
        }
    }
    false
}
async fn exec_web_fetch(args: &serde_json::Value) -> (String, String) {
    let url = args["url"].as_str().unwrap_or("").trim();
    let max_chars = args.get("max_chars").and_then(|v| v.as_u64()).unwrap_or(6000) as usize;
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return ("Error: URL must start with http:// or https://".into(), "error".into());
    }
    if is_private_url(url) {
        return ("Error: Cannot fetch private/local URLs".into(), "error".into());
    }
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).user_agent("Amni-Code/2.0").build().unwrap_or_default();
    match client.get(url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return (format!("HTTP error: {}", resp.status()), "error".into());
            }
            match resp.text().await {
                Ok(body) => {
                    let text = strip_html(&body);
                    let truncated = if text.len() > max_chars { text[..max_chars].to_string() } else { text };
                    (truncated, "success".into())
                }
                Err(e) => (format!("Read error: {}", e), "error".into()),
            }
        }
        Err(e) => (format!("Fetch failed: {}", e), "error".into()),
    }
}
async fn exec_web_search(args: &serde_json::Value) -> (String, String) {
    let query = args["query"].as_str().unwrap_or("").trim();
    let max_results = args.get("max_results").and_then(|v| v.as_u64()).unwrap_or(5).min(10) as usize;
    if query.is_empty() {
        return ("Error: empty search query".into(), "error".into());
    }
    let encoded = urlencoding::encode(query);
    let url = format!("https://html.duckduckgo.com/html/?q={}", encoded);
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).user_agent("Amni-Code/2.0").build().unwrap_or_default();
    match client.get(&url).send().await {
        Ok(resp) => {
            match resp.text().await {
                Ok(html) => {
                    let mut results = Vec::new();
                    let re_link = regex::Regex::new(r#"class="result__a"[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#).unwrap();
                    let re_snip = regex::Regex::new(r#"class="result__snippet"[^>]*>(.*?)</(?:a|td|div)"#).unwrap();
                    let re_tags = regex::Regex::new(r"<[^>]+>").unwrap();
                    let re_uddg = regex::Regex::new(r"uddg=([^&]+)").unwrap();
                    let link_matches: Vec<_> = re_link.captures_iter(&html).collect();
                    for cap in link_matches.iter().take(max_results) {
                        let raw_href = &cap[1];
                        let title = re_tags.replace_all(&cap[2], "").trim().to_string();
                        let real_url = re_uddg.captures(raw_href).map(|u| urlencoding::decode(&u[1]).unwrap_or_default().to_string()).unwrap_or_else(|| raw_href.to_string());
                        let pos = cap.get(0).map(|m| m.end()).unwrap_or(0);
                        let snippet = if pos < html.len() {
                            let search_slice = &html[pos..(pos + 2000).min(html.len())];
                            re_snip.captures(search_slice).map(|s| re_tags.replace_all(&s[1], "").trim().to_string()).unwrap_or_default()
                        } else { String::new() };
                        results.push(serde_json::json!({"title": title, "url": real_url, "snippet": snippet}));
                    }
                    (serde_json::json!({"results": results, "count": results.len()}).to_string(), "success".into())
                }
                Err(e) => (format!("Read error: {}", e), "error".into()),
            }
        }
        Err(e) => (format!("Search failed: {}", e), "error".into()),
    }
}
fn load_memory_store(cwd: &Path) -> HashMap<String, Vec<String>> {
    let mem_path = cwd.join(".amni-memory.json");
    if mem_path.exists() {
        std::fs::read_to_string(&mem_path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    } else {
        let home_path = dirs::home_dir().map(|h| h.join(".amni").join("memory.json")).unwrap_or_default();
        if home_path.exists() {
            std::fs::read_to_string(&home_path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }
}
fn save_memory_store(cwd: &Path, store: &HashMap<String, Vec<String>>) {
    let mem_path = cwd.join(".amni-memory.json");
    if let Ok(json) = serde_json::to_string_pretty(store) {
        let _ = std::fs::write(&mem_path, json);
    }
}
fn load_custom_instructions(cwd: &Path) -> String {
    let candidates = [".amni-instructions.md", ".amni-instructions.txt", ".github/copilot-instructions.md", "AGENTS.md"];
    let mut instructions = Vec::new();
    for name in &candidates {
        let path = cwd.join(name);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    instructions.push(format!("--- {} ---\n{}", name, trimmed));
                }
            }
        }
    }
    if instructions.is_empty() { String::new() } else { format!("PROJECT INSTRUCTIONS:\n{}", instructions.join("\n\n")) }
}
struct ToolCall {
    id: String,
    name: String,
    args: serde_json::Value,
}
async fn llm_request(
    config: &Config,
    messages: &[serde_json::Value],
    use_tools: bool,
) -> Result<(serde_json::Value, Vec<ToolCall>), (String, bool)> {
    let (url, key_header) = match config.provider.as_str() {
        "ollama" => (format!("{}/v1/chat/completions", config.base_url), None),
        "local" => (format!("{}/v1/chat/completions", config.base_url), None),
        "amni" => (format!("{}/v1/chat/completions", amni_base_url(&config.base_url)), None),
        "openai" => (
            format!("{}/v1/chat/completions", config.base_url.trim_end_matches('/')),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        "anthropic" => (
            "https://api.anthropic.com/v1/messages".into(),
            Some(("x-api-key", config.api_key.clone())),
        ),
        "xai" => (
            "https://api.x.ai/v1/chat/completions".into(),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        "google" => ("dummy".into(), None),
        "deepseek" => (
            "https://api.deepseek.com/chat/completions".into(),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        "groq" => (
            "https://api.groq.com/openai/v1/chat/completions".into(),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        "mistral" => (
            "https://api.mistral.ai/v1/chat/completions".into(),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        "cohere" => (
            "https://api.cohere.com/v2/chat".into(),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        "together" => (
            "https://api.together.xyz/v1/chat/completions".into(),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        "fireworks" => (
            "https://api.fireworks.ai/inference/v1/chat/completions".into(),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        "perplexity" => (
            "https://api.perplexity.ai/chat/completions".into(),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        "openrouter" => (
            "https://openrouter.ai/api/v1/chat/completions".into(),
            Some(("Authorization", format!("Bearer {}", config.api_key))),
        ),
        other => return Err((format!("Unknown provider: {}", other), false)),
    };
    if config.provider == "google" {
        let prompt = messages.iter().filter_map(|m| {
            let role = m["role"].as_str().unwrap_or("");
            let content = m["content"].as_str().unwrap_or("");
            Some(format!("{}: {}", role, content))
        }).collect::<Vec<_>>().join("\n");
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", config.model, config.api_key);
        let body = serde_json::json!({"contents": [{"parts": [{"text": prompt}]}]});
        let client = reqwest::Client::new();
        let resp = client.post(&url).header("Content-Type", "application/json").json(&body).send().await.map_err(|e| (format!("Request failed: {}", e), false))?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| (format!("Read failed: {}", e), false))?;
        if !status.is_success() {
            return Err((format!("API error {}: {}", status, &text[..text.len().min(500)]), false));
        }
        let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| (format!("Parse error: {}", e), false))?;
        let content = json["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("").to_string();
        let raw_msg = serde_json::json!({"role": "assistant", "content": content});
        return Ok((raw_msg, Vec::new()));
    }
    let max_tokens = std::env::var("AMNI_CODE_MAX_TOKENS").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(4096);
    let mut body =
        serde_json::json!({"model": config.model, "messages": messages, "max_tokens": max_tokens});
    if use_tools {
        let tools: serde_json::Value = serde_json::from_str(TOOLS_JSON).unwrap();
        body["tools"] = tools;
        body["tool_choice"] = serde_json::json!("auto");
    }
    let client = reqwest::Client::new();
    let mut req = client.post(&url).header("Content-Type", "application/json");
    if let Some((k, v)) = &key_header {
        req = req.header(k.to_owned(), v.to_owned());
    }
    if config.provider == "anthropic" {
        req = req.header("anthropic-version", "2023-06-01");
    }
    let resp = req
        .json(&body)
        .send()
        .await
        .map_err(|e| (format!("Request failed: {}", e), false))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| (format!("Read failed: {}", e), false))?;
    if !status.is_success() {
        let is_tool_err = text.contains("does not support tools")
            || text.contains("tool_use")
            || text.contains("tools is not supported");
        return Err((
            format!("API error {}: {}", status, &text[..text.len().min(500)]),
            is_tool_err,
        ));
    }
    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| (format!("Parse error: {}", e), false))?;
    let raw_msg = json["choices"][0]["message"].clone();
    let mut tool_calls = Vec::new();
    if use_tools {
        if let Some(calls) = raw_msg.get("tool_calls").and_then(|t| t.as_array()) {
            for call in calls {
                let id = call["id"].as_str().unwrap_or("").to_string();
                let name = call["function"]["name"].as_str().unwrap_or("").to_string();
                let args: serde_json::Value =
                    serde_json::from_str(call["function"]["arguments"].as_str().unwrap_or("{}"))
                        .unwrap_or_default();
                tool_calls.push(ToolCall { id, name, args });
            }
        }
    }
    Ok((raw_msg, tool_calls))
}
async fn llm_call(
    config: &Config,
    messages: &[serde_json::Value],
) -> Result<(serde_json::Value, Vec<ToolCall>), String> {
    match llm_request(config, messages, true).await {
        Ok(r) => Ok(r),
        Err((_msg, true)) => llm_request(config, messages, false)
            .await
            .map_err(|(e, _)| e),
        Err((msg, false)) => Err(msg),
    }
}
fn compact_context(messages: &mut Vec<serde_json::Value>) {
    let total_chars: usize = messages
        .iter()
        .filter_map(|m| m["content"].as_str())
        .map(|s| s.len())
        .sum();
    if total_chars < 80000 || messages.len() < 20 {
        return;
    }
    let keep_recent = 10;
    if messages.len() <= keep_recent + 1 {
        return;
    }
    let system_msg = messages[0].clone();
    let recent: Vec<_> = messages[messages.len() - keep_recent..].to_vec();
    let dropped = messages.len() - 1 - keep_recent;
    messages.clear();
    messages.push(system_msg);
    messages.push(serde_json::json!({
        "role": "system",
        "content": format!("[Context auto-compacted: {} earlier messages removed to maintain performance. Continue from the recent context below.]", dropped)
    }));
    messages.extend(recent);
}
#[derive(Debug, PartialEq)]
enum SlashCmd {
    Tool(&'static str, serde_json::Value),
    SetMode(&'static str),
    Prompt(String),
    Help,
}
fn parse_slash(input: &str) -> Option<SlashCmd> {
    let rest = input.trim().strip_prefix('/')?;
    let (cmd, arg) = match rest.split_once(char::is_whitespace) {
        Some((c, a)) => (c, a.trim()),
        None => (rest, ""),
    };
    match cmd {
        "test" => Some(SlashCmd::Tool("run_tests", if arg.is_empty() { serde_json::json!({}) } else { serde_json::json!({ "command": arg }) })),
        "status" => Some(SlashCmd::Tool("git_status", serde_json::json!({}))),
        "diff" => Some(SlashCmd::Tool("git_diff", if arg.is_empty() { serde_json::json!({}) } else { serde_json::json!({ "path": arg }) })),
        "log" => Some(SlashCmd::Tool("git_log", serde_json::json!({}))),
        "commit" => Some(SlashCmd::Prompt(format!(
            "Review changes with git_status and git_diff, then git_add and git_commit with a clear, concise message{}.",
            if arg.is_empty() { String::new() } else { format!(" (the user suggests: \"{}\")", arg) }
        ))),
        "review" => Some(SlashCmd::Prompt(format!(
            "Review {} for bugs, edge cases, security, and clarity. Inspect with read_file/git_diff, then list concrete findings as 'file:line: issue -> fix'; if clean, say so.",
            if arg.is_empty() { "the recent changes (git_diff)".to_string() } else { format!("`{}`", arg) }
        ))),
        "plan" => Some(SlashCmd::SetMode("plan")),
        "edit" => Some(SlashCmd::SetMode("edit")),
        "auto" | "autonomous" => Some(SlashCmd::SetMode("autonomous")),
        "help" | "?" => Some(SlashCmd::Help),
        _ => None,
    }
}
const SLASH_HELP: &str = "**Slash commands:**\n- `/test [cmd]` - run the test suite (auto-detected)\n- `/status` - git status\n- `/diff [path]` - git diff\n- `/log` - recent commits\n- `/commit [msg]` - review + stage + commit\n- `/review [path]` - code review of changes or a path\n- `/plan` `/edit` `/auto` - switch agent mode\n- `/help` - this list";
enum SlashOutcome { NotSlash, Handled, Rewrite(String) }
async fn handle_slash(app: &App, sid: &str, user_msg: &str, wd: &Option<String>, tx: &tokio::sync::mpsc::Sender<SseEvent>) -> SlashOutcome {
    let scmd = match parse_slash(user_msg) { Some(c) => c, None => return SlashOutcome::NotSlash };
    let inline = match scmd {
        SlashCmd::Prompt(p) => return SlashOutcome::Rewrite(p),
        other => other,
    };
    let _ = tx.send(SseEvent::default().event("session").data(serde_json::json!({ "session_id": sid }).to_string())).await;
    match inline {
        SlashCmd::Help => {
            let _ = tx.send(SseEvent::default().event("message").data(serde_json::json!({ "message": SLASH_HELP }).to_string())).await;
        }
        SlashCmd::SetMode(m) => {
            { let mut c = app.config.lock().await; c.mode = m.to_string(); persist_config(&c.clone()); }
            let _ = tx.send(SseEvent::default().event("message").data(serde_json::json!({ "message": format!("Switched to **{}** mode.", m) }).to_string())).await;
        }
        SlashCmd::Tool(name, args) => {
            let cwd_now = session_cwd(app, sid, wd).await;
            let _ = tx.send(SseEvent::default().event("tool_start").data(serde_json::json!({ "tool": name, "input": &args }).to_string())).await;
            let (output, status) = exec_tool(name, &args, &cwd_now).await;
            let _ = tx.send(SseEvent::default().event("tool_result").data(serde_json::json!({ "tool": name, "input": &args, "output": &output, "status": &status }).to_string())).await;
            let _ = tx.send(SseEvent::default().event("message").data(serde_json::json!({ "message": format!("`/{}` result:\n```\n{}\n```", name, output) }).to_string())).await;
        }
        SlashCmd::Prompt(_) => {}
    }
    let _ = tx.send(SseEvent::default().event("done").data(serde_json::json!({ "session_id": sid }).to_string())).await;
    SlashOutcome::Handled
}
fn extract_mentions(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'@' && (i == 0 || b[i - 1].is_ascii_whitespace()) {
            let start = i + 1;
            let mut j = start;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || matches!(b[j], b'/' | b'.' | b'_' | b'-')) {
                j += 1;
            }
            if j > start {
                let p = text[start..j].trim_end_matches(|c: char| c == '.' || c == ',');
                if !p.is_empty() && (p.contains('/') || p.contains('.')) {
                    out.push(p.to_string());
                }
            }
            i = j.max(start);
        } else {
            i += 1;
        }
    }
    out
}
async fn session_cwd(app: &App, sid: &str, wd: &Option<String>) -> PathBuf {
    let cwd = app.cwd.lock().await.clone();
    match app.sessions.lock().await.get(sid) {
        Some(s) if !s.working_dir.is_empty() => PathBuf::from(&s.working_dir),
        _ => wd.as_ref().map_or(cwd, |w| PathBuf::from(w)),
    }
}
async fn resolve_mentions(user_msg: &str, cwd: &PathBuf) -> Option<String> {
    let mentions = extract_mentions(user_msg);
    if mentions.is_empty() {
        return None;
    }
    let mut ctx = String::new();
    let mut added = 0;
    for m in mentions.iter().take(8) {
        let full = if PathBuf::from(m).is_absolute() { PathBuf::from(m) } else { cwd.join(m) };
        if let Ok(content) = tokio::fs::read_to_string(&full).await {
            let snippet: String = content.chars().take(8000).collect();
            ctx.push_str(&format!("Contents of {}:\n```\n{}\n```\n\n", m, snippet));
            added += 1;
        }
    }
    if added == 0 { None } else { Some(format!("{}{}", ctx, user_msg)) }
}
async fn agent_loop_stream(app:App,sid:String,mut user_msg:String,wd:Option<String>,tx:tokio::sync::mpsc::Sender<SseEvent>){match handle_slash(&app,&sid,&user_msg,&wd,&tx).await{SlashOutcome::Handled=>return,SlashOutcome::Rewrite(p)=>user_msg=p,SlashOutcome::NotSlash=>{}}{let __mcwd=session_cwd(&app,&sid,&wd).await;if let Some(__ex)=resolve_mentions(&user_msg,&__mcwd).await{user_msg=__ex;}}let config=app.config.lock().await.clone();ensure_model_loaded(&config).await;let cwd=app.cwd.lock().await.clone();let cwd_path=if let Some(s)=app.sessions.lock().await.get(&sid){if!s.working_dir.is_empty(){PathBuf::from(&s.working_dir)}else{cwd.clone()}}else{wd.as_ref().map_or(cwd.clone(),|w|PathBuf::from(w))};{let mut sessions=app.sessions.lock().await;let session=sessions.entry(sid.clone()).or_default();if session.messages.is_empty(){let custom=load_custom_instructions(&cwd_path);let sys=SYSTEM_PROMPT.replace("{CWD}",&cwd_path.display().to_string()).replace("{CUSTOM_INSTRUCTIONS}",&custom);session.messages.push(serde_json::json!({"role":"system","content":sys}));if let Some(w)=wd{session.working_dir=w.clone();}}session.messages.push(serde_json::json!({"role":"user","content":&user_msg}));compact_context(&mut session.messages);}let interrupt_flag={let mut interrupts=app.interrupts.lock().await;let flag=interrupts.entry(sid.clone()).or_insert_with(||Arc::new(AtomicBool::new(false))).clone();flag.store(false,std::sync::atomic::Ordering::Relaxed);flag};
let _=tx.send(SseEvent::default().event("session").data(serde_json::json!({"session_id":&sid}).to_string())).await;
let max_iters=std::env::var("AMNI_CODE_MAX_ITERS").ok().and_then(|v|v.parse::<u32>().ok()).unwrap_or(100);let mut it:u32=0;while it<max_iters{if interrupt_flag.load(std::sync::atomic::Ordering::Relaxed){let _=tx.send(SseEvent::default().event("interrupted").data(serde_json::json!({"session_id":&sid}).to_string())).await;return;}it+=1;let messages=app.sessions.lock().await.entry(sid.clone()).or_default().messages.clone();match llm_call(&config,&messages).await{Ok((raw_msg,tool_calls))=>{if let Some(widgets)=raw_msg.get("amni_widgets").and_then(|w|w.as_array()){for w in widgets{let _=tx.send(SseEvent::default().event("widget").data(w.to_string())).await;}}if tool_calls.is_empty(){let content=raw_msg["content"].as_str().unwrap_or("").to_string();app.sessions.lock().await.entry(sid.clone()).or_default().messages.push(raw_msg);let _=tx.send(SseEvent::default().event("message").data(serde_json::json!({"message":&content}).to_string())).await;let _=tx.send(SseEvent::default().event("done").data(serde_json::json!({"session_id":&sid}).to_string())).await;return;}
                app.sessions
                    .lock()
                    .await
                    .entry(sid.clone())
                    .or_default()
                    .messages
                    .push(raw_msg);
                for tc in &tool_calls {
                    let _ = tx
                        .send(
                            SseEvent::default().event("tool_start").data(
                                serde_json::json!({"tool": &tc.name, "input": &tc.args})
                                    .to_string(),
                            ),
                        )
                        .await;
                    // Backup before file operations
                    if tc.name == "write_file" || tc.name == "edit_file" {
                        if let Some(path) = tc.args["path"].as_str() {
                            let full_path = if PathBuf::from(path).is_absolute() {
                                PathBuf::from(path)
                            } else {
                                cwd_path.join(path)
                            };
                            let original = tokio::fs::read_to_string(&full_path).await.ok();
                            let mut backups = app.backups.lock().await;
                            let session_backups = backups.entry(sid.clone()).or_default();
                            let path_str = full_path.display().to_string();
                            if !session_backups.iter().any(|(p, _)| p == &path_str) {
                                session_backups.push((path_str, original));
                            }
                        }
                    }
                    let amni_mode = { app.config.lock().await.mode.clone() };
                    let (output, status) = if amni_mode == "plan" && is_mutating_tool(&tc.name) {
                        let argp: String = serde_json::to_string(&tc.args).unwrap_or_default().chars().take(220).collect();
                        (format!("[PLAN MODE] Proposed (not executed): {}({}). You are in plan mode (read-only) - keep exploring with read-only tools, present the full plan, then tell the user to switch to edit or autonomous mode to apply it.", tc.name, argp), "planned".into())
                    } else if tc.name == "memory_read" {
                        let key = tc.args["key"].as_str().unwrap_or("").to_string();
                        let mem = app.memory.lock().await;
                        let entries = mem.get(&key).cloned().unwrap_or_default();
                        if entries.is_empty() { (format!("No memory found for key '{}'", key), "success".into()) }
                        else { (entries.join("\n---\n"), "success".into()) }
                    } else if tc.name == "memory_write" {
                        let key = tc.args["key"].as_str().unwrap_or("").to_string();
                        let content = tc.args["content"].as_str().unwrap_or("").to_string();
                        let mut mem = app.memory.lock().await;
                        mem.entry(key.clone()).or_default().push(content.clone());
                        save_memory_store(&cwd_path, &mem);
                        (format!("Stored to memory key '{}': {} chars", key, content.len()), "success".into())
                    } else {
                        exec_tool(&tc.name, &tc.args, &cwd_path).await
                    };
                    let _ = tx
                        .send(
                            SseEvent::default().event("tool_result").data(
                                serde_json::json!({
                                    "tool": &tc.name,
                                    "input": &tc.args,
                                    "output": &output,
                                    "status": &status
                                })
                                .to_string(),
                            ),
                        )
                        .await;
                    app.sessions
                        .lock()
                        .await
                        .entry(sid.clone())
                        .or_default()
                        .messages
                        .push(serde_json::json!({"role": "tool", "tool_call_id": &tc.id, "content": &output}));
                }
            }
            Err(e) => {
                let err_msg = format!("Error: {}", e);
                app.sessions
                    .lock()
                    .await
                    .entry(sid.clone())
                    .or_default()
                    .messages
                    .push(serde_json::json!({"role": "assistant", "content": &err_msg}));
                let _ = tx
                    .send(
                        SseEvent::default()
                            .event("error")
                            .data(serde_json::json!({"error": &err_msg}).to_string()),
                    )
                    .await;
                let _ = tx
                    .send(
                        SseEvent::default()
                            .event("done")
                            .data(serde_json::json!({"session_id": &sid}).to_string()),
                    )
                    .await;
                return;
            }
        }
    }
}
async fn handle_chat(
    State(app): State<App>,
    Json(req): Json<ChatReq>,
) -> Sse<impl futures::Stream<Item = Result<SseEvent, Infallible>>> {
let sid=req.session_id.unwrap_or_else(||uuid::Uuid::new_v4().to_string());let(tx,rx)=tokio::sync::mpsc::channel::<SseEvent>(32);tokio::spawn(async move{agent_loop_stream(app,sid,req.message,req.working_dir,tx).await;});
    Sse::new(ReceiverStream::new(rx).map(|e| Ok::<_, Infallible>(e)))
        .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15)))
}
async fn handle_config_get(State(app): State<App>) -> Json<Config> {
    let mut cfg = app.config.lock().await;
    let base_before = cfg.base_url.clone();
    normalize_config(&mut cfg);
    if cfg.base_url != base_before {
        persist_config(&cfg);
    }
    Json(cfg.clone())
}
async fn handle_config_set(State(app): State<App>, Json(req): Json<ConfigReq>) -> Json<Config> {
    let mut cfg = app.config.lock().await;
    if let Some(v) = req.provider {
        cfg.provider = v;
    }
    if let Some(v) = req.model {
        cfg.model = v;
    }
    if let Some(v) = req.api_key {
        cfg.api_key = v;
    }
    if let Some(v) = req.base_url {
        cfg.base_url = v;
    }
    if let Some(v) = req.auto_approve {
        cfg.auto_approve = v;
    }
    if let Some(v) = req.mode {
        cfg.mode = v;
    }
    if let Some(v) = req.working_dir {
        cfg.working_dir = v.clone();
        *app.cwd.lock().await = PathBuf::from(v);
    }
    if let Some(v) = req.model_dir {
        cfg.model_dir = v;
    }
    normalize_config(&mut cfg);
    persist_config(&cfg);
    Json(cfg.clone())
}
#[derive(Deserialize)]
struct UndoReq {
    session_id: String,
}
async fn handle_undo(State(app): State<App>, Json(req): Json<UndoReq>) -> Json<serde_json::Value> {
    let mut backups = app.backups.lock().await;
    if let Some(changes) = backups.remove(&req.session_id) {
        let mut reverted = Vec::new();
        for (path, original) in changes.into_iter().rev() {
            match original {
                Some(content) => {
                    if tokio::fs::write(&path, &content).await.is_ok() {
                        reverted.push(path);
                    }
                }
                None => {
                    let _ = tokio::fs::remove_file(&path).await;
                    reverted.push(path);
                }
            }
        }
        Json(serde_json::json!({"status": "ok", "reverted": reverted}))
    } else {
        Json(serde_json::json!({"status": "no_changes", "reverted": []}))
    }
}
async fn handle_accept(State(app): State<App>, Json(req): Json<UndoReq>) -> Json<serde_json::Value> {
    app.backups.lock().await.remove(&req.session_id);
    Json(serde_json::json!({"status": "ok"}))
}
#[derive(Deserialize)]
struct SteerReq {
    session_id: String,
    message: String,
}
async fn handle_steer(State(app): State<App>, Json(req): Json<SteerReq>) -> Json<serde_json::Value> {
    let mut sessions = app.sessions.lock().await;
    if let Some(session) = sessions.get_mut(&req.session_id) {
        session.messages.push(serde_json::json!({"role": "user", "content": format!("[STEER: {}]", req.message)}));
        compact_context(&mut session.messages);
        Json(serde_json::json!({"status": "steered"}))
    } else {
        Json(serde_json::json!({"status": "no_session"}))
    }
}
#[derive(Deserialize)]
struct InterruptReq {
    session_id: String,
}
async fn handle_interrupt(State(app): State<App>, Json(req): Json<InterruptReq>) -> Json<serde_json::Value> {
    let interrupts = app.interrupts.lock().await;
    if let Some(flag) = interrupts.get(&req.session_id) {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
        Json(serde_json::json!({"status": "interrupted"}))
    } else {
        Json(serde_json::json!({"status": "no_session"}))
    }
}
fn find_amni_ai_dir(config: &Config) -> Option<PathBuf> {
    if let Ok(d) = std::env::var("AMNI_AI_DIR") {
        let p = PathBuf::from(d);
        if p.join("amni").join("web").join("server.py").exists() { return Some(p); }
    }
    let mut search_parents: Vec<PathBuf> = Vec::new();
    let working = PathBuf::from(&config.working_dir);
    if let Some(p) = working.parent() { search_parents.push(p.to_path_buf()); }
    if let Some(gp) = working.parent().and_then(|p| p.parent()) { search_parents.push(gp.to_path_buf()); }
    if let Ok(cur) = std::env::current_dir() {
        if let Some(p) = cur.parent() { search_parents.push(p.to_path_buf()); }
        search_parents.push(cur);
    }
    // All case variants for the directory name
    let names = ["Amni-Ai", "Amni-AI", "amni-ai", "amni_ai", "AmniAI"];
    for parent in &search_parents {
        for name in &names {
            let c = parent.join(name);
            if c.join("amni").join("web").join("server.py").exists() { return Some(c); }
        }
    }
    None
}
async fn stop_managed_server(app:&App)->bool{
    let mut guard=app.amni_proc.lock().await;
    let child=guard.take();
    drop(guard);
    if let Some(mut child)=child{
        let _=child.start_kill();
        let _=child.wait().await;
        true
    }else{false}
}
async fn probe_server_health(client:&reqwest::Client,base:&str)->(bool,Option<String>){
    match client.get(format!("{}/health",base)).send().await{
        Ok(resp)=>{if !resp.status().is_success(){return(false,None);}match resp.json::<serde_json::Value>().await{Ok(json)=>(true,json["active_model"].as_str().map(|s|s.to_string())),Err(_)=>(true,None)}},
        Err(_)=>(false,None),
    }
}
async fn handle_server_status(State(app): State<App>) -> Json<serde_json::Value> {
    let cfg = app.config.lock().await.clone();
    let local_providers = ["amni", "local", "ollama"];
    if !local_providers.contains(&cfg.provider.as_str()) {
        return Json(serde_json::json!({"running": true, "applicable": false}));
    }
    let base = if cfg.provider == "amni" { amni_base_url(&cfg.base_url) } else { cfg.base_url.trim_end_matches('/').to_string() };
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(2)).build().unwrap_or_default();
    let (reachable,active_model)=if cfg.provider=="amni"{probe_server_health(&client,&base).await}else{(client.get(format!("{}/health", base)).send().await.map(|r| r.status().is_success()).unwrap_or(false),None)};
    let desired_model=if cfg.provider=="amni"{if cfg.model.trim().is_empty(){"adam:granite-gf17".to_string()}else{cfg.model.clone()}}else{cfg.model.clone()};
    let matches_model=true;
    let running = reachable && matches_model;
    let proc_running = app.amni_proc.lock().await.is_some();
    Json(serde_json::json!({"running": running, "reachable": reachable, "matching": matches_model, "active_model": active_model, "desired_model": desired_model, "starting": proc_running && !running, "applicable": true, "provider": cfg.provider}))
}
async fn handle_server_start(State(app): State<App>) -> Json<serde_json::Value> {
    let cfg = app.config.lock().await.clone();
    if cfg.provider != "amni" && cfg.provider != "local" {
        return Json(serde_json::json!({"status": "not_applicable"}));
    }
    let base = if cfg.provider == "amni" { amni_base_url(&cfg.base_url) } else { cfg.base_url.trim_end_matches('/').to_string() };
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(2)).build().unwrap_or_default();
    let tx = app.server_log_tx.clone();
    let desired_model=if cfg.provider=="amni"{if cfg.model.trim().is_empty(){"amni-a1".to_string()}else{cfg.model.clone()}}else{cfg.model.clone()};
    if cfg.provider=="amni"{
        let (reachable,active_model)=probe_server_health(&client,&base).await;
        if reachable && active_model.as_deref()==Some(desired_model.as_str()) {
            return Json(serde_json::json!({"status": "already_running", "active_model": active_model, "desired_model": desired_model}));
        }
        if reachable {
            if app.amni_proc.lock().await.is_some() {
                let _ = tx.send(format!("Restarting Amni-AI for selected model: {}", desired_model));
                let _ = stop_managed_server(&app).await;
            } else {
                return Json(serde_json::json!({"status": "error", "message": format!("Amni-AI is already running with model '{}' but '{}' is selected. Stop the external Amni-AI process or let Amni-Code launch it.", active_model.unwrap_or_else(|| "unknown".to_string()), desired_model)}));
            }
        }
    } else if client.get(format!("{}/health", base)).send().await.map(|r| r.status().is_success()).unwrap_or(false) {
        return Json(serde_json::json!({"status": "already_running"}));
    }
    // Resolve command before acquiring the proc lock (avoid holding mutex across awaits)
    if app.amni_proc.lock().await.is_some() {
        return Json(serde_json::json!({"status": "already_starting"}));
    }
    enum LaunchTarget { LlamaServer(PathBuf, u16), AmniAI(PathBuf) }
    let target: LaunchTarget = if cfg.provider == "local" {
        let model_dir = if !cfg.model_dir.is_empty() {
            Some(PathBuf::from(&cfg.model_dir))
        } else {
            auto_detect_model_dir(&cfg.working_dir).await
        };
        let port = cfg.base_url.split(':').last().and_then(|p| p.parse::<u16>().ok()).unwrap_or(8787);
        let gguf = if let Some(ref d) = model_dir {
            find_gguf_path(d, &cfg.model).await
        } else { None };
        if let Some(gpath) = gguf {
            LaunchTarget::LlamaServer(gpath, port)
        } else {
            match find_amni_ai_dir(&cfg) {
                Some(d) => LaunchTarget::AmniAI(d),
                None => return Json(serde_json::json!({"status": "error", "message": "No GGUF model found for llama-server and Amni-AI directory not found. Configure a model directory or set AMNI_AI_DIR."})),
            }
        }
    } else {
        match find_amni_ai_dir(&cfg) {
            Some(d) => LaunchTarget::AmniAI(d),
            None => return Json(serde_json::json!({"status": "error", "message": "Could not find Amni-AI directory. Set AMNI_AI_DIR env var or place Amni-Ai alongside Amni-Code."})),
        }
    };
    let target_dir_str = match &target {
        LaunchTarget::LlamaServer(p, _) => p.display().to_string(),
        LaunchTarget::AmniAI(p) => p.display().to_string(),
    };
    // Now acquire the lock to actually store the process
    let mut proc_guard = app.amni_proc.lock().await;
    if proc_guard.is_some() {
        return Json(serde_json::json!({"status": "already_starting"}));
    }
    let mut cmd = match target {
        LaunchTarget::LlamaServer(gpath, port) => {
            let _ = tx.send(format!("Starting llama-server: {} --port {}", gpath.display(), port));
            let mut c = tokio::process::Command::new("llama-server");
            c.args(["--model", &gpath.to_string_lossy(), "--port", &port.to_string(), "--ctx-size", "8192"])
             .stdout(Stdio::piped()).stderr(Stdio::piped());
            c
        }
        LaunchTarget::AmniAI(d) => {
            let _ = tx.send(format!("Starting Amni-AI from: {}", d.display()));
            let mut c = tokio::process::Command::new("python");
            c.args(["-m", "amni.web.server", "--model", desired_model.as_str()]).current_dir(&d).stdout(Stdio::piped()).stderr(Stdio::piped());
            c
        }
    };
    match cmd.spawn() {
        Ok(mut child) => {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();
            *proc_guard = Some(child);
            drop(proc_guard);
            if let Some(out) = stdout {
                let tx2 = tx.clone();
                tokio::spawn(async move {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let mut lines = BufReader::new(out).lines();
                    while let Ok(Some(line)) = lines.next_line().await { let _ = tx2.send(line); }
                });
            }
            if let Some(err) = stderr {
                let tx3 = tx.clone();
                tokio::spawn(async move {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let mut lines = BufReader::new(err).lines();
                    while let Ok(Some(line)) = lines.next_line().await { let _ = tx3.send(format!("[err] {}", line)); }
                });
            }
            let tx4 = tx.clone();
            tokio::spawn(async move {
                let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(2)).build().unwrap_or_default();
                for i in 0..90u32 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    if client.get(format!("{}/health", base)).send().await.map(|r| r.status().is_success()).unwrap_or(false) {
                        let _ = tx4.send("__SERVER_READY__".to_string());
                        return;
                    }
                    if i % 5 == 4 { let _ = tx4.send(format!("Waiting for server... ({}s)", (i+1)*2)); }
                }
                let _ = tx4.send("__SERVER_TIMEOUT__".to_string());
            });
            Json(serde_json::json!({"status": "starting", "dir": target_dir_str}))
        }
        Err(e) => Json(serde_json::json!({"status": "error", "message": e.to_string()}))
    }
}
async fn handle_server_log(State(app): State<App>) -> Sse<impl futures::Stream<Item = Result<SseEvent, Infallible>> + Send + 'static> {
    let mut bcast_rx = app.server_log_tx.subscribe();
    let (tx, rx) = tokio::sync::mpsc::channel::<SseEvent>(128);
    tokio::spawn(async move {
        loop {
            match bcast_rx.recv().await {
                Ok(line) if line == "__SERVER_READY__" => {
                    let _ = tx.send(SseEvent::default().event("ready").data("Server ready")).await;
                    break;
                }
                Ok(line) if line == "__SERVER_TIMEOUT__" => {
                    let _ = tx.send(SseEvent::default().event("timeout").data("Server failed to start after 3 minutes")).await;
                    break;
                }
                Ok(line) => { let _ = tx.send(SseEvent::default().data(line)).await; }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });
    Sse::new(ReceiverStream::new(rx).map(|e| Ok::<_, Infallible>(e))).keep_alive(KeepAlive::default())
}
async fn handle_models(State(app): State<App>) -> Json<ModelsRes> {
    let cfg = app.config.lock().await.clone();
    let base = cfg.base_url.trim_end_matches('/').to_string();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();
    let mut models = Vec::new();
    tracing::info!(
        "Model discovery: provider={}, base_url={}",
        cfg.provider,
        base
    );
    if cfg.provider == "ollama" {
        let ollama_installed = tokio::process::Command::new("ollama")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ollama_installed {
            let mut resp = client.get(format!("{}/api/tags", base)).send().await;
            if resp.is_err() {
                let _ = tokio::process::Command::new("ollama").arg("serve").spawn();
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                resp = client.get(format!("{}/api/tags", base)).send().await;
            }
            if let Ok(r) = resp {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    if let Some(arr) = json["models"].as_array() {
                        for m in arr {
                            if let Some(n) = m["name"].as_str() {
                                models.push(n.to_string());
                            }
                        }
                    }
                }
            }
        }
        if models.is_empty() {
            let scan_dir = if !cfg.model_dir.is_empty() {
                Some(PathBuf::from(&cfg.model_dir))
            } else {
                auto_detect_model_dir(&cfg.working_dir).await
            };
            if let Some(d) = scan_dir {
                tracing::info!("Ollama file scan: {:?}", d);
                models = collect_models(&d).await;
            }
        }
    } else if cfg.provider == "local" {
        tracing::info!("Local: trying {}/v1/models", base);
        match client.get(format!("{}/v1/models", base)).send().await {
            Ok(resp) => {
                let status = resp.status();
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        tracing::info!("Local /v1/models status={} body={}", status, json);
                        if let Some(arr) = json["data"].as_array() {
                            for m in arr {
                                if let Some(n) = m["id"].as_str() {
                                    models.push(n.to_string());
                                }
                            }
                        }
                    }
                    Err(e) => tracing::warn!("Local /v1/models parse error: {}", e),
                }
            }
            Err(e) => tracing::warn!("Local /v1/models fetch error: {}", e),
        }
        if models.is_empty() {
            tracing::info!("Local: trying {}/api/tags", base);
            match client.get(format!("{}/api/tags", base)).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            tracing::info!("Local /api/tags status={} body={}", status, json);
                            if let Some(arr) = json["models"].as_array() {
                                for m in arr {
                                    if let Some(n) = m["name"].as_str() {
                                        models.push(n.to_string());
                                    }
                                }
                            }
                        }
                        Err(e) => tracing::warn!("Local /api/tags parse error: {}", e),
                    }
                }
                Err(e) => tracing::warn!("Local /api/tags fetch error: {}", e),
            }
        }
        let scan_dir = if !cfg.model_dir.is_empty() {
            Some(PathBuf::from(&cfg.model_dir))
        } else {
            auto_detect_model_dir(&cfg.working_dir).await
        };
        if let Some(d) = scan_dir {
            tracing::info!("Local file scan: {:?}", d);
            let file_models = collect_models(&d).await;
            for m in file_models {
                if !models.contains(&m) {
                    models.push(m);
                }
            }
        }
    } else if cfg.provider == "openai" {
        if !cfg.api_key.is_empty() {
            if let Ok(resp) = client
                .get(format!("{}/v1/models", base))
                .header("Authorization", format!("Bearer {}", cfg.api_key))
                .send()
                .await
            {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(arr) = json["data"].as_array() {
                        for m in arr {
                            if let Some(id) = m["id"].as_str() {
                                if id.starts_with("gpt-") || id.starts_with("o") {
                                    // Skip known deprecated
                                    if !id.contains("preview")
                                        && !id.contains("deprecated")
                                        && !id.contains("dall-e")
                                        && !id.contains("tts")
                                        && !id.contains("whisper")
                                        && !id.contains("embedding")
                                        && !id.contains("moderation")
                                        && !id.contains("babbage")
                                        && !id.contains("davinci")
                                        && !id.contains("ada")
                                        && !id.contains("curie")
                                    {
                                        models.push(id.to_string());
                                    }
                                }
                            }
                        }
                        models.sort();
                        models.dedup();
                    }
                }
            }
        }
        // Fallback if fetch fails
        if models.is_empty() {
            models.extend(
                [
                    "gpt-5.4",
                    "gpt-5.4-pro",
                    "gpt-5.4-mini",
                    "gpt-5.4-nano",
                    "gpt-5-mini",
                    "gpt-5-nano",
                    "gpt-5",
                    "gpt-4.1",
                    "gpt-5.2",
                    "gpt-5.1",
                    "gpt-5.2-pro",
                    "gpt-5-pro",
                    "o3-pro",
                    "o3",
                    "o4-mini",
                    "gpt-4.1-mini",
                    "gpt-4.1-nano",
                    "o1-pro",
                    "o3-mini",
                    "o1",
                    "gpt-4o",
                    "gpt-4o-mini",
                    "gpt-4-turbo",
                    "gpt-3.5-turbo",
                    "gpt-4",
                ]
                .iter()
                .map(|s| s.to_string()),
            );
        }
    } else if cfg.provider == "anthropic" {
        models.extend(
            [
                "claude-sonnet-4-20250514",
                "claude-opus-4-20250514",
                "claude-haiku-4-20241022",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
    } else if cfg.provider == "xai" {
        models.extend(
            [
                "grok-4-1-fast-reasoning",
                "grok-4-1-fast-non-reasoning",
                "grok-4.20-0309-reasoning",
                "grok-4.20-0309-non-reasoning",
                "grok-4.20-multi-agent-0309",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
    } else if cfg.provider == "google" {
        models.extend(["gemini-2.5-pro","gemini-2.5-flash","gemini-2.0-flash","gemini-2.0-flash-lite","gemini-1.5-pro"].iter().map(|s| s.to_string()));
    } else if cfg.provider == "deepseek" {
        models.extend(["deepseek-chat","deepseek-reasoner","deepseek-coder"].iter().map(|s| s.to_string()));
    } else if cfg.provider == "groq" {
        models.extend(["llama-3.3-70b-versatile","llama-3.1-8b-instant","llama-3.1-70b-versatile","mixtral-8x7b-32768","gemma2-9b-it","deepseek-r1-distill-llama-70b"].iter().map(|s| s.to_string()));
    } else if cfg.provider == "mistral" {
        models.extend(["mistral-large-latest","mistral-medium-latest","mistral-small-latest","codestral-latest","open-mistral-nemo","open-codestral-mamba"].iter().map(|s| s.to_string()));
    } else if cfg.provider == "cohere" {
        models.extend(["command-r-plus","command-r","command-a-03-2025"].iter().map(|s| s.to_string()));
    } else if cfg.provider == "together" {
        models.extend(["meta-llama/Llama-3.3-70B-Instruct-Turbo","meta-llama/Meta-Llama-3.1-405B-Instruct-Turbo","deepseek-ai/DeepSeek-R1","Qwen/Qwen2.5-72B-Instruct-Turbo","mistralai/Mixtral-8x22B-Instruct-v0.1"].iter().map(|s| s.to_string()));
    } else if cfg.provider == "fireworks" {
        models.extend(["accounts/fireworks/models/llama-v3p3-70b-instruct","accounts/fireworks/models/mixtral-8x22b-instruct","accounts/fireworks/models/deepseek-r1"].iter().map(|s| s.to_string()));
    } else if cfg.provider == "perplexity" {
        models.extend(["sonar-pro","sonar","sonar-deep-research","sonar-reasoning-pro"].iter().map(|s| s.to_string()));
    } else if cfg.provider == "openrouter" {
        models.extend(["openai/gpt-4o","anthropic/claude-sonnet-4","google/gemini-2.5-pro","meta-llama/llama-3.3-70b-instruct","deepseek/deepseek-r1"].iter().map(|s| s.to_string()));
    } else if cfg.provider == "amni" {
        let amni_base = amni_base_url(&base);
        if let Ok(resp) = client.get(format!("{}/v1/models", amni_base)).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(arr) = json["data"].as_array() {
                    for m in arr {
                        if m["available"].as_bool().unwrap_or(true) {
                            if let Some(id) = m["id"].as_str() { models.push(id.to_string()); }
                        }
                    }
                }
            }
        }
    }
    tracing::info!("Model discovery found {} models", models.len());
    Json(ModelsRes { models })
}
async fn handle_dirs(
    axum::extract::Query(q): axum::extract::Query<DirQuery>,
    State(app): State<App>,
) -> Json<DirsRes> {
    let base = match q.path {
        Some(p) => p,
        None => app.config.lock().await.working_dir.clone(),
    };
    let mut dirs = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(&base).await {
        while let Ok(Some(e)) = rd.next_entry().await {
            if e.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                let n = e.file_name().to_string_lossy().to_string();
                if !n.starts_with('.') {
                    dirs.push(n);
                }
            }
        }
    }
    dirs.sort();
    Json(DirsRes { dirs })
}
async fn handle_ls(
    axum::extract::Query(q): axum::extract::Query<DirQuery>,
    State(app): State<App>,
) -> Json<LsRes> {
    let base = match q.path {
        Some(p) => p,
        None => app.config.lock().await.working_dir.clone(),
    };
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(&base).await {
        while let Ok(Some(e)) = rd.next_entry().await {
            let n = e.file_name().to_string_lossy().to_string();
            if n.starts_with('.') { continue; }
            if e.file_type().await.map(|t| t.is_dir()).unwrap_or(false) { dirs.push(n); } else { files.push(n); }
        }
    }
    dirs.sort();
    files.sort();
    Json(LsRes { dirs, files })
}
async fn handle_read_file(
    axum::extract::Query(q): axum::extract::Query<DirQuery>,
) -> Json<ReadFileRes> {
    let path = q.path.unwrap_or_default();
    let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    Json(ReadFileRes { content })
}
async fn handle_write_file(
    Json(req): Json<WriteFileReq>,
) -> Json<serde_json::Value> {
    match tokio::fs::write(&req.path, &req.content).await {
        Ok(_) => Json(serde_json::json!({"ok":true})),
        Err(e) => Json(serde_json::json!({"ok":false,"error":e.to_string()})),
    }
}
async fn find_gguf_path(model_dir: &Path, model_name: &str) -> Option<PathBuf> {
    let target_gguf = format!("{}.gguf", model_name);
    let mut stack: Vec<(PathBuf, u32)> = vec![(model_dir.to_path_buf(), 0)];
    let mut best_match = None;
    let root_name = model_dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
    while let Some((current_dir, depth)) = stack.pop() {
        if depth > 3 {
            continue;
        }
        if let Ok(mut rd) = tokio::fs::read_dir(&current_dir).await {
            let dname = current_dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            while let Ok(Some(e)) = rd.next_entry().await {
                let path = e.path();
                if path.is_dir() {
                    let child_dname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !child_dname.starts_with('.')
                        && child_dname != "palace_textures"
                        && child_dname != "__pycache__"
                        && child_dname != "node_modules"
                    {
                        stack.push((path, depth + 1));
                    }
                } else if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                    if fname == target_gguf.as_str() {
                        return Some(path);
                    }
                    if let Some(stem) = fname.strip_suffix(".gguf") {
                        if let Some(id) = gguf_model_id(stem, &dname, &root_name) {
                            if id == model_name {
                                best_match = Some(path.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    best_match
}
async fn find_model_path_for_ollama(model_dir: &Path, model_name: &str) -> Option<PathBuf> {
    let target_gguf = format!("{}.gguf", model_name);
    let mut stack: Vec<(PathBuf, u32)> = vec![(model_dir.to_path_buf(), 0)];
    let mut best_match = None;
    let root_name = model_dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
    while let Some((current_dir, depth)) = stack.pop() {
        if depth > 3 {
            continue;
        }
        if let Ok(mut rd) = tokio::fs::read_dir(&current_dir).await {
            let dname = current_dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let mut has_safetensors = false;
            let mut subdirs = Vec::new();
            while let Ok(Some(e)) = rd.next_entry().await {
                let path = e.path();
                if path.is_dir() {
                    let child_dname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !child_dname.starts_with('.')
                        && child_dname != "palace_textures"
                        && child_dname != "__pycache__"
                        && child_dname != "node_modules"
                    {
                        subdirs.push((path, depth + 1));
                    }
                } else if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                    if fname.ends_with(".safetensors") {
                        has_safetensors = true;
                    }
                    if fname == target_gguf.as_str() {
                        return Some(path);
                    }
                    if let Some(stem) = fname.strip_suffix(".gguf") {
                        if let Some(id) = gguf_model_id(stem, &dname, &root_name) {
                            if id == model_name {
                                best_match = Some(path.clone());
                            }
                        }
                    }
                }
            }
            if has_safetensors && dname == model_name && dname != root_name {
                return Some(current_dir);
            }
            stack.extend(subdirs);
        }
    }
    best_match
}
async fn ensure_model_loaded(config: &Config) {
    if config.provider != "ollama" {
        return;
    }
    let model_dir = if !config.model_dir.is_empty() {
        Some(PathBuf::from(&config.model_dir))
    } else {
        auto_detect_model_dir(&config.working_dir).await
    };
    let dir = match model_dir {
        Some(d) => d,
        None => return,
    };
    let model_path = match find_model_path_for_ollama(&dir, &config.model).await {
        Some(p) => p,
        None => return,
    };
    tracing::info!(
        "Model '{}' has file/dir: {:?} — checking Ollama",
        config.model,
        model_path
    );
    let base = config.base_url.trim_end_matches('/');
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_default();
    let already_exists = client
        .post(format!("{}/api/show", base))
        .json(&serde_json::json!({"model": config.model}))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    if already_exists {
        tracing::info!("Model '{}' already exists in Ollama", config.model);
        return;
    }
    tracing::info!(
        "Importing '{}' into Ollama from {:?}",
        config.model,
        model_path
    );
    let canonical_path = model_path.canonicalize().unwrap_or(model_path);
    let path_str = canonical_path.display().to_string().replace('\\', "/");
    match client
        .post(format!("{}/api/create", base))
        .json(&serde_json::json!({"model": config.model, "from": path_str, "stream": false}))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            tracing::info!("Imported '{}' into Ollama successfully", config.model);
        }
        Ok(resp) => {
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!("Ollama import failed for '{}': {}", config.model, text);
        }
        Err(e) => tracing::warn!("Ollama import request failed: {}", e),
    }
}
async fn auto_detect_model_dir(working_dir: &str) -> Option<PathBuf> {
    let candidates = [
        PathBuf::from(working_dir).join("models"),
        PathBuf::from(working_dir).join("Amni-Ai").join("models"),
        PathBuf::from(working_dir)
            .parent()
            .map(|p| p.join("models"))
            .unwrap_or_default(),
        dirs::home_dir()
            .map(|h| h.join("models"))
            .unwrap_or_default(),
        dirs::home_dir()
            .map(|h| h.join(".cache").join("huggingface").join("hub"))
            .unwrap_or_default(),
    ];
    for c in &candidates {
        if !c.as_os_str().is_empty() && tokio::fs::metadata(c).await.is_ok() {
            tracing::info!("Auto-detected model dir: {:?}", c);
            return Some(c.clone());
        }
    }
    None
}
/// For a GGUF stem like "model-00001-of-00004" returns Some("model") for shard 1, None for others.
/// For unsplit files like "Qwen3.5-9B-Q8_0" returns Some("Qwen3.5-9B-Q8_0").
fn gguf_model_id(stem: &str, dir_name: &str, root_name: &str) -> Option<String> {
    // Detect pattern: <base>[-_.]<NNNNN>-of-<MMMMM>
    if let Some(of_pos) = stem.rfind("-of-") {
        let after_of = &stem[of_pos + 4..];
        if !after_of.is_empty() && after_of.chars().all(|c| c.is_ascii_digit()) {
            let before_of = &stem[..of_pos];
            let mut shard_len = 0;
            for c in before_of.chars().rev() {
                if c.is_ascii_digit() {
                    shard_len += 1;
                } else {
                    break;
                }
            }
            if shard_len > 0 {
                let sep_idx = before_of.len() - shard_len;
                if sep_idx > 0 {
                    let sep = before_of[sep_idx - 1..sep_idx].chars().next().unwrap();
                    if sep == '-' || sep == '_' || sep == '.' {
                        let shard_str = &before_of[sep_idx..];
                        let shard_n: u32 = shard_str.parse().unwrap_or(99);
                        let mut base = before_of[..sep_idx - 1].to_string();
                        if base == "model" || base.is_empty() {
                            if !dir_name.is_empty() && dir_name != root_name {
                                base = dir_name.to_string();
                            }
                        }
                        return if shard_n <= 1 { Some(base) } else { None }; // skip non-first shards
                    }
                }
            }
        }
    }
    Some(stem.to_string())
}
async fn collect_models(dir: &PathBuf) -> Vec<String> {
    let mut models = Vec::new();
    let mut stack: Vec<(PathBuf, u32)> = vec![(dir.clone(), 0)];
    let root_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
    while let Some((current_dir, depth)) = stack.pop() {
        if depth > 3 { continue; }
        if let Ok(mut rd) = tokio::fs::read_dir(&current_dir).await {
            // Collect all entries first so we can check for safetensors before deciding
            let dname = current_dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let mut subdirs: Vec<PathBuf> = Vec::new();
            let mut gguf_stems: Vec<String> = Vec::new();
            let mut has_safetensors = false;
            while let Ok(Some(e)) = rd.next_entry().await {
                let path = e.path();
                let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if path.is_dir() {
                    if fname.starts_with('.') || fname == "__pycache__" || fname == "node_modules" || fname == "palace_textures" { continue; }
                    // PTEX directory
                    let manifest = path.join("manifest.json");
                    if manifest.exists() {
                        if let Ok(s) = tokio::fs::read_to_string(&manifest).await {
                            if let Ok(j) = serde_json::from_str::<serde_json::Value>(&s) {
                                if j["format"].as_str() == Some("ptex") {
                                    if !models.contains(&fname) { models.push(fname); }
                                    continue;
                                }
                            }
                        }
                    }
                    subdirs.push(path);
                } else if fname.ends_with(".safetensors") {
                    has_safetensors = true;
                } else if let Some(stem) = fname.strip_suffix(".gguf") {
                    gguf_stems.push(stem.to_string());
                }
            }
            if has_safetensors {
                if !dname.is_empty() && dname != root_name {
                    if !models.contains(&dname) { models.push(dname); }
                }
                // Don't recurse — this entire dir is one model
            } else {
                for stem in gguf_stems {
                    if let Some(id) = gguf_model_id(&stem, &dname, &root_name) {
                        if !models.contains(&id) { models.push(id); }
                    }
                }
                for sub in subdirs { stack.push((sub, depth + 1)); }
            }
        }
    }
    models.sort();
    models
}
#[derive(Deserialize)]
struct HfSearchQuery {
    q: Option<String>,
}
#[derive(Serialize)]
struct HfModelResult {
    id: String,
    downloads: u64,
    likes: u64,
    tags: Vec<String>,
}
async fn handle_hf_search(Query(q): Query<HfSearchQuery>) -> Json<Vec<HfModelResult>> {
    let query = q.q.unwrap_or_default();
    if query.is_empty() {
        return Json(vec![]);
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    let url = format!(
        "https://huggingface.co/api/models?search={}&filter=gguf&sort=downloads&direction=-1&limit=20",
        urlencoding::encode(&query)
    );
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("HF search error: {}", e);
            return Json(vec![]);
        }
    };
    let json: Vec<serde_json::Value> = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Json(vec![]),
    };
    let results: Vec<HfModelResult> = json
        .into_iter()
        .map(|m| HfModelResult {
            id: m["id"].as_str().unwrap_or("").to_string(),
            downloads: m["downloads"].as_u64().unwrap_or(0),
            likes: m["likes"].as_u64().unwrap_or(0),
            tags: m["tags"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|t| t.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        })
        .filter(|m| !m.id.is_empty())
        .collect();
    Json(results)
}
#[derive(Deserialize)]
struct HfFilesQuery {
    repo: Option<String>,
}
#[derive(Serialize)]
struct HfFileInfo {
    name: String,
    size: u64,
}
async fn handle_hf_files(Query(q): Query<HfFilesQuery>) -> Json<Vec<HfFileInfo>> {
    let repo = q.repo.unwrap_or_default();
    if repo.is_empty() {
        return Json(vec![]);
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    let url = format!(
        "https://huggingface.co/api/models/{}/tree/main",
        repo
    );
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("HF files error: {}", e);
            return Json(vec![]);
        }
    };
    let json: Vec<serde_json::Value> = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Json(vec![]),
    };
    let files: Vec<HfFileInfo> = json
        .iter()
        .filter_map(|f| {
            let name = f["path"].as_str()?;
            if name.ends_with(".gguf") {
                Some(HfFileInfo {
                    name: name.to_string(),
                    size: f["size"].as_u64().unwrap_or(0),
                })
            } else {
                None
            }
        })
        .collect();
    Json(files)
}
#[derive(Deserialize)]
struct HfDownloadReq {
    repo: String,
    file: String,
}
async fn handle_hf_download(
    State(app): State<App>,
    Json(req): Json<HfDownloadReq>,
) -> Json<serde_json::Value> {
    let cfg = app.config.lock().await.clone();
    let dest_dir = if !cfg.model_dir.is_empty() {
        PathBuf::from(&cfg.model_dir)
    } else {
        match auto_detect_model_dir(&cfg.working_dir).await {
            Some(d) => d,
            None => {
                let fallback = dirs::home_dir()
                    .map(|h| h.join("models"))
                    .unwrap_or_else(|| PathBuf::from("models"));
                let _ = tokio::fs::create_dir_all(&fallback).await;
                fallback
            }
        }
    };
    let _ = tokio::fs::create_dir_all(&dest_dir).await;
    let dest_file = dest_dir.join(&req.file);
    if dest_file.exists() {
        return Json(serde_json::json!({"status": "exists", "path": dest_file.display().to_string()}));
    }
    {
        let mut prog = app.dl_progress.lock().await;
        if !prog.done && prog.total > 0 && prog.downloaded < prog.total {
            return Json(
                serde_json::json!({"status": "busy", "message": "A download is already in progress"}),
            );
        }
        *prog = DownloadProgress {
            repo: req.repo.clone(),
            file: req.file.clone(),
            downloaded: 0,
            total: 0,
            done: false,
            error: String::new(),
        };
    }
    let progress = app.dl_progress.clone();
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        req.repo, req.file
    );
    let dest_path_str = dest_file.display().to_string();
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(7200))
            .build()
            .unwrap_or_default();
        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                let mut p = progress.lock().await;
                p.done = true;
                p.error = format!("Request failed: {}", e);
                return;
            }
        };
        if !resp.status().is_success() {
            let mut p = progress.lock().await;
            p.done = true;
            p.error = format!("HTTP {}", resp.status());
            return;
        }
        let total = resp.content_length().unwrap_or(0);
        {
            let mut p = progress.lock().await;
            p.total = total;
        }
        let tmp_path = dest_file.with_extension("gguf.part");
        let mut file = match tokio::fs::File::create(&tmp_path).await {
            Ok(f) => f,
            Err(e) => {
                let mut p = progress.lock().await;
                p.done = true;
                p.error = format!("File create error: {}", e);
                return;
            }
        };
        use tokio::io::AsyncWriteExt;
        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    if file.write_all(&bytes).await.is_err() {
                        let mut p = progress.lock().await;
                        p.done = true;
                        p.error = "Write error".to_string();
                        return;
                    }
                    downloaded += bytes.len() as u64;
                    let mut p = progress.lock().await;
                    p.downloaded = downloaded;
                }
                Err(e) => {
                    let mut p = progress.lock().await;
                    p.done = true;
                    p.error = format!("Stream error: {}", e);
                    return;
                }
            }
        }
        let _ = file.flush().await;
        drop(file);
        if let Err(e) = tokio::fs::rename(&tmp_path, &dest_file).await {
            let mut p = progress.lock().await;
            p.done = true;
            p.error = format!("Rename error: {}", e);
            return;
        }
        let mut p = progress.lock().await;
        p.downloaded = total;
        p.done = true;
        tracing::info!("Download complete: {:?}", dest_file);
    });
    Json(serde_json::json!({"status": "started", "path": dest_path_str}))
}
async fn handle_hf_progress(State(app): State<App>) -> Json<DownloadProgress> {
    Json(app.dl_progress.lock().await.clone())
}
async fn handle_health() -> &'static str {
    "ok"
}
async fn handle_fs_events(State(app): State<App>) -> Sse<impl futures::Stream<Item = Result<SseEvent, Infallible>> + Send + 'static> {
    let mut rx = app.fs_tx.subscribe();
    let (tx, out_rx) = tokio::sync::mpsc::channel::<SseEvent>(64);
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(data) => { let _ = tx.send(SseEvent::default().event("fs").data(data)).await; }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });
    Sse::new(ReceiverStream::new(out_rx).map(|e| Ok::<_, Infallible>(e))).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(30)))
}
async fn handle_memory_list(State(app): State<App>) -> Json<serde_json::Value> {
    let mem = app.memory.lock().await;
    let keys: Vec<&String> = mem.keys().collect();
    Json(serde_json::json!({"keys": keys, "count": keys.len()}))
}
async fn serve_ui() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cwd = std::env::current_dir().unwrap_or_default();
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe_path = std::env::current_exe().unwrap_or_default();
    let get_hash = |dir: &std::path::Path| std::process::Command::new("git").args(["rev-parse","HEAD"]).current_dir(dir).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let hash_before = get_hash(src_dir);
    let _ = std::process::Command::new("git").args(["pull","--ff-only","--quiet"]).current_dir(src_dir).status();
    let hash_after = get_hash(src_dir);
    if hash_before.as_deref() != hash_after.as_deref() {
        println!("  New version detected, rebuilding...");
        let ok = std::process::Command::new("cargo").args(["install","--path",".","--force","--quiet"]).current_dir(src_dir).status().map(|s|s.success()).unwrap_or(false);
        if ok { println!("  Restarting..."); let _ = std::process::Command::new(&exe_path).args(std::env::args().skip(1)).status(); std::process::exit(0); }
    }
    let home_env = dirs::home_dir()
        .map(|h| h.join(".amni").join(".env"))
        .unwrap_or_default();
    for env_path in [home_env, cwd.join(".env")] {
        if env_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&env_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((k, v)) = line.split_once('=') {
                        std::env::set_var(k.trim(), v.trim());
                    }
                }
            }
        }
    }
    let config_path = dirs::home_dir()
        .map(|h| h.join(".amni").join("config.json"))
        .unwrap_or_default();
    let mut config: Config = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(Config::default)
    } else {
        Config::default()
    };
    let base_before = config.base_url.clone();
    normalize_config(&mut config);
    if config.base_url != base_before { persist_config(&config); }
    if config.working_dir.is_empty() { config.working_dir = cwd.to_string_lossy().into(); }
    // CLI arg: first non-flag argument sets working directory
    let cli_dir: Option<String> = std::env::args().skip(1).find(|a| !a.starts_with('-') && std::fs::metadata(a).map(|m| m.is_dir()).unwrap_or(false));
    if let Some(ref d) = cli_dir {
        let p = std::fs::canonicalize(d).unwrap_or_else(|_| PathBuf::from(d));
        config.working_dir = p.to_string_lossy().to_string();
    }
    let effective_cwd = PathBuf::from(&config.working_dir);
    let custom_inst = load_custom_instructions(&effective_cwd);
    println!("\n  Amni-Code v2.2.0 — AI Coding Agent");
    println!("  Working dir: {}", effective_cwd.display());
    if !custom_inst.is_empty() { println!("  Custom instructions: loaded from project"); }
    let memory = load_memory_store(&effective_cwd);
    if !memory.is_empty() { println!("  Memory: {} keys loaded", memory.len()); }
    let (server_log_tx,_)=broadcast::channel::<String>(512);
    let (fs_tx,_)=broadcast::channel::<String>(256);
    let app=App{sessions:Arc::new(Mutex::new(HashMap::new())),config:Arc::new(Mutex::new(config)),cwd:Arc::new(Mutex::new(effective_cwd.clone())),dl_progress:Arc::new(Mutex::new(DownloadProgress::default())),backups:Arc::new(Mutex::new(HashMap::new())),interrupts:Arc::new(Mutex::new(HashMap::new())),amni_proc:Arc::new(Mutex::new(None)),server_log_tx:Arc::new(server_log_tx),memory:Arc::new(Mutex::new(memory)),fs_tx:Arc::new(fs_tx.clone())};
    // File watcher — polls for changes every 2s and broadcasts via fs_tx
    let watch_dir = effective_cwd.clone();
    let fs_tx_clone = fs_tx.clone();
    tokio::spawn(async move {
        let mut snapshot: HashMap<String, u64> = HashMap::new();
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let mut current: HashMap<String, u64> = HashMap::new();
            if let Ok(mut rd) = tokio::fs::read_dir(&watch_dir).await {
                while let Ok(Some(e)) = rd.next_entry().await {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') || name == "node_modules" || name == "__pycache__" || name == "target" { continue; }
                    if let Ok(meta) = e.metadata().await {
                        let mtime = meta.modified().ok().and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
                        current.insert(name.clone(), mtime);
                    }
                }
            }
            let mut changes = Vec::new();
            for (name, mtime) in &current {
                match snapshot.get(name) {
                    None => changes.push(format!("added:{}", name)),
                    Some(old) if old != mtime => changes.push(format!("modified:{}", name)),
                    _ => {}
                }
            }
            for name in snapshot.keys() {
                if !current.contains_key(name) { changes.push(format!("deleted:{}", name)); }
            }
            if !changes.is_empty() {
                let payload = serde_json::json!({"changes": changes}).to_string();
                let _ = fs_tx_clone.send(payload);
            }
            snapshot = current;
        }
    });
    let router = Router::new()
        .route("/", get(serve_ui))
        .route("/api/chat", post(handle_chat))
        .route(
            "/api/config",
            get(handle_config_get).post(handle_config_set),
        )
        .route("/api/undo", post(handle_undo))
        .route("/api/accept", post(handle_accept))
        .route("/api/interrupt", post(handle_interrupt))
        .route("/api/steer", post(handle_steer))
        .route("/api/server-start", post(handle_server_start))
        .route("/api/server-log", get(handle_server_log))
        .route("/api/server-status", get(handle_server_status))
        .route("/api/models", get(handle_models))
        .route("/api/dirs", get(handle_dirs))
        .route("/api/ls", get(handle_ls))
        .route("/api/read_file", get(handle_read_file))
        .route("/api/write_file", post(handle_write_file))
        .route("/api/hf/search", get(handle_hf_search))
        .route("/api/hf/files", get(handle_hf_files))
        .route("/api/hf/download", post(handle_hf_download))
        .route("/api/hf/progress", get(handle_hf_progress))
        .route("/api/fs-events", get(handle_fs_events))
        .route("/api/memory", get(handle_memory_list))
        .route("/health", get(handle_health))
        .layer(CorsLayer::permissive())
        .with_state(app);
    let use_browser = std::env::args().any(|a| a == "--browser");
    if use_browser {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        println!("  Server: http://localhost:3000");
        println!("  Opening browser...\n");
        let _ = open::that("http://localhost:3000");
        axum::serve(listener, router).await?;
    } else {
        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            println!("  Server: http://localhost:3000\n");
            axum::serve(listener, router).await.unwrap();
        });
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        use tao::event::{Event, WindowEvent};
        use tao::event_loop::{ControlFlow, EventLoop};
        use tao::window::WindowBuilder;
        use wry::WebViewBuilder;
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Amni-Code — AI Coding Agent")
            .with_inner_size(tao::dpi::LogicalSize::new(1200.0, 800.0))
            .build(&event_loop)
            .unwrap();
        let _webview = WebViewBuilder::new()
            .with_url("http://localhost:3000")
            .with_devtools(true)
            .build(&window)
            .unwrap();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                *control_flow = ControlFlow::Exit;
            }
        });
    }
    Ok(())
}
#[cfg(test)]
mod git_tool_tests {
    use super::*;
    #[tokio::test]
    async fn git_tools_roundtrip() {
        let dir = std::env::temp_dir().join(format!("amni_git_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let cwd = dir.clone();
        let _ = git_run(&cwd, &["init"]).await;
        let _ = git_run(&cwd, &["config", "user.email", "test@amni.local"]).await;
        let _ = git_run(&cwd, &["config", "user.name", "Amni Test"]).await;
        std::fs::write(dir.join("hello.txt"), "hi").unwrap();
        let (s, st) = exec_tool("git_status", &serde_json::json!({}), &cwd).await;
        assert_eq!(st, "success");
        assert!(s.contains("hello.txt"), "status should list hello.txt, got: {}", s);
        let (_, st2) = exec_tool("git_add", &serde_json::json!({"path": "."}), &cwd).await;
        assert_eq!(st2, "success");
        let (c, st3) = exec_tool("git_commit", &serde_json::json!({"message": "first commit"}), &cwd).await;
        assert_eq!(st3, "success", "commit failed: {}", c);
        let (lg, st4) = exec_tool("git_log", &serde_json::json!({"count": 5}), &cwd).await;
        assert_eq!(st4, "success");
        assert!(lg.contains("first commit"), "log should contain the commit, got: {}", lg);
        let (_, st5) = exec_tool("git_commit", &serde_json::json!({"message": "   "}), &cwd).await;
        assert_eq!(st5, "error", "empty-message commit must be rejected");
        let (u, st6) = exec_tool("definitely_not_a_tool", &serde_json::json!({}), &cwd).await;
        assert_eq!(st6, "error");
        assert!(u.contains("Unknown tool"));
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[test]
    fn mode_gate_classification() {
        for t in ["read_file","list_directory","search_files","git_status","git_diff","git_log","web_fetch","web_search","memory_read"] {
            assert!(!is_mutating_tool(t), "{} should be read-only (allowed in plan mode)", t);
        }
        for t in ["write_file","edit_file","run_command","git_add","git_commit","memory_write"] {
            assert!(is_mutating_tool(t), "{} should be mutating (gated in plan mode)", t);
        }
        let gated = |mode: &str, tool: &str| mode == "plan" && is_mutating_tool(tool);
        assert!(gated("plan", "write_file"), "plan mode must gate write_file");
        assert!(!gated("plan", "read_file"), "plan mode must allow read_file");
        assert!(!gated("edit", "write_file"), "edit mode executes write_file");
        assert!(!gated("autonomous", "run_command"), "autonomous executes run_command");
    }
    #[test]
    fn test_command_detection() {
        let base = std::env::temp_dir().join(format!("amni_tc_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let mk = |sub: &str, marker: &str| -> PathBuf {
            let d = base.join(sub);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join(marker), "x").unwrap();
            d
        };
        assert_eq!(detect_test_command(&mk("rust", "Cargo.toml")), Some("cargo test"));
        assert_eq!(detect_test_command(&mk("go", "go.mod")), Some("go test ./..."));
        assert_eq!(detect_test_command(&mk("py", "pyproject.toml")), Some("python -m pytest -q"));
        assert_eq!(detect_test_command(&mk("js", "package.json")), Some("npm test"));
        let empty = base.join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(detect_test_command(&empty), None);
        let _ = std::fs::remove_dir_all(&base);
    }
    #[test]
    fn slash_command_parsing() {
        assert_eq!(parse_slash("/test"), Some(SlashCmd::Tool("run_tests", serde_json::json!({}))));
        assert_eq!(parse_slash("/test cargo test --lib"), Some(SlashCmd::Tool("run_tests", serde_json::json!({"command":"cargo test --lib"}))));
        assert_eq!(parse_slash("/status"), Some(SlashCmd::Tool("git_status", serde_json::json!({}))));
        assert_eq!(parse_slash("/diff src/main.rs"), Some(SlashCmd::Tool("git_diff", serde_json::json!({"path":"src/main.rs"}))));
        assert_eq!(parse_slash("/log"), Some(SlashCmd::Tool("git_log", serde_json::json!({}))));
        assert_eq!(parse_slash("/plan"), Some(SlashCmd::SetMode("plan")));
        assert_eq!(parse_slash("/auto"), Some(SlashCmd::SetMode("autonomous")));
        assert_eq!(parse_slash("/help"), Some(SlashCmd::Help));
        assert!(matches!(parse_slash("/commit fix the bug"), Some(SlashCmd::Prompt(_))));
        assert!(matches!(parse_slash("/review"), Some(SlashCmd::Prompt(_))));
        assert_eq!(parse_slash("hello world"), None);
        assert_eq!(parse_slash("/unknowncmd"), None);
        assert_eq!(parse_slash("not /a slash"), None);
    }
    #[test]
    fn lint_command_detection() {
        let base = std::env::temp_dir().join(format!("amni_lint_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let mk = |sub: &str, marker: &str| -> PathBuf {
            let d = base.join(sub);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join(marker), "x").unwrap();
            d
        };
        assert_eq!(detect_lint_command(&mk("rust", "Cargo.toml")), Some("cargo clippy --quiet --all-targets"));
        assert_eq!(detect_lint_command(&mk("go", "go.mod")), Some("go vet ./..."));
        assert_eq!(detect_lint_command(&mk("py", "pyproject.toml")), Some("ruff check ."));
        assert_eq!(detect_lint_command(&mk("js", ".eslintrc.json")), Some("npx --no-install eslint ."));
        assert_eq!(detect_lint_command(&mk("npm", "package.json")), Some("npm run lint --if-present"));
        let empty = base.join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(detect_lint_command(&empty), None);
        let _ = std::fs::remove_dir_all(&base);
    }
    #[test]
    fn mention_extraction() {
        assert_eq!(extract_mentions("explain @src/main.rs please"), vec!["src/main.rs"]);
        assert_eq!(extract_mentions("@Cargo.toml and @README.md"), vec!["Cargo.toml", "README.md"]);
        assert_eq!(extract_mentions("look at @src/main.rs."), vec!["src/main.rs"]);
        assert_eq!(extract_mentions("@a/b/c.rs"), vec!["a/b/c.rs"]);
        assert!(extract_mentions("email me at foo@bar.com").is_empty());
        assert!(extract_mentions("just @someuser here").is_empty());
        assert!(extract_mentions("no mentions here").is_empty());
    }
}
