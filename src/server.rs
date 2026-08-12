use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::{
    auth,
    circuit_breaker::{ProviderPool, RoutingStrategy},
    config::Config,
    export,
    metrics::{MetricsStore, ModelStatusBuilder, SnapshotBuilder},
    models::*,
    proxy, router, templates,
    usage_store::UsageStore,
    APP_NAME, VERSION,
};

fn csp_dashboard() -> &'static str {
    "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'"
}

fn csp_flow() -> &'static str {
    "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'"
}

fn apply_security_headers<B>(resp: &mut Response<B>, is_dashboard: bool) {
    let csp = if is_dashboard {
        csp_dashboard()
    } else {
        csp_flow()
    };
    resp.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(csp).unwrap(),
    );
    resp.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    resp.headers_mut()
        .insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub metrics: Arc<Mutex<MetricsStore>>,
    pub usage_store: Arc<Mutex<UsageStore>>,
    pub router: router::Router,
    pub provider_pool: ProviderPool,
    pub started_at: std::time::Instant,
}

pub fn build_router(config: Config) -> Router {
    let max_events = if config.metrics_max_events > 0 {
        config.metrics_max_events
    } else {
        2000
    };

    let routing_strategy = match config.routing.as_str() {
        "random" => router::Strategy::Random,
        _ => router::Strategy::RoundRobin,
    };

    let provider_strategy = match config.routing.as_str() {
        "random" => RoutingStrategy::Random,
        "fallback" => RoutingStrategy::Fallback,
        _ => RoutingStrategy::RoundRobin,
    };

    let provider_pool = ProviderPool::new(
        config.upstreams.clone(),
        provider_strategy,
        config.circuit_max_failures,
        config.circuit_reset_secs,
    );

    let pool_for_probe = provider_pool.clone();
    let probe_interval = config.probe_interval_ms;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(probe_interval));
        loop {
            interval.tick().await;
            pool_for_probe.probe().await;
        }
    });

    let state = AppState {
        router: router::Router::new(config.models.clone(), routing_strategy),
        config: config.clone(),
        metrics: Arc::new(Mutex::new(MetricsStore::new(max_events))),
        usage_store: Arc::new(Mutex::new(UsageStore::new(
            &config.usage_db_path,
            config.usage_retention_days,
        ))),
        provider_pool,
        started_at: std::time::Instant::now(),
    };

    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/models", get(models_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/flow", get(flow_handler))
        .route("/playground", get(playground_handler))
        .route("/playground/test", post(proxy::playground_test_handler))
        .route("/metrics", get(metrics_handler))
        .route("/diag", get(diag_handler))
        .route("/usage", get(usage_handler))
        .route("/limits", get(limits_handler))
        .route("/providers", get(providers_handler))
        .route("/export/{format}", get(export_handler))
        .route(
            "/v1/chat/completions",
            post(proxy::chat_completions_handler),
        )
        .with_state(state)
}

fn require_auth(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<(), (StatusCode, &'static str)> {
    if state.config.management_token.is_empty() {
        return Ok(());
    }
    let auth_val = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    if auth::is_authorized(&state.config.management_token, auth_val) {
        Ok(())
    } else {
        Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn models_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let models: Vec<serde_json::Value> = state
        .config
        .models
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m,
                "object": "model",
                "created": 0,
                "owned_by": "opencode"
            })
        })
        .collect();
    Json(serde_json::json!({ "object": "list", "data": models }))
}

async fn dashboard_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Response<Body>, (StatusCode, &'static str)> {
    require_auth(&state, &headers)?;
    let html = templates::dashboard::render(VERSION);
    let mut resp = Response::new(Body::from(html));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    apply_security_headers(&mut resp, true);
    Ok(resp)
}

async fn flow_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Response<Body>, (StatusCode, &'static str)> {
    require_auth(&state, &headers)?;
    let html = templates::flow::render(VERSION);
    let mut resp = Response::new(Body::from(html));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    apply_security_headers(&mut resp, false);
    Ok(resp)
}

async fn playground_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Response<Body>, (StatusCode, &'static str)> {
    require_auth(&state, &headers)?;
    let model = state
        .config
        .models
        .first()
        .cloned()
        .unwrap_or_else(|| "gpt-5".to_string());
    let base_url = state.config.upstream.trim_end_matches('/').to_string();
    let html = templates::playground::render(VERSION, &model, &base_url);
    let mut resp = Response::new(Body::from(html));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    apply_security_headers(&mut resp, false);
    Ok(resp)
}

async fn metrics_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Snapshot>, (StatusCode, &'static str)> {
    require_auth(&state, &headers)?;

    let metrics = state.metrics.lock().unwrap();
    let all_events: Vec<Event> = metrics.events().iter().cloned().collect();
    let model_status = ModelStatusBuilder::build(
        &all_events,
        &state.config.models,
        &state.config.primary_models,
    );
    let usage = state.usage_store.lock().unwrap().summary();

    let snapshot = SnapshotBuilder::build(
        &metrics,
        300_000,
        &model_status,
        usage,
        &state.config.primary_models,
        &state.config.routing,
    );

    Ok(Json(snapshot))
}

async fn diag_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<DiagResponse>, (StatusCode, &'static str)> {
    require_auth(&state, &headers)?;

    let provider_snapshots = state.provider_pool.snapshots().await;

    let metrics = state.metrics.lock().unwrap();
    let all_events: Vec<Event> = metrics.events().iter().cloned().collect();
    let model_status = ModelStatusBuilder::build(
        &all_events,
        &state.config.models,
        &state.config.primary_models,
    );
    let usage = state.usage_store.lock().unwrap().summary();
    let snapshot = SnapshotBuilder::build(
        &metrics,
        300_000,
        &model_status,
        usage,
        &state.config.primary_models,
        &state.config.routing,
    );
    drop(metrics);

    let s = &snapshot.summary.window;
    let primary = &snapshot.model_status.primary;
    let uptime = snapshot.uptime_seconds;
    let h = uptime / 3600;
    let m = (uptime % 3600) / 60;

    let diag = DiagResponse {
        version: VERSION.to_string(),
        app: APP_NAME.to_string(),
        uptime_seconds: uptime,
        uptime_human: format!("{}h {}m", h, m),
        generated_at: snapshot.generated_at,
        routing: state.config.routing.clone(),
        providers: provider_snapshots
            .iter()
            .map(|s| DiagProvider {
                name: s.name.clone(),
                url: s.url.clone(),
                state: s.state.clone(),
                circuit: s.circuit.clone(),
                total_requests: s.total_requests,
                total_failures: s.total_failures,
            })
            .collect(),
        models_count: state.config.models.len(),
        primary_models_count: primary.len(),
        primary_models: primary
            .iter()
            .map(|m| DiagModel {
                model: m.model.clone(),
                state: m.state.clone(),
                limited: m.limited,
            })
            .collect(),
        window_5min: DiagWindow {
            requests: s.requests,
            ok: s.ok,
            fail: s.fail,
            rate_limited: s.rate_limited,
            latency_ms_avg: s.latency_ms_avg,
            latency_ms_max: s.latency_ms_max,
            tokens_per_minute: s.tokens_per_minute,
        },
        health: if s.fail > s.ok {
            "error"
        } else if (s.fail as f64) > (s.requests as f64) * 0.2 {
            "warn"
        } else {
            "ok"
        }
        .to_string(),
        errors: vec![],
    };

    Ok(Json(diag))
}

async fn usage_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    require_auth(&state, &headers)?;
    let usage = state.usage_store.lock().unwrap().summary();
    Ok(Json(
        serde_json::to_value(&usage).unwrap_or(serde_json::json!({ "enabled": false })),
    ))
}

async fn limits_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    require_auth(&state, &headers)?;

    let metrics = state.metrics.lock().unwrap();
    let all_events: Vec<Event> = metrics.events().iter().cloned().collect();
    let model_status = ModelStatusBuilder::build(
        &all_events,
        &state.config.models,
        &state.config.primary_models,
    );
    let limits: Vec<Limit> = model_status
        .all
        .iter()
        .filter(|m| m.limited || m.rate_limit_remaining.is_some())
        .map(|m| Limit {
            model: m.model.clone(),
            limited: m.limited,
            rate_limit_remaining: m.rate_limit_remaining,
            rate_limit_limit: m.rate_limit_limit,
            reset_at: None,
            reset_in_seconds: None,
            error_type: m.error_type.clone(),
            last_status: m.last_status,
        })
        .collect();

    Ok(Json(serde_json::json!({ "limits": limits })))
}

async fn providers_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    require_auth(&state, &headers)?;
    let snapshots = state.provider_pool.snapshots().await;
    Ok(Json(serde_json::json!({ "providers": snapshots })))
}

async fn export_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    req: axum::extract::Request,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    require_auth(&state, &headers)?;

    let format = req.uri().path().trim_start_matches("/export/");
    let usage = state.usage_store.lock().unwrap();

    let content_type = if format == "json" {
        "application/json"
    } else {
        "text/csv; charset=utf-8"
    };

    let body = match format {
        "json" => export::generate_json(&usage),
        _ => export::generate_csv(&usage),
    };

    Ok(([("Content-Type", content_type)], body))
}
