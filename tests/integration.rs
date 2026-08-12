use axum::body::Body;
use axum::http::{Request, StatusCode};
use opencode_proxy::config::Config;
use opencode_proxy::server;
use serde_json;
use tower::util::ServiceExt;

fn test_config() -> Config {
    let mut config = Config::from_env();
    config.management_token.clear();
    config.usage_db_path.clear();
    config
}

fn authed_config() -> Config {
    let mut config = Config::from_env();
    config.management_token = "test123".to_string();
    config.usage_db_path.clear();
    config
}

async fn get(app: &mut axum::Router, uri: &str) -> axum::response::Response<Body> {
    app.clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn get_authed(app: &mut axum::Router, uri: &str) -> axum::response::Response<Body> {
    app.clone()
        .oneshot(
            Request::builder()
                .uri(uri)
                .header("Authorization", "Bearer test123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn health_returns_ok() {
    let mut app = server::build_router(test_config());
    let resp = get(&mut app, "/health").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn models_returns_list() {
    let mut app = server::build_router(test_config());
    let resp = get(&mut app, "/v1/models").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["object"], "list");
    assert!(!json["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn dashboard_no_auth_returns_unauthorized() {
    let mut app = server::build_router(authed_config());
    let resp = get(&mut app, "/dashboard").await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn dashboard_with_auth_returns_html() {
    let mut app = server::build_router(authed_config());
    let resp = get_authed(&mut app, "/dashboard").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/html"));
}

#[tokio::test]
async fn export_csv_returns_csv() {
    let mut app = server::build_router(authed_config());
    let resp = get_authed(&mut app, "/export/csv").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/csv"));
}

#[tokio::test]
async fn export_json_returns_json() {
    let mut app = server::build_router(authed_config());
    let resp = get_authed(&mut app, "/export/json").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/json"));
}

#[tokio::test]
async fn usage_no_auth_returns_unauthorized() {
    let mut app = server::build_router(authed_config());
    let resp = get(&mut app, "/usage").await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn limits_no_auth_returns_unauthorized() {
    let mut app = server::build_router(authed_config());
    let resp = get(&mut app, "/limits").await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn playground_returns_html() {
    let mut app = server::build_router(test_config());
    let resp = get(&mut app, "/playground").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/html"));
}

#[tokio::test]
async fn diag_no_auth_returns_unauthorized() {
    let mut app = server::build_router(authed_config());
    let resp = get(&mut app, "/diag").await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn flow_returns_html() {
    let mut app = server::build_router(test_config());
    let resp = get(&mut app, "/flow").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/html"));
}

#[tokio::test]
async fn metrics_no_auth_returns_unauthorized() {
    let mut app = server::build_router(authed_config());
    let resp = get(&mut app, "/metrics").await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn metrics_with_auth_returns_json() {
    let mut app = server::build_router(authed_config());
    let resp = get_authed(&mut app, "/metrics").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("application/json"));
}

#[tokio::test]
async fn usage_with_auth_returns_json() {
    let mut app = server::build_router(authed_config());
    let resp = get_authed(&mut app, "/usage").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("enabled").is_some());
}

#[tokio::test]
async fn providers_no_auth_returns_unauthorized() {
    let mut app = server::build_router(authed_config());
    let resp = get(&mut app, "/providers").await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn providers_with_auth_returns_json() {
    let mut app = server::build_router(authed_config());
    let resp = get_authed(&mut app, "/providers").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("providers").is_some());
    assert!(json["providers"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn limits_with_auth_returns_json() {
    let mut app = server::build_router(authed_config());
    let resp = get_authed(&mut app, "/limits").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("limits").is_some());
}
