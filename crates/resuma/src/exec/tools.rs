//! Tool registry — HTTP fetch, AI proxy, and extensible actions.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::OnceLock;
use std::time::Duration;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::{json, Value};
use tracing::warn;

use crate::core::{Result, ResumaError};

pub type ToolFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type ToolFn = fn(Value) -> ToolFuture;

static TOOLS: Lazy<RwLock<HashMap<String, ToolFn>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    register_builtin(&mut m);
    RwLock::new(m)
});

static HTTP: OnceLock<reqwest::Client> = OnceLock::new();

pub fn init_http_client() {
    let _ = http_client();
}

pub fn http_client() -> &'static reqwest::Client {
    HTTP.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("resuma-exec/1.0")
            .build()
            .expect("reqwest client")
    })
}

fn register_builtin(m: &mut HashMap<String, ToolFn>) {
    m.insert("echo".into(), tool_echo);
    m.insert("ai".into(), tool_ai);
    m.insert("scrape".into(), tool_scrape);
    m.insert("fetch".into(), tool_fetch);
}

fn tool_echo(args: Value) -> ToolFuture {
    Box::pin(async move { Ok(args) })
}

fn tool_fetch(args: Value) -> ToolFuture {
    Box::pin(async move {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ResumaError::validation("fetch requires url"))?;

        super::ssrf::validate_fetch_url(url)?;

        let method = args
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        let client = http_client();
        let mut req = match method.to_uppercase().as_str() {
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            _ => client.get(url),
        };

        if let Some(headers) = args.get("headers").and_then(|h| h.as_object()) {
            for (k, v) in headers {
                if super::ssrf::BLOCKED_HEADERS
                    .iter()
                    .any(|blocked| k.eq_ignore_ascii_case(blocked))
                {
                    continue;
                }
                if let Some(s) = v.as_str() {
                    req = req.header(k.as_str(), s);
                }
            }
        }

        if let Some(body) = args.get("body") {
            if body.is_string() {
                req = req.body(body.as_str().unwrap().to_string());
            } else {
                req = req.json(body);
            }
        }

        let res = req
            .send()
            .await
            .map_err(|e| ResumaError::Other(e.to_string()))?;
        let status = res.status().as_u16();
        let content_type = res
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let max_bytes = super::ssrf::max_fetch_bytes();
        let body = res
            .bytes()
            .await
            .map_err(|e| ResumaError::Other(e.to_string()))?;
        if body.len() > max_bytes {
            return Err(ResumaError::PayloadTooLarge);
        }
        let body = String::from_utf8_lossy(&body).into_owned();

        Ok(json!({
            "url": url,
            "status": status,
            "content_type": content_type,
            "body": body,
        }))
    })
}

fn tool_scrape(args: Value) -> ToolFuture {
    Box::pin(async move {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // Use public search API stub chain: fetch HTML from a constructed search URL
        // Apps can register a custom `scrape` tool for production crawlers.
        let search_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding(query));
        let fetched = tool_fetch(json!({ "url": search_url })).await?;
        let body = fetched
            .get("body")
            .and_then(|b| b.as_str())
            .unwrap_or("");

        let items: Vec<Value> = body
            .split("result__a")
            .skip(1)
            .take(10)
            .enumerate()
            .map(|(i, chunk)| {
                let title = chunk
                    .split('>')
                    .nth(1)
                    .and_then(|s| s.split('<').next())
                    .unwrap_or("result")
                    .trim();
                json!({ "name": title, "index": i, "query": query })
            })
            .collect();

        Ok(json!({
            "query": query,
            "count": items.len(),
            "items": items,
        }))
    })
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

fn tool_ai(args: Value) -> ToolFuture {
    Box::pin(async move {
        let api_key = match std::env::var("RESUMA_AI_API_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => {
                warn!("RESUMA_AI_API_KEY not set — using local AI stub");
                return tool_ai_stub(args).await;
            }
        };

        let base = std::env::var("RESUMA_AI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".into());
        let model = std::env::var("RESUMA_AI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());

        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("Analyze the following data.");
        let data = args.get("data").cloned().unwrap_or(Value::Null);

        let user_content = if data.is_null() {
            prompt.to_string()
        } else {
            format!("{prompt}\n\nData:\n{}", serde_json::to_string_pretty(&data).unwrap_or_default())
        };

        let body = json!({
            "model": model,
            "messages": [
                { "role": "system", "content": "You are a helpful assistant running inside Resuma OS." },
                { "role": "user", "content": user_content }
            ],
            "temperature": 0.2
        });

        let url = format!("{}/chat/completions", base.trim_end_matches('/'));
        let res = http_client()
            .post(&url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ResumaError::Other(format!("AI request failed: {e}")))?;

        let status = res.status();
        let payload: Value = res
            .json()
            .await
            .map_err(|e| ResumaError::Other(format!("AI response parse failed: {e}")))?;

        if !status.is_success() {
            return Err(ResumaError::Other(format!(
                "AI provider error {}: {}",
                status,
                payload
            )));
        }

        let text = payload
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(json!({
            "summary": text,
            "model": model,
            "provider": "resuma-ai",
        }))
    })
}

async fn tool_ai_stub(args: Value) -> Result<Value> {
    let prompt = args
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("analyze");
    Ok(json!({
        "summary": format!("[resuma stub] processed: {prompt}"),
        "tokens": 0,
        "provider": "stub",
    }))
}

pub fn register_tool(name: &str, f: ToolFn) {
    TOOLS.write().insert(name.to_string(), f);
}

pub async fn dispatch(name: &str, args: Value) -> Result<Value> {
    let f = TOOLS
        .read()
        .get(name)
        .copied()
        .ok_or_else(|| ResumaError::UnknownTool(name.to_string()))?;
    f(args).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn echo_tool_roundtrip() {
        init_http_client();
        let out = dispatch("echo", json!({ "x": 1 })).await.unwrap();
        assert_eq!(out["x"], 1);
    }
}
