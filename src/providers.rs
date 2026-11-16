//! Provider table, request shaping, live model discovery, local-server detection.
//! Model ids in the fallback lists were checked against each provider's own docs on 2026-11-16
//! (see changelog v3.0.0). Live `/models` listings always win over these lists.
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Wire {
    OpenAi,
    Anthropic,
}

pub struct Provider {
    pub id: &'static str,
    pub label: &'static str,
    pub base: &'static str,
    pub chat_path: &'static str,
    pub models_path: &'static str,
    pub wire: Wire,
    pub cloud: bool,
    pub key_envs: &'static [&'static str],
    pub fallback: &'static [&'static str],
}

pub const PROVIDERS: &[Provider] = &[
    Provider { id: "amni", label: "Adam / Amni-AI (local)", base: "http://127.0.0.1:7700", chat_path: "/v1/chat/completions", models_path: "/v1/models", wire: Wire::OpenAi, cloud: false, key_envs: &[], fallback: &["adam:3b", "adam:8b"] },
    Provider { id: "ollama", label: "Ollama (local)", base: "http://127.0.0.1:11434", chat_path: "/v1/chat/completions", models_path: "/api/tags", wire: Wire::OpenAi, cloud: false, key_envs: &[], fallback: &[] },
    Provider { id: "local", label: "Custom local (OpenAI-compatible)", base: "http://127.0.0.1:8080", chat_path: "/v1/chat/completions", models_path: "/v1/models", wire: Wire::OpenAi, cloud: false, key_envs: &[], fallback: &[] },
    Provider { id: "anthropic", label: "Anthropic", base: "https://api.anthropic.com", chat_path: "/v1/messages", models_path: "/v1/models", wire: Wire::Anthropic, cloud: true, key_envs: &["ANTHROPIC_API_KEY"], fallback: &["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5", "claude-fable-5-2"] },
    Provider { id: "openai", label: "OpenAI", base: "https://api.openai.com", chat_path: "/v1/chat/completions", models_path: "/v1/models", wire: Wire::OpenAi, cloud: true, key_envs: &["OPENAI_API_KEY"], fallback: &["gpt-6-astra", "gpt-6.1-sol", "gpt-6-luna", "gpt-6.1-terra"] },
    Provider { id: "google", label: "Google Gemini", base: "https://generativelanguage.googleapis.com", chat_path: "/v1beta/openai/chat/completions", models_path: "/v1beta/openai/models", wire: Wire::OpenAi, cloud: true, key_envs: &["GEMINI_API_KEY", "GOOGLE_AI_API_KEY", "GOOGLE_API_KEY"], fallback: &["gemini-3.8-flash", "gemini-3.6-flash", "gemini-3.5-flash-lite", "gemini-3.1-pro-preview"] },
    Provider { id: "xai", label: "xAI Grok", base: "https://api.x.ai", chat_path: "/v1/chat/completions", models_path: "/v1/models", wire: Wire::OpenAi, cloud: true, key_envs: &["XAI_API_KEY", "GROK_API_KEY", "GROK_key", "xAI_key", "XAI_KEY"], fallback: &["grok-4.7", "grok-4.6", "grok-4.5", "grok-build-0.1"] },
    Provider { id: "deepseek", label: "DeepSeek", base: "https://api.deepseek.com", chat_path: "/chat/completions", models_path: "/models", wire: Wire::OpenAi, cloud: true, key_envs: &["DEEPSEEK_API_KEY"], fallback: &[] },
    Provider { id: "groq", label: "Groq", base: "https://api.groq.com", chat_path: "/openai/v1/chat/completions", models_path: "/openai/v1/models", wire: Wire::OpenAi, cloud: true, key_envs: &["GROQ_API_KEY"], fallback: &[] },
    Provider { id: "mistral", label: "Mistral", base: "https://api.mistral.ai", chat_path: "/v1/chat/completions", models_path: "/v1/models", wire: Wire::OpenAi, cloud: true, key_envs: &["MISTRAL_API_KEY", "MISTRAL_KEY"], fallback: &[] },
    Provider { id: "together", label: "Together", base: "https://api.together.xyz", chat_path: "/v1/chat/completions", models_path: "/v1/models", wire: Wire::OpenAi, cloud: true, key_envs: &["TOGETHER_API_KEY"], fallback: &[] },
    Provider { id: "fireworks", label: "Fireworks", base: "https://api.fireworks.ai", chat_path: "/inference/v1/chat/completions", models_path: "/inference/v1/models", wire: Wire::OpenAi, cloud: true, key_envs: &["FIREWORKS_API_KEY"], fallback: &[] },
    Provider { id: "perplexity", label: "Perplexity", base: "https://api.perplexity.ai", chat_path: "/chat/completions", models_path: "", wire: Wire::OpenAi, cloud: true, key_envs: &["PERPLEXITY_API_KEY"], fallback: &[] },
    Provider { id: "openrouter", label: "OpenRouter", base: "https://openrouter.ai", chat_path: "/api/v1/chat/completions", models_path: "/api/v1/models", wire: Wire::OpenAi, cloud: true, key_envs: &["OPENROUTER_API_KEY"], fallback: &[] },
];

pub fn provider(id: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|p| p.id == id)
}
pub fn is_cloud(id: &str) -> bool {
    provider(id).map(|p| p.cloud).unwrap_or(false)
}
pub fn default_model(id: &str) -> String {
    provider(id).and_then(|p| p.fallback.first()).map(|s| s.to_string()).unwrap_or_default()
}
pub fn env_key(id: &str) -> String {
    provider(id)
        .and_then(|p| p.key_envs.iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty())))
        .unwrap_or_default()
}

/// Suggested local models (owner's targets). Context/tool facts from the HF model cards.
pub struct LocalModel {
    pub id: &'static str,
    pub label: &'static str,
    pub hf: &'static str,
    pub gguf: &'static str,
    pub ollama: &'static str,
    pub ctx: u32,
    pub tool_note: &'static str,
}
pub const SUGGESTED_LOCAL: &[LocalModel] = &[
    LocalModel { id: "granite-4.2-3b", label: "IBM Granite 4.2 3B", hf: "ibm-granite/granite-4.2-3b", gguf: "ibm-granite/granite-4.2-3b-GGUF", ollama: "hf.co/ibm-granite/granite-4.2-3b-GGUF", ctx: 131072, tool_note: "128K native context; tool calls as <tool_call><function=..><parameter=..> (qwen3_coder parser)" },
    LocalModel { id: "qwen3.8-27b", label: "Qwen 3.8 27B", hf: "Qwen/Qwen3.8-27B", gguf: "unsloth/Qwen3.8-27B-GGUF", ollama: "hf.co/unsloth/Qwen3.8-27B-GGUF", ctx: 262144, tool_note: "262K native context; thinking on by default (<think>); tool format not stated on card" },
];

/// Small local models get the compact text tool protocol instead of the full native schema.
pub fn is_small_local(provider_id: &str, model: &str) -> bool {
    if is_cloud(provider_id) {
        return false;
    }
    let m = model.to_lowercase();
    ["3b", "1b", "2b", "4b", "1.5b", "0.5b", "granite"].iter().any(|t| m.contains(t))
}

pub const COMPACT_TOOLS: &[&str] = &["read_file", "write_file", "edit_file", "list_directory", "search_files", "run_command", "run_tests"];

pub fn prompt_tools_system(tools: &[Value]) -> String {
    let mut s = String::from("\n\n# Tools\nYou can call tools. To call one, reply with ONLY this block (one per call) and nothing else:\n<tool_call>\n{\"name\": \"TOOL_NAME\", \"arguments\": {\"arg\": \"value\"}}\n</tool_call>\nAfter you call a tool you will get a <tool_response>. When you are done, answer normally with no <tool_call>.\nAvailable tools:\n");
    for t in tools {
        let f = &t["function"];
        let props: Vec<String> = f["parameters"]["properties"].as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default();
        s.push_str(&format!("- {}({}): {}\n", f["name"].as_str().unwrap_or(""), props.join(", "), f["description"].as_str().unwrap_or("").chars().take(110).collect::<String>()));
    }
    s
}

/// Parse tool calls written as text: `<tool_call>{json}</tool_call>` (Hermes/Qwen style) and
/// `<tool_call><function=NAME><parameter=K>V</parameter></function></tool_call>` (Granite 4.2 / qwen3_coder style).
pub fn parse_text_tool_calls(content: &str) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    let re_block = regex::Regex::new(r"(?s)<tool_call>(.*?)(?:</tool_call>|$)").unwrap();
    let re_fn = regex::Regex::new(r"(?s)<function=([A-Za-z0-9_\-\.]+)>(.*?)(?:</function>|$)").unwrap();
    let re_param = regex::Regex::new(r"(?s)<parameter=([A-Za-z0-9_\-]+)>\s*(.*?)\s*</parameter>").unwrap();
    for cap in re_block.captures_iter(content) {
        let inner = cap[1].trim();
        if let Some(f) = re_fn.captures(inner) {
            let mut args = serde_json::Map::new();
            for p in re_param.captures_iter(&f[2]) {
                let raw = p[2].to_string();
                let v = serde_json::from_str::<Value>(&raw).ok().filter(|v| !v.is_string() || raw.starts_with('"')).unwrap_or(Value::String(raw));
                args.insert(p[1].to_string(), v);
            }
            out.push((f[1].to_string(), Value::Object(args)));
            continue;
        }
        let jtxt = inner.trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim();
        if let Ok(v) = serde_json::from_str::<Value>(jtxt) {
            if let Some(name) = v["name"].as_str() {
                let args = match &v["arguments"] {
                    Value::String(s) => serde_json::from_str(s).unwrap_or(json!({})),
                    Value::Null => v.get("parameters").cloned().unwrap_or(json!({})),
                    a => a.clone(),
                };
                out.push((name.to_string(), args));
            }
        }
    }
    out
}

pub fn strip_tool_markup(content: &str) -> String {
    let re = regex::Regex::new(r"(?s)<tool_call>.*?(?:</tool_call>|$)").unwrap();
    let re_think = regex::Regex::new(r"(?s)<think>.*?</think>").unwrap();
    re_think.replace_all(&re.replace_all(content, ""), "").trim().to_string()
}

/// Rewrite an OpenAI-native history for servers/models that only follow text tool calls.
pub fn to_prompt_tool_messages(messages: &[Value], tools: &[Value]) -> Vec<Value> {
    let mut out = Vec::new();
    for (i, m) in messages.iter().enumerate() {
        let role = m["role"].as_str().unwrap_or("");
        match role {
            "system" if i == 0 => out.push(json!({"role":"system","content": format!("{}{}", m["content"].as_str().unwrap_or(""), prompt_tools_system(tools))})),
            "assistant" if m.get("tool_calls").map(|t| t.is_array()).unwrap_or(false) => {
                let mut text = m["content"].as_str().unwrap_or("").to_string();
                for c in m["tool_calls"].as_array().unwrap() {
                    let args: Value = serde_json::from_str(c["function"]["arguments"].as_str().unwrap_or("{}")).unwrap_or(json!({}));
                    text.push_str(&format!("\n<tool_call>\n{}\n</tool_call>", json!({"name": c["function"]["name"], "arguments": args})));
                }
                out.push(json!({"role":"assistant","content": text.trim()}));
            }
            "tool" => out.push(json!({"role":"user","content": format!("<tool_response>\n{}\n</tool_response>", m["content"].as_str().unwrap_or(""))})),
            _ => out.push(json!({"role": role, "content": m["content"].clone()})),
        }
    }
    out
}

/// OpenAI-shaped messages + tools -> Anthropic Messages API body.
pub fn to_anthropic_body(model: &str, messages: &[Value], tools: Option<&[Value]>, max_tokens: u32) -> Value {
    let mut system = Vec::new();
    let mut out: Vec<Value> = Vec::new();
    let push = |out: &mut Vec<Value>, role: &str, blocks: Vec<Value>| {
        if let Some(last) = out.last_mut() {
            if last["role"] == role {
                last["content"].as_array_mut().unwrap().extend(blocks);
                return;
            }
        }
        out.push(json!({"role": role, "content": blocks}));
    };
    for m in messages {
        let content_text = m["content"].as_str().unwrap_or("").to_string();
        match m["role"].as_str().unwrap_or("") {
            "system" => system.push(content_text),
            "tool" => push(&mut out, "user", vec![json!({"type":"tool_result","tool_use_id": m["tool_call_id"], "content": content_text})]),
            "assistant" => {
                if let Some(raw) = m.get("_anthropic_content").and_then(|c| c.as_array()) {
                    push(&mut out, "assistant", raw.clone());
                    continue;
                }
                let mut blocks = Vec::new();
                if !content_text.is_empty() {
                    blocks.push(json!({"type":"text","text": content_text}));
                }
                if let Some(calls) = m["tool_calls"].as_array() {
                    for c in calls {
                        let input: Value = serde_json::from_str(c["function"]["arguments"].as_str().unwrap_or("{}")).unwrap_or(json!({}));
                        blocks.push(json!({"type":"tool_use","id": c["id"], "name": c["function"]["name"], "input": input}));
                    }
                }
                if !blocks.is_empty() {
                    push(&mut out, "assistant", blocks);
                }
            }
            _ => push(&mut out, "user", vec![json!({"type":"text","text": content_text})]),
        }
    }
    let mut body = json!({"model": model, "max_tokens": max_tokens, "messages": out});
    if !system.is_empty() {
        body["system"] = json!(system.join("\n\n"));
    }
    if let Some(ts) = tools {
        let at: Vec<Value> = ts.iter().map(|t| json!({"name": t["function"]["name"], "description": t["function"]["description"], "input_schema": t["function"]["parameters"]})).collect();
        body["tools"] = Value::Array(at);
        body["tool_choice"] = json!({"type":"auto"});
    }
    body
}

/// Anthropic response -> OpenAI-shaped assistant message (keeps raw blocks for faithful replay).
pub fn from_anthropic(resp: &Value) -> Value {
    let blocks = resp["content"].as_array().cloned().unwrap_or_default();
    let text: String = blocks.iter().filter(|b| b["type"] == "text").filter_map(|b| b["text"].as_str()).collect::<Vec<_>>().join("");
    let calls: Vec<Value> = blocks
        .iter()
        .filter(|b| b["type"] == "tool_use")
        .map(|b| json!({"id": b["id"], "type":"function", "function": {"name": b["name"], "arguments": b["input"].to_string()}}))
        .collect();
    let text = if text.is_empty() && resp["stop_reason"] == "refusal" {
        format!("[The model declined this request{}]", resp["stop_details"]["category"].as_str().map(|c| format!(": {c}")).unwrap_or_default())
    } else {
        text
    };
    let mut msg = json!({"role":"assistant","content": text, "_anthropic_content": blocks});
    if !calls.is_empty() {
        msg["tool_calls"] = Value::Array(calls);
    }
    msg
}

pub struct ChatRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Value,
}

/// Build the HTTP request for one chat turn. `tools` = None sends no tool schema.
pub fn build_chat_request(provider_id: &str, base_url: &str, api_key: &str, model: &str, messages: &[Value], tools: Option<&[Value]>, max_tokens: u32) -> Result<ChatRequest, String> {
    let p = provider(provider_id).ok_or_else(|| format!("Unknown provider: {provider_id}"))?;
    let base = if base_url.trim().is_empty() || (p.cloud && !base_url.starts_with("http")) { p.base.to_string() } else { base_url.trim_end_matches('/').to_string() };
    let url = format!("{base}{}", p.chat_path);
    let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
    if p.wire == Wire::Anthropic {
        headers.push(("x-api-key".into(), api_key.to_string()));
        headers.push(("anthropic-version".into(), "2023-06-01".into()));
        return Ok(ChatRequest { url, headers, body: to_anthropic_body(model, messages, tools, max_tokens) });
    }
    if !api_key.is_empty() {
        headers.push(("Authorization".into(), format!("Bearer {api_key}")));
    }
    let clean: Vec<Value> = messages
        .iter()
        .map(|m| {
            let mut m = m.clone();
            if let Some(o) = m.as_object_mut() {
                o.remove("_anthropic_content");
                o.remove("amni_widgets");
            }
            m
        })
        .collect();
    let mut body = json!({"model": model, "messages": clean});
    // OpenAI's current chat models take max_completion_tokens (max_tokens is deprecated there).
    if provider_id == "openai" {
        body["max_completion_tokens"] = json!(max_tokens);
    } else {
        body["max_tokens"] = json!(max_tokens);
    }
    if let Some(ts) = tools {
        body["tools"] = Value::Array(ts.to_vec());
        body["tool_choice"] = json!("auto");
    }
    Ok(ChatRequest { url, headers, body })
}

/// Normalise any provider's JSON reply to (assistant message in OpenAI shape, tool calls).
pub fn parse_chat_response(provider_id: &str, json: &Value, allow_text_tools: bool) -> (Value, Vec<(String, String, Value)>) {
    let mut msg = if provider(provider_id).map(|p| p.wire) == Some(Wire::Anthropic) { from_anthropic(json) } else { json["choices"][0]["message"].clone() };
    if msg.is_null() {
        msg = json!({"role":"assistant","content": ""});
    }
    let mut calls = Vec::new();
    if let Some(arr) = msg.get("tool_calls").and_then(|t| t.as_array()) {
        for (i, c) in arr.iter().enumerate() {
            let id = c["id"].as_str().filter(|s| !s.is_empty()).map(String::from).unwrap_or_else(|| format!("call_{i}"));
            let raw = &c["function"]["arguments"];
            let args = match raw {
                Value::String(s) => serde_json::from_str(s).unwrap_or(json!({})),
                Value::Object(_) => raw.clone(),
                _ => json!({}),
            };
            calls.push((id, c["function"]["name"].as_str().unwrap_or("").to_string(), args));
        }
    }
    if calls.is_empty() && allow_text_tools {
        let content = msg["content"].as_str().unwrap_or("").to_string();
        let parsed = parse_text_tool_calls(&content);
        if !parsed.is_empty() {
            let tc: Vec<Value> = parsed
                .iter()
                .enumerate()
                .map(|(i, (n, a))| json!({"id": format!("tcall_{i}"), "type":"function", "function": {"name": n, "arguments": a.to_string()}}))
                .collect();
            for (i, (n, a)) in parsed.into_iter().enumerate() {
                calls.push((format!("tcall_{i}"), n, a));
            }
            msg["content"] = json!(strip_tool_markup(&content));
            msg["tool_calls"] = Value::Array(tc);
        }
    }
    (msg, calls)
}

// ---------- live model discovery ----------

fn model_cache() -> &'static Mutex<HashMap<String, (Instant, Vec<String>)>> {
    static C: OnceLock<Mutex<HashMap<String, (Instant, Vec<String>)>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn is_chat_model(provider_id: &str, id: &str) -> bool {
    let l = id.to_lowercase();
    let junk = ["embed", "tts", "whisper", "transcribe", "dall-e", "image", "moderation", "realtime", "audio", "video", "voice", "imagine", "lyria", "robotics", "babbage", "davinci", "rerank", "live"];
    if junk.iter().any(|j| l.contains(j)) {
        return false;
    }
    match provider_id {
        "openai" => l.starts_with("gpt-") || l.starts_with('o'),
        "google" => l.contains("gemini") || l.contains("gemma"),
        _ => true,
    }
}

pub fn parse_models_list(provider_id: &str, json: &Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(arr) = json["data"].as_array() {
        for m in arr {
            if m["available"].as_bool() == Some(false) {
                continue;
            }
            if let Some(id) = m["id"].as_str() {
                out.push(id.trim_start_matches("models/").to_string());
            }
        }
    }
    if let Some(arr) = json["models"].as_array() {
        for m in arr {
            if let Some(n) = m["name"].as_str().or(m["model"].as_str()) {
                out.push(n.trim_start_matches("models/").to_string());
            }
        }
    }
    out.retain(|m| is_chat_model(provider_id, m));
    out.dedup();
    out
}

pub struct ModelList {
    pub models: Vec<String>,
    pub source: &'static str,
    pub error: Option<String>,
}

/// Live list (cached 10 min) -> confirmed fallback. Cloud providers are never contacted offline or without a key.
pub async fn list_models(provider_id: &str, base_url: &str, api_key: &str, offline: bool) -> ModelList {
    let Some(p) = provider(provider_id) else { return ModelList { models: vec![], source: "none", error: Some(format!("unknown provider {provider_id}")) } };
    let fb = || p.fallback.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    if p.cloud && offline {
        return ModelList { models: fb(), source: "fallback", error: Some("offline mode: cloud provider not contacted".into()) };
    }
    if p.cloud && api_key.is_empty() {
        return ModelList { models: fb(), source: "fallback", error: Some("no API key set".into()) };
    }
    if p.models_path.is_empty() {
        return ModelList { models: fb(), source: "fallback", error: None };
    }
    let base = if base_url.trim().is_empty() { p.base.to_string() } else { base_url.trim_end_matches('/').to_string() };
    let key = format!("{provider_id}|{base}");
    if let Some((t, v)) = model_cache().lock().await.get(&key) {
        if t.elapsed() < Duration::from_secs(600) && !v.is_empty() {
            return ModelList { models: v.clone(), source: "cache", error: None };
        }
    }
    let client = reqwest::Client::builder().timeout(Duration::from_secs(if p.cloud { 6 } else { 2 })).connect_timeout(Duration::from_millis(if p.cloud { 3000 } else { 700 })).build().unwrap_or_default();
    let mut paths = vec![p.models_path];
    if !p.cloud {
        for alt in ["/v1/models", "/api/tags"] {
            if !paths.contains(&alt) {
                paths.push(alt);
            }
        }
    }
    let mut last_err = None;
    for path in paths {
        let mut req = client.get(format!("{base}{path}"));
        if p.wire == Wire::Anthropic {
            req = req.header("x-api-key", api_key).header("anthropic-version", "2023-06-01");
        } else if !api_key.is_empty() {
            req = req.bearer_auth(api_key);
        }
        match req.send().await {
            Ok(r) if r.status().is_success() => {
                if let Ok(j) = r.json::<Value>().await {
                    let mut v = parse_models_list(provider_id, &j);
                    if p.cloud {
                        v.sort();
                        v.reverse();
                    }
                    if !v.is_empty() {
                        model_cache().lock().await.insert(key, (Instant::now(), v.clone()));
                        return ModelList { models: v, source: "live", error: None };
                    }
                }
            }
            Ok(r) => last_err = Some(format!("{path}: HTTP {}", r.status())),
            Err(e) => last_err = Some(format!("{path}: {}", if e.is_connect() { "not reachable".to_string() } else { e.to_string() })),
        }
    }
    ModelList { models: fb(), source: "fallback", error: last_err }
}

/// Probe well-known local model servers concurrently (fast: ~700 ms worst case).
pub async fn detect_local_servers() -> Vec<Value> {
    let targets = [
        ("amni", "Adam / Amni-AI", "http://127.0.0.1:7700"),
        ("ollama", "Ollama", "http://127.0.0.1:11434"),
        ("local", "llama.cpp server", "http://127.0.0.1:8080"),
        ("local", "LM Studio", "http://127.0.0.1:1234"),
    ];
    let futs = targets.iter().map(|(pid, label, base)| async move {
        let client = reqwest::Client::builder().timeout(Duration::from_millis(900)).connect_timeout(Duration::from_millis(500)).no_proxy().build().unwrap_or_default();
        let mut models = Vec::new();
        let mut up = false;
        for path in ["/api/tags", "/v1/models"] {
            if let Ok(r) = client.get(format!("{base}{path}")).send().await {
                if r.status().is_success() {
                    up = true;
                    if let Ok(j) = r.json::<Value>().await {
                        models = parse_models_list(pid, &j);
                    }
                    if !models.is_empty() {
                        break;
                    }
                }
            }
        }
        json!({"provider": pid, "label": label, "base_url": base, "up": up, "models": models})
    });
    futures::future::join_all(futs).await
}

pub fn providers_json(offline: bool) -> Value {
    let list: Vec<Value> = PROVIDERS
        .iter()
        .map(|p| json!({"id": p.id, "label": p.label, "cloud": p.cloud, "offline_capable": !p.cloud, "available": !(offline && p.cloud), "has_env_key": !env_key(p.id).is_empty(), "default_base": p.base, "default_model": default_model(p.id)}))
        .collect();
    let local: Vec<Value> = SUGGESTED_LOCAL
        .iter()
        .map(|m| json!({"id": m.id, "label": m.label, "hf": m.hf, "gguf": m.gguf, "ollama": m.ollama, "context": m.ctx, "tools": m.tool_note}))
        .collect();
    json!({"providers": list, "suggested_local": local, "offline": offline})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tools() -> Vec<Value> {
        vec![json!({"type":"function","function":{"name":"read_file","description":"Read a file","parameters":{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}}})]
    }
    #[test]
    fn no_stale_ids_in_fallbacks() {
        let stale = ["claude-sonnet-4-20250514", "claude-opus-4-20250514", "gpt-4o", "gemini-1.5-pro", "grok-beta", "gemini-2.0-flash", "grok-4-1-fast-reasoning"];
        for p in PROVIDERS {
            for m in p.fallback {
                assert!(!stale.contains(m), "{} still lists stale {}", p.id, m);
            }
        }
        assert_eq!(default_model("anthropic"), "claude-opus-5-5");
        assert_eq!(default_model("openai"), "gpt-6-astra");
        assert_eq!(default_model("google"), "gemini-3.8-flash");
        assert_eq!(default_model("xai"), "grok-4.7");
    }
    #[test]
    fn openai_uses_max_completion_tokens() {
        let r = build_chat_request("openai", "https://api.openai.com", "k", "gpt-6-astra", &[json!({"role":"user","content":"hi"})], Some(&tools()), 1000).unwrap();
        assert_eq!(r.url, "https://api.openai.com/v1/chat/completions");
        assert_eq!(r.body["max_completion_tokens"], 1000);
        assert!(r.body.get("max_tokens").is_none());
        assert!(r.body.get("temperature").is_none());
        assert!(r.headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer k"));
    }
    #[test]
    fn google_uses_openai_compat_endpoint() {
        let r = build_chat_request("google", "", "g", "gemini-3.8-flash", &[json!({"role":"user","content":"hi"})], Some(&tools()), 500).unwrap();
        assert_eq!(r.url, "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions");
        assert_eq!(r.body["tools"][0]["function"]["name"], "read_file");
        assert!(!r.url.contains("key="), "key goes in the header, not the URL");
    }
    #[test]
    fn anthropic_body_shape() {
        let msgs = vec![
            json!({"role":"system","content":"sys"}),
            json!({"role":"user","content":"read a.txt"}),
            json!({"role":"assistant","content":"","tool_calls":[{"id":"tu_1","type":"function","function":{"name":"read_file","arguments":"{\"path\":\"a.txt\"}"}}]}),
            json!({"role":"tool","tool_call_id":"tu_1","content":"hello"}),
        ];
        let r = build_chat_request("anthropic", "https://api.anthropic.com", "k", "claude-opus-5-5", &msgs, Some(&tools()), 16000).unwrap();
        assert_eq!(r.url, "https://api.anthropic.com/v1/messages");
        let b = &r.body;
        assert_eq!(b["system"], "sys");
        assert_eq!(b["max_tokens"], 16000);
        assert!(b.get("temperature").is_none() && b.get("thinking").is_none());
        assert_eq!(b["tools"][0]["input_schema"]["properties"]["path"]["type"], "string");
        assert_eq!(b["tool_choice"]["type"], "auto");
        assert_eq!(b["messages"][1]["content"][0]["type"], "tool_use");
        assert_eq!(b["messages"][1]["content"][0]["input"]["path"], "a.txt");
        assert_eq!(b["messages"][2]["role"], "user");
        assert_eq!(b["messages"][2]["content"][0]["type"], "tool_result");
        assert!(r.headers.iter().any(|(k, _)| k == "x-api-key"));
        assert!(r.headers.iter().any(|(k, v)| k == "anthropic-version" && v == "2023-06-01"));
    }
    #[test]
    fn anthropic_response_roundtrip_keeps_blocks() {
        let resp = json!({"stop_reason":"tool_use","content":[{"type":"thinking","thinking":"","signature":"s"},{"type":"text","text":"Reading."},{"type":"tool_use","id":"tu_9","name":"read_file","input":{"path":"x"}}]});
        let (msg, calls) = parse_chat_response("anthropic", &resp, false);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "tu_9");
        assert_eq!(calls[0].2["path"], "x");
        let body = to_anthropic_body("claude-opus-5-5", &[json!({"role":"user","content":"q"}), msg], None, 100);
        assert_eq!(body["messages"][1]["content"][0]["type"], "thinking", "thinking block replayed unchanged");
        let refusal = from_anthropic(&json!({"stop_reason":"refusal","stop_details":{"category":"cyber"},"content":[]}));
        assert!(refusal["content"].as_str().unwrap().contains("cyber"));
    }
    #[test]
    fn text_tool_calls_json_and_granite_xml() {
        let a = parse_text_tool_calls("ok <tool_call>\n{\"name\":\"read_file\",\"arguments\":{\"path\":\"a.txt\"}}\n</tool_call>");
        assert_eq!(a, vec![("read_file".to_string(), json!({"path":"a.txt"}))]);
        let g = parse_text_tool_calls("<tool_call>\n<function=list_directory>\n<parameter=path>\nsrc\n</parameter>\n</function>\n</tool_call>");
        assert_eq!(g, vec![("list_directory".to_string(), json!({"path":"src"}))]);
        let (msg, calls) = parse_chat_response("ollama", &json!({"choices":[{"message":{"role":"assistant","content":"<think>x</think>Let me look.<tool_call>{\"name\":\"read_file\",\"arguments\":\"{\\\"path\\\":\\\"b\\\"}\"}</tool_call>"}}]}), true);
        assert_eq!(calls[0].1, "read_file");
        assert_eq!(calls[0].2["path"], "b");
        assert_eq!(msg["content"], "Let me look.");
        assert!(parse_text_tool_calls("no tools here").is_empty());
    }
    #[test]
    fn prompt_mode_rewrites_history() {
        let msgs = vec![
            json!({"role":"system","content":"S"}),
            json!({"role":"assistant","content":"","tool_calls":[{"id":"1","type":"function","function":{"name":"read_file","arguments":"{\"path\":\"a\"}"}}]}),
            json!({"role":"tool","tool_call_id":"1","content":"DATA"}),
        ];
        let out = to_prompt_tool_messages(&msgs, &tools());
        assert!(out[0]["content"].as_str().unwrap().contains("read_file(path)"));
        assert!(out[1]["content"].as_str().unwrap().contains("<tool_call>"));
        assert_eq!(out[2]["role"], "user");
        assert!(out[2]["content"].as_str().unwrap().contains("DATA"));
    }
    #[test]
    fn model_list_parsing_filters_non_chat() {
        let j = json!({"data":[{"id":"gpt-6-astra"},{"id":"gpt-image-2.6-flare"},{"id":"gpt-realtime-2"},{"id":"whisper-1"}]});
        assert_eq!(parse_models_list("openai", &j), vec!["gpt-6-astra"]);
        let o = json!({"models":[{"name":"hf.co/ibm-granite/granite-4.2-3b-GGUF:Q4_K_M"},{"name":"nomic-embed-text"}]});
        assert_eq!(parse_models_list("ollama", &o), vec!["hf.co/ibm-granite/granite-4.2-3b-GGUF:Q4_K_M"]);
        let g = json!({"data":[{"id":"models/gemini-3.8-flash"},{"id":"models/gemini-embedding-2"}]});
        assert_eq!(parse_models_list("google", &g), vec!["gemini-3.8-flash"]);
    }
    #[test]
    fn small_local_detection() {
        assert!(is_small_local("ollama", "hf.co/ibm-granite/granite-4.2-3b-GGUF"));
        assert!(!is_small_local("ollama", "hf.co/unsloth/Qwen3.8-27B-GGUF"));
        assert!(!is_small_local("anthropic", "claude-haiku-5-5"));
    }
    #[tokio::test]
    async fn offline_never_contacts_cloud() {
        let t = Instant::now();
        let l = list_models("openai", "http://10.255.255.1", "k", true).await;
        assert_eq!(l.source, "fallback");
        assert!(t.elapsed() < Duration::from_millis(50));
    }
}
