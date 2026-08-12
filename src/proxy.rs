use std::time::Instant;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode, header, HeaderValue},
    response::Response,
};
use futures::stream::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::metrics::store::now_ms;
use crate::models::Event;
use crate::server::AppState;

const SAFE_UPSTREAM_HEADERS: &[&str] = &[
    "retry-after", "ratelimit-reset", "rate-limit-reset",
    "x-ratelimit-reset", "x-rate-limit-reset",
    "ratelimit-remaining", "rate-limit-remaining",
    "x-ratelimit-remaining", "x-rate-limit-remaining",
    "ratelimit-limit", "rate-limit-limit",
    "x-ratelimit-limit", "x-rate-limit-limit",
];

pub async fn chat_completions_handler(
    State(state): State<AppState>,
    _headers: HeaderMap,
    body: String,
) -> Response<Body> {
    let started_at = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("?").to_string();

    let json_body: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return err_response(StatusCode::BAD_REQUEST, "invalid JSON"),
    };

    if body.len() > state.config.max_body_bytes as usize {
        return err_response(StatusCode::PAYLOAD_TOO_LARGE, "body too large");
    }

    let requested_model = json_body
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("auto");
    let selected_model = state.router.select(requested_model);
    let is_streaming = json_body.get("stream").and_then(|s| s.as_bool()).unwrap_or(false);

    let mut last_err: Option<String> = None;
    let mut attempts = 0;
    let max_attempts = state.config.upstreams.len().max(1);

    while attempts < max_attempts {
        let provider = match state.provider_pool.select().await {
            Some(p) => p,
            None => {
                return err_response(StatusCode::SERVICE_UNAVAILABLE, "no providers available");
            }
        };

        let (base_url, api_key, provider_name) = provider;
        let upstream_url = format!("{}/chat/completions", base_url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(state.config.timeout_secs))
            .build()
            .unwrap();

        match client
            .post(&upstream_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&json_body)
            .send()
            .await
        {
            Ok(upstream_response) => {
                let upstream_status = upstream_response.status().as_u16();
                let is_ok = upstream_status < 400;

                if !is_ok {
                    state.provider_pool.record_result(&provider_name, false, upstream_status, 0).await;
                    last_err = Some(format!("{}: {}", provider_name, upstream_status));
                    attempts += 1;
                    continue;
                }

                state.provider_pool.record_result(&provider_name, true, upstream_status, 0).await;

                let mut safe_headers = Vec::new();
                for name in SAFE_UPSTREAM_HEADERS {
                    if let Some(val) = upstream_response.headers().get(*name) {
                        if let Ok(v) = val.to_str() {
                            safe_headers.push((name.to_string(), v.to_string()));
                        }
                    }
                }

                let mut resp_headers = HeaderMap::new();
                resp_headers.insert("X-Model-Used", selected_model.parse().unwrap());
                resp_headers.insert("X-Request-Id", request_id.parse().unwrap());
                resp_headers.insert("X-Provider", provider_name.parse().unwrap());
                for (name, val) in &safe_headers {
                    if let Ok(name) = name.parse::<axum::http::HeaderName>() {
                        if let Ok(val) = val.parse::<axum::http::HeaderValue>() {
                            resp_headers.insert(name, val);
                        }
                    }
                }

                if is_streaming {
                    resp_headers.insert(header::CONTENT_TYPE, "text/event-stream; charset=utf-8".parse().unwrap());
                    return handle_streaming(state, upstream_response, selected_model, started_at, upstream_status, resp_headers).await;
                } else {
                    resp_headers.insert(header::CONTENT_TYPE, "application/json; charset=utf-8".parse().unwrap());
                    return handle_non_streaming(state, upstream_response, selected_model, started_at, upstream_status, resp_headers).await;
                }
            }
            Err(e) => {
                state.provider_pool.record_result(&provider_name, false, 502, 0).await;
                last_err = Some(format!("{}: {}", provider_name, e));
                attempts += 1;
            }
        }
    }

    err_response(StatusCode::BAD_GATEWAY, &last_err.unwrap_or_else(|| "all providers failed".to_string()))
}

pub async fn playground_test_handler(
    State(state): State<AppState>,
    body: String,
) -> Response<Body> {
    let started_at = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("?").to_string();

    let json_body: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return err_response(StatusCode::BAD_REQUEST, "invalid JSON"),
    };

    let model = json_body.get("model").and_then(|m| m.as_str()).unwrap_or("auto");
    let base_url = json_body.get("baseUrl").and_then(|u| u.as_str()).unwrap_or(&state.config.upstream).trim_end_matches('/').to_string();
    let api_key = json_body.get("apiKey").and_then(|k| k.as_str()).unwrap_or("").to_string();
    let prompt = json_body.get("prompt").and_then(|p| p.as_str()).unwrap_or("");
    let is_streaming = json_body.get("stream").and_then(|s| s.as_bool()).unwrap_or(false);

    if prompt.is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "prompt is required");
    }

    let upstream_url = format!("{}/chat/completions", base_url);

    let messages = serde_json::json!([
        { "role": "user", "content": prompt }
    ]);

    let request_body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": is_streaming,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let upstream_response = match client
        .post(&upstream_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            let latency_ms = started_at.elapsed().as_millis() as u64;
            let err_msg = e.to_string();
            let error_type = if err_msg.contains("timeout") { "timeout" } else { "network" };
            record_event(&state, model, false, 502, latency_ms, 0, 0, 0, Some(error_type.to_string()), false, None, false, false);
            return err_response(StatusCode::BAD_GATEWAY, &err_msg);
        }
    };

    let upstream_status = upstream_response.status().as_u16();

    let mut safe_headers = Vec::new();
    for name in SAFE_UPSTREAM_HEADERS {
        if let Some(val) = upstream_response.headers().get(*name) {
            if let Ok(v) = val.to_str() {
                safe_headers.push((name.to_string(), v.to_string()));
            }
        }
    }

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert("X-Model-Used", model.parse().unwrap());
    resp_headers.insert("X-Request-Id", request_id.parse().unwrap());
    for (name, val) in &safe_headers {
        if let Ok(name) = name.parse::<axum::http::HeaderName>() {
            if let Ok(val) = val.parse::<axum::http::HeaderValue>() {
                resp_headers.insert(name, val);
            }
        }
    }

    if is_streaming {
        resp_headers.insert(header::CONTENT_TYPE, "text/event-stream; charset=utf-8".parse().unwrap());
        handle_streaming(state, upstream_response, model.to_string(), started_at, upstream_status, resp_headers).await
    } else {
        resp_headers.insert(header::CONTENT_TYPE, "application/json; charset=utf-8".parse().unwrap());
        handle_non_streaming(state, upstream_response, model.to_string(), started_at, upstream_status, resp_headers).await
    }
}

fn err_response(status: StatusCode, msg: &str) -> Response<Body> {
    let body = serde_json::json!({ "error": msg }).to_string();
    let mut resp = Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap();
    resp.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    resp.headers_mut().insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    resp
}

async fn handle_non_streaming(
    state: AppState,
    upstream_response: reqwest::Response,
    selected_model: String,
    started_at: Instant,
    upstream_status: u16,
    resp_headers: HeaderMap,
) -> Response<Body> {
    let is_ok = upstream_status < 400;
    let is_rate_limited = upstream_status == 429;

    let upstream_body = match upstream_response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            let latency_ms = started_at.elapsed().as_millis() as u64;
            record_event(&state, &selected_model, false, 502, latency_ms, 0, 0, 0, Some("upstream_read_error".to_string()), false, None, false, false);
            return err_response(StatusCode::BAD_GATEWAY, &e.to_string());
        }
    };

    let latency_ms = started_at.elapsed().as_millis() as u64;

    let error_type: Option<String> = if !is_ok && !is_rate_limited {
        Some(format!("upstream_{}", upstream_status))
    } else if is_rate_limited {
        Some("rate_limited".to_string())
    } else {
        None
    };

    let finish_reason: String;
    let (prompt, completion, total, usage_reported) = if is_ok {
        let (p, c, t, r, f) = extract_usage_from_body(&upstream_body);
        finish_reason = f;
        (p, c, t, r)
    } else {
        finish_reason = "stop".to_string();
        (0, 0, 0, false)
    };

    let (prompt, completion, total, usage_reported, usage_estimated) =
        if is_ok && !usage_reported {
            let body_str = String::from_utf8_lossy(&upstream_body);
            let estimated = estimate_tokens(&body_str);
            (estimated.0, estimated.1, estimated.0 + estimated.1, false, true)
        } else { (prompt, completion, total, usage_reported, false) };

    record_event(&state, &selected_model, is_ok, upstream_status, latency_ms,
        prompt, completion, total, error_type, is_rate_limited,
        Some(&finish_reason), usage_reported, usage_estimated);

    let status = StatusCode::from_u16(upstream_status).unwrap_or(StatusCode::OK);
    let mut resp = Response::new(Body::from(upstream_body.to_vec()));
    resp.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    resp.headers_mut().insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    *resp.status_mut() = status;
    *resp.headers_mut() = resp_headers;
    resp
}

async fn handle_streaming(
    state: AppState,
    upstream_response: reqwest::Response,
    selected_model: String,
    started_at: Instant,
    upstream_status: u16,
    resp_headers: HeaderMap,
) -> Response<Body> {
    let (tx, rx) = mpsc::channel::<Result<axum::body::Bytes, std::io::Error>>(32);
    let state_clone = state.clone();
    let model_clone = selected_model.clone();

    tokio::spawn(async move {
        let mut stream = upstream_response.bytes_stream();
        let mut last_usage_json = String::new();
        let mut total_received: u64 = 0;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    total_received += chunk.len() as u64;
                    let chunk_str = String::from_utf8_lossy(&chunk);
                    for line in chunk_str.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data != "[DONE]" && data.contains("\"usage\"") {
                                last_usage_json = data.to_string();
                            }
                        }
                    }
                    if tx.send(Ok(chunk)).await.is_err() { break; }
                }
                Err(e) => {
                    let _ = tx.send(Err(std::io::Error::new(std::io::ErrorKind::Other, e))).await;
                    break;
                }
            }
        }

        let latency_ms = started_at.elapsed().as_millis() as u64;
        let (prompt, completion, total, usage_reported) = if !last_usage_json.is_empty() {
            extract_usage_from_sse(&last_usage_json)
        } else { (0, 0, 0, false) };

        let (prompt, completion, total, usage_reported, usage_estimated) =
            if !usage_reported && total_received > 0 {
                (0, 0, 0, false, true)
            } else { (prompt, completion, total, usage_reported, false) };

        record_event(&state_clone, &model_clone, true, upstream_status, latency_ms,
            prompt, completion, total, None, false, Some("stop"), usage_reported, usage_estimated);
    });

    let stream = ReceiverStream::new(rx);
    let body = Body::from_stream(stream);
    let status = StatusCode::from_u16(upstream_status).unwrap_or(StatusCode::OK);
    let mut resp = Response::new(body);
    resp.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    resp.headers_mut().insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    *resp.status_mut() = status;
    *resp.headers_mut() = resp_headers;
    resp
}

fn extract_usage_from_body(body: &[u8]) -> (u64, u64, u64, bool, String) {
    let parsed: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return (0, 0, 0, false, "stop".to_string()),
    };
    let finish_reason = parsed
        .get("choices").and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("finish_reason"))
        .and_then(|f| f.as_str())
        .unwrap_or("stop").to_string();
    match parsed.get("usage") {
        Some(usage) => {
            let p = usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let c = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let t = usage.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(p + c);
            (p, c, t, true, finish_reason)
        }
        None => (0, 0, 0, false, finish_reason),
    }
}

fn extract_usage_from_sse(data: &str) -> (u64, u64, u64, bool) {
    let parsed: Value = match serde_json::from_str(data) {
        Ok(v) => v,
        Err(_) => return (0, 0, 0, false),
    };
    match parsed.get("usage") {
        Some(usage) => {
            let p = usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let c = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let t = usage.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(p + c);
            (p, c, t, true)
        }
        None => (0, 0, 0, false),
    }
}

fn estimate_tokens(text: &str) -> (u64, u64) {
    let len = text.len() as u64;
    (len / 4, len / 8)
}

fn record_event(
    state: &AppState,
    model: &str,
    ok: bool,
    status: u16,
    latency_ms: u64,
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
    error_type: Option<String>,
    rate_limited: bool,
    finish_reason: Option<&str>,
    usage_reported: bool,
    usage_estimated: bool,
) {
    let event = Event {
        ts: now_ms() as i64,
        model: model.to_string(),
        ok,
        status,
        latency_ms,
        prompt_tokens,
        completion_tokens,
        total_tokens,
        error_type,
        rate_limited,
        finish_reason: finish_reason.map(|s| s.to_string()),
        cost: 0.0,
        usage_reported,
        usage_estimated,
    };

    state.metrics.lock().unwrap().record(event.clone());

    // Write to usage store (sync file I/O in background)
    let usage = state.usage_store.clone();
    std::thread::spawn(move || {
        usage.lock().unwrap().record(&event);
    });
}
