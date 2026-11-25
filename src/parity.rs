use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
pub const LOOP_LIMIT: u32 = 7;
pub const STEP_LIMIT: usize = 70;
pub const POLL_SECS: u64 = 7;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub detail: String,
}
#[derive(Default)]
pub struct State {
    pub context_used: usize,
    pub context_limit: usize,
    pub tasks: Vec<Task>,
    pub next_id: u64,
    pub term_log: String,
    pub code_script: Option<Value>,
}
impl State {
    pub fn new() -> Self {
        Self { context_limit: context_limit(), ..Self::default() }
    }
}
pub fn new_state() -> Arc<Mutex<State>> {
    Arc::new(Mutex::new(State::new()))
}
pub fn context_limit() -> usize {
    std::env::var("AMNI_CODE_CONTEXT").ok().and_then(|v| v.parse().ok()).unwrap_or(128_000)
}
pub fn estimate_tokens(messages: &[Value]) -> usize {
    let chars: usize = messages.iter().map(|m| match m.get("content") {
        Some(Value::String(s)) => s.len(),
        Some(Value::Array(a)) => a.iter().map(|x| x.get("text").and_then(|t| t.as_str()).map(|s| s.len()).unwrap_or(0)).sum(),
        _ => 0,
    }).sum();
    (chars + 3) / 4
}
pub fn set_usage(state: &mut State, used: usize) {
    state.context_used = used;
    state.context_limit = context_limit();
}
fn parse_kv_line(s: &str) -> Value {
    let mut o = serde_json::Map::new();
    o.insert("on".into(), json!(true));
    for part in s.split_whitespace() {
        if let Some((k, v)) = part.split_once('=') {
            match k {
                "entries" | "pages" => { if let Ok(n) = v.parse::<u64>() { o.insert(k.into(), json!(n)); } }
                "used" | "capacity" => {
                    let num = v.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.');
                    if let Ok(n) = num.parse::<f64>() { o.insert(format!("{k}_mb"), json!(n)); }
                }
                "fill" => {
                    let num = v.trim_end_matches('%');
                    if let Ok(n) = num.parse::<f64>() { o.insert("fill".into(), json!(n / 100.0)); }
                }
                "engine" => { o.insert("engine".into(), json!(v)); }
                _ => {}
            }
        }
    }
    Value::Object(o)
}
pub fn atex_stats() -> Value {
    let out = Command::new("atex").arg("stats").output();
    match out {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            let line = s.lines().find(|l| l.contains("entries=") || l.contains("fill=")).unwrap_or(s.trim());
            if line.is_empty() { json!({"on": true}) } else { parse_kv_line(line) }
        }
        Ok(_) => json!({"on": false}),
        Err(_) => json!({"on": false}),
    }
}
pub fn session_json(state: &State, code_mode: bool) -> Value {
    let limit = if state.context_limit == 0 { context_limit() } else { state.context_limit };
    let used = state.context_used;
    let fill = if limit == 0 { 0.0 } else { (used as f64 / limit as f64).clamp(0.0, 1.0) };
    let atex = atex_stats();
    let atex_on = atex.get("on").and_then(|x| x.as_bool()).unwrap_or(false);
    let atex_fill = atex.get("fill").and_then(|x| x.as_f64());
    let atex_label = if atex_on {
        match atex_fill { Some(f) => format!("ATEX on {}%", (f * 100.0).round() as i32), None => "ATEX on".into() }
    } else { "ATEX off".into() };
    json!({
        "context_used": used,
        "context_limit": limit,
        "context_fill": fill,
        "context_label": format!("{}% context", (fill * 100.0).round() as i32),
        "context_fill_css": format!("{:.3}", fill),
        "atex": atex,
        "atex_label": atex_label,
        "code_mode": code_mode,
        "script": state.code_script,
        "mods": [{"id": "amniscient", "label": "Amniscient", "default": true, "backdrop": "snow"}]
    })
}
fn add_broken(out: &mut Vec<Value>, path: &Path, reason: &str) {
    out.push(json!({"path": path.display().to_string(), "reason": reason}));
}
fn scan_json_file(out: &mut Vec<Value>, path: &Path) {
    match std::fs::read_to_string(path) {
        Err(e) => add_broken(out, path, &format!("unreadable: {e}")),
        Ok(s) if s.trim().is_empty() => add_broken(out, path, "empty"),
        Ok(s) => { if serde_json::from_str::<Value>(&s).is_err() { add_broken(out, path, "invalid JSON"); } }
    }
}
fn scan_dir_json(out: &mut Vec<Value>, dir: &Path, exts: &[&str]) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if let Some(n) = p.file_name().and_then(|s| s.to_str()) {
                if n == "node_modules" || n == "target" || n.starts_with('.') { continue; }
            }
            scan_dir_json(out, &p, exts);
        } else if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
            if exts.contains(&ext) { scan_json_file(out, &p); }
        }
    }
}
pub fn doctor_report() -> Value {
    let mut broken = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dirs = [
        home.join(".claude/agents"),
        home.join(".claude/skills"),
        home.join(".claude/commands"),
        home.join(".claude/hooks"),
        home.join(".cursor/skills"),
        home.join(".cursor/hooks"),
        cwd.join(".claude/agents"),
        cwd.join(".claude/skills"),
        cwd.join(".claude/commands"),
        cwd.join(".claude/hooks"),
        cwd.join(".cursor/skills"),
        cwd.join(".agents"),
        cwd.join("skills"),
    ];
    for d in &dirs {
        if d.is_dir() { scan_dir_json(&mut broken, d, &["json", "jsonc"]); }
    }
    for f in [home.join(".claude/settings.json"), cwd.join(".claude/settings.json"), cwd.join(".cursor/hooks.json")] {
        if f.is_file() { scan_json_file(&mut broken, &f); }
    }
    json!({"ok": broken.is_empty(), "broken": broken, "count": broken.len()})
}
fn git(cwd: &Path, args: &[&str]) -> (String, bool) {
    match Command::new("git").args(args).current_dir(cwd).output() {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).to_string();
            if !o.stderr.is_empty() {
                if !s.is_empty() { s.push('\n'); }
                s.push_str(&String::from_utf8_lossy(&o.stderr));
            }
            (s.trim().to_string(), o.status.success())
        }
        Err(e) => (format!("git not available: {e}"), false),
    }
}
pub fn current_branch(cwd: &Path) -> String {
    let (s, ok) = git(cwd, &["rev-parse", "--abbrev-ref", "HEAD"]);
    if ok && !s.is_empty() { s } else { "HEAD".into() }
}
pub fn branch_protected(name: &str) -> bool {
    matches!(name, "main" | "master" | "trunk" | "production")
}
pub fn repo_is_public(cwd: &Path) -> bool {
    let (out, ok) = match Command::new("gh").args(["repo", "view", "--json", "isPrivate"]).current_dir(cwd).output() {
        Ok(o) => (String::from_utf8_lossy(&o.stdout).to_string(), o.status.success()),
        Err(_) => return true,
    };
    if !ok { return true; }
    match serde_json::from_str::<Value>(&out) {
        Ok(v) => !v.get("isPrivate").and_then(|x| x.as_bool()).unwrap_or(false),
        Err(_) => true,
    }
}
pub fn can_auto_push(cwd: &Path) -> Result<String, String> {
    if repo_is_public(cwd) { return Err("public repo: will not auto-push".into()); }
    let br = current_branch(cwd);
    if branch_protected(&br) { return Err(format!("branch {br} is protected: will not auto-push")); }
    Ok(br)
}
pub fn git_dirty(cwd: &Path) -> bool {
    let (s, ok) = git(cwd, &["status", "--porcelain"]);
    ok && !s.is_empty()
}
pub fn ship_commit_and_pr(cwd: &Path, msg: &str) -> String {
    let mut log = String::new();
    if git_dirty(cwd) {
        let _ = git(cwd, &["add", "-A"]);
        let body = if msg.trim().is_empty() { "ship: review fixes" } else { msg.trim() };
        let (c, ok) = git(cwd, &["commit", "-m", body]);
        if ok { log.push_str("Committed local work.\n"); } else { log.push_str(&format!("Commit: {c}\n")); }
    } else {
        log.push_str("Working tree clean.\n");
    }
    match can_auto_push(cwd) {
        Ok(br) => {
            let (p, pok) = git(cwd, &["push", "-u", "origin", "HEAD"]);
            if pok { log.push_str(&format!("Pushed {br}.\n")); } else { log.push_str(&format!("Push: {p}\n")); }
            if pok {
                let has_pr = Command::new("gh").args(["pr", "view", "--json", "url"]).current_dir(cwd).status().map(|s| s.success()).unwrap_or(false);
                if !has_pr {
                    let t = if msg.trim().is_empty() { "ship: review fixes" } else { msg.trim() };
                    let pr = Command::new("gh").args(["pr", "create", "--title", t, "--body", "Opened by Amni-Code /ship."]).current_dir(cwd).output();
                    match pr {
                        Ok(o) if o.status.success() => { log.push_str(&format!("Opened PR:\n{}", String::from_utf8_lossy(&o.stdout))); }
                        Ok(o) => { log.push_str(&format!("PR: {}\n", String::from_utf8_lossy(&o.stderr))); }
                        Err(e) => { log.push_str(&format!("PR skipped: {e}\n")); }
                    }
                } else {
                    log.push_str("A PR already exists for this branch.\n");
                }
            }
        }
        Err(e) => { log.push_str(&format!("{e}. Left changes local.\n")); }
    }
    log
}
pub fn ship_prompt(user_arg: &str) -> String {
    let extra = if user_arg.trim().is_empty() { String::new() } else { format!(" The user notes: {}.", user_arg.trim()) };
    format!("Review the current changes with git_status and git_diff. Fix only real bugs you can confirm in the code. Do not invent issues.{extra} After fixes, summarize findings as 'file:line: issue -> fix'. Do not push to public remotes.")
}
pub fn pr_monitor_once(cwd: &Path) -> String {
    let mut parts = Vec::new();
    if let Ok(o) = Command::new("gh").args(["pr", "view", "--json", "url,mergeable,reviewDecision,statusCheckRollup,comments"]).current_dir(cwd).output() {
        if o.status.success() {
            if let Ok(v) = serde_json::from_str::<Value>(&String::from_utf8_lossy(&o.stdout)) {
                if let Some(m) = v.get("mergeable").and_then(|x| x.as_str()) { parts.push(format!("mergeable={m}")); }
                if let Some(r) = v.get("reviewDecision").and_then(|x| x.as_str()) { parts.push(format!("review={r}")); }
                if let Some(checks) = v.get("statusCheckRollup").and_then(|x| x.as_array()) {
                    let bad: Vec<String> = checks.iter().filter_map(|c| {
                        let st = c.get("state").or_else(|| c.get("conclusion")).and_then(|s| s.as_str()).unwrap_or("");
                        if matches!(st, "FAILURE" | "ERROR" | "TIMED_OUT" | "CANCELLED" | "failing") {
                            Some(c.get("name").or_else(|| c.get("context")).and_then(|n| n.as_str()).unwrap_or("check").to_string())
                        } else { None }
                    }).collect();
                    if bad.is_empty() { parts.push("github checks ok or none".into()); } else { parts.push(format!("failing: {}", bad.join(", "))); }
                }
                if let Some(comments) = v.get("comments").and_then(|x| x.as_array()) {
                    if !comments.is_empty() { parts.push(format!("comments={}", comments.len())); }
                }
            }
        }
    }
    if let Ok(o) = Command::new("git").args(["status", "--porcelain"]).current_dir(cwd).output() {
        let s = String::from_utf8_lossy(&o.stdout);
        if s.contains("UU ") || s.contains("AA ") { parts.push("merge conflicts".into()); }
    }
    if parts.is_empty() { "no PR status (local only or gh missing)".into() } else { parts.join("; ") }
}
pub fn next_task_id(state: &mut State) -> String {
    state.next_id += 1;
    format!("t{}", state.next_id)
}
pub fn add_task(state: &mut State, kind: &str, title: &str, detail: &str) -> String {
    let id = next_task_id(state);
    state.tasks.push(Task { id: id.clone(), kind: kind.into(), title: title.into(), status: "running".into(), detail: detail.into() });
    id
}
pub fn stop_task(state: &mut State, id: &str) -> bool {
    if let Some(t) = state.tasks.iter_mut().find(|t| t.id == id) {
        t.status = "stopped".into();
        true
    } else { false }
}
pub fn tasks_json(state: &State) -> Value {
    json!({"tasks": state.tasks})
}
pub fn changes_json(cwd: &Path) -> Value {
    let (diff, _) = git(cwd, &["diff", "--stat"]);
    let (stat, _) = git(cwd, &["status", "--short", "--branch"]);
    json!({"status": stat, "diff_stat": diff})
}
pub fn run_term(cwd: &Path, cmd: &str, log: &mut String) -> String {
    if cmd.trim().is_empty() { return "empty command".into(); }
    let out = Command::new("bash").args(["-lc", cmd]).current_dir(cwd).output();
    let text = match out {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).to_string();
            let e = String::from_utf8_lossy(&o.stderr);
            if !e.is_empty() { if !s.is_empty() { s.push('\n'); } s.push_str(&e); }
            if s.is_empty() { s = if o.status.success() { String::new() } else { format!("exit {}", o.status.code().unwrap_or(1)) }; }
            s
        }
        Err(e) => format!("exec failed: {e}"),
    };
    if !log.is_empty() { log.push('\n'); }
    log.push_str(&format!("$ {cmd}\n{text}"));
    if log.len() > 80_000 { let cut = log.len() - 60_000; log.replace_range(0..cut, "…\n"); }
    text
}
pub fn extract_code_script(text: &str) -> Option<Value> {
    for tag in ["```code-mode", "```amni-code", "```json"] {
        if let Some(rest) = text.split(tag).nth(1) {
            let body = rest.split("```").next().unwrap_or("").trim();
            if let Ok(v) = serde_json::from_str::<Value>(body) {
                if v.get("steps").and_then(|s| s.as_array()).is_some() { return Some(v); }
            }
        }
    }
    if let Ok(v) = serde_json::from_str::<Value>(text.trim()) {
        if v.get("steps").and_then(|s| s.as_array()).is_some() { return Some(v); }
    }
    None
}
pub const CODE_MODE_PROMPT: &str = "Code Mode is on. When a turn needs several tool calls, write one script in a code-mode fence whose JSON has a steps array. Each step is {\"id\",\"tool\",\"args\", optional \"parallel\":true, optional \"after\":[ids]}. The runner executes up to 70 steps with a runaway loop limit of 7 repeats of the same tool+args. Prefer reading only the output you need. Individual tool calls stay visible. Do not put secrets in the script.";
pub fn run_code_steps(script: &Value) -> Result<Vec<Value>, String> {
    let steps = script.get("steps").and_then(|s| s.as_array()).ok_or("script has no steps")?;
    if steps.len() > STEP_LIMIT { return Err(format!("script has {} steps; limit is {STEP_LIMIT}", steps.len())); }
    Ok(steps.clone())
}
pub fn local_test_hint(cwd: &Path) -> Option<&'static str> {
    if cwd.join("Cargo.toml").exists() { Some("cargo test --offline --color never") }
    else if cwd.join("package.json").exists() { Some("npm test --if-present") }
    else if cwd.join("pyproject.toml").exists() || cwd.join("pytest.ini").exists() { Some("python -m pytest -q") }
    else { None }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_estimate_and_fill() {
        let msgs = vec![json!({"role":"user","content":"abcd"}), json!({"role":"assistant","content":"efghijkl"})];
        assert_eq!(estimate_tokens(&msgs), 3);
        let mut s = State::new();
        set_usage(&mut s, 64000);
        s.context_limit = 128000;
        let j = session_json(&s, false);
        assert!((j["context_fill"].as_f64().unwrap() - 0.5).abs() < 0.01);
        assert!(j["atex"].is_object());
        assert_eq!(j["mods"][0]["id"], "amniscient");
    }
    #[test]
    fn public_default_and_protected() {
        assert!(branch_protected("main") && branch_protected("master"));
        assert!(!branch_protected("rikku/claude-parity"));
        assert!(extract_code_script("```code-mode\n{\"steps\":[{\"id\":\"1\",\"tool\":\"git_status\",\"args\":{}}]}\n```").is_some());
        assert!(extract_code_script("hello").is_none());
        let steps = run_code_steps(&json!({"steps":[{"id":"1"}]})).unwrap();
        assert_eq!(steps.len(), 1);
        assert!(run_code_steps(&json!({"steps":(0..80).map(|i| json!({"id":i})).collect::<Vec<_>>()})).is_err());
    }
    #[test]
    fn parse_atex_line() {
        let v = parse_kv_line("entries=3713 pages=110 used=111.31MB capacity=115.34MB fill=96.5% engine=gold/helium-native");
        assert_eq!(v["on"], json!(true));
        assert_eq!(v["entries"], json!(3713));
        assert!((v["fill"].as_f64().unwrap() - 0.965).abs() < 0.001);
    }
    #[test]
    fn task_stop() {
        let mut s = State::new();
        let id = add_task(&mut s, "background", "ship", "monitor");
        assert!(stop_task(&mut s, &id));
        assert_eq!(s.tasks[0].status, "stopped");
        assert!(!stop_task(&mut s, "nope"));
    }
}
