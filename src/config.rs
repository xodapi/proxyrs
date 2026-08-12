use std::env;

const DEFAULT_MODELS: &[&str] = &[
    "big-pickle",
    "deepseek-v4-flash-free",
    "mimo-v2.5-free",
    "north-mini-code-free",
    "nemotron-3-ultra-free",
];

const DEFAULT_PRIMARY_MODELS: &[&str] = &[
    "deepseek-v4-flash-free",
    "mimo-v2.5-free",
    "north-mini-code-free",
    "nemotron-3-ultra-free",
];

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3001;
const DEFAULT_UPSTREAM: &str = "https://opencode.ai/zen/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_MAX_BODY_BYTES: u64 = 2 * 1024 * 1024;
const DEFAULT_METRICS_MAX_EVENTS: usize = 2000;
const DEFAULT_USAGE_RETENTION_DAYS: u32 = 30;
const DEFAULT_CIRCUIT_FAILURES: u32 = 3;
const DEFAULT_CIRCUIT_RESET_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub models: Vec<String>,
    pub primary_models: Vec<String>,
    pub upstream: String,
    pub upstreams: Vec<(String, String, String)>,
    pub api_key: String,
    pub routing: String,
    pub timeout_secs: u64,
    pub max_body_bytes: u64,
    pub metrics_max_events: usize,
    pub management_token: String,
    pub usage_db_path: String,
    pub usage_retention_days: u32,
    pub logger: bool,
    pub access_log: bool,
    pub probe_interval_ms: u64,
    pub circuit_max_failures: u32,
    pub circuit_reset_secs: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let models = parse_list(env::var("MODELS").ok());
        let active_models = if models.is_empty() {
            DEFAULT_MODELS.iter().map(|s| s.to_string()).collect()
        } else {
            models
        };

        let primary = parse_list(env::var("PRIMARY_MODELS").ok());
        let primary_models = if primary.is_empty() {
            DEFAULT_PRIMARY_MODELS
                .iter()
                .filter(|m| active_models.contains(&m.to_string()))
                .map(|s| s.to_string())
                .collect()
        } else {
            primary
        };

        let host = env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_PORT);

        let upstream = env::var("UPSTREAM_URL").unwrap_or_else(|_| DEFAULT_UPSTREAM.to_string());
        let api_key = env::var("OPENCODE_ZEN_API_KEY").unwrap_or_else(|_| "public".to_string());
        let upstreams = parse_upstreams(&upstream, &api_key);

        let routing = match env::var("ROUTING").ok() {
            Some(v) if v == "random" => "random".to_string(),
            Some(v) if v == "fallback" => "fallback".to_string(),
            _ => "round-robin".to_string(),
        };

        Self {
            host,
            port,
            models: active_models,
            primary_models,
            upstream,
            upstreams,
            api_key,
            routing,
            timeout_secs: env::var("UPSTREAM_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_TIMEOUT_SECS),
            max_body_bytes: env::var("MAX_BODY_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_MAX_BODY_BYTES),
            metrics_max_events: env::var("METRICS_MAX_EVENTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_METRICS_MAX_EVENTS),
            management_token: env::var("MANAGEMENT_TOKEN")
                .or_else(|_| env::var("OPENCODE_PROXY_TOKEN"))
                .unwrap_or_default(),
            usage_db_path: env::var("USAGE_DB_PATH").unwrap_or_default(),
            usage_retention_days: env::var("USAGE_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_USAGE_RETENTION_DAYS),
            logger: true,
            access_log: env::var("ACCESS_LOG")
                .ok()
                .map(|v| v != "0" && v != "false")
                .unwrap_or(true),
            probe_interval_ms: env::var("PROBE_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30000),
            circuit_max_failures: env::var("CIRCUIT_MAX_FAILURES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_CIRCUIT_FAILURES),
            circuit_reset_secs: env::var("CIRCUIT_RESET_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_CIRCUIT_RESET_SECS),
        }
    }
}

fn parse_list(value: Option<String>) -> Vec<String> {
    match value {
        Some(v) => v
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        None => Vec::new(),
    }
}

fn parse_upstreams(upstream: &str, default_key: &str) -> Vec<(String, String, String)> {
    let urls: Vec<String> = upstream
        .split(',')
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    urls.into_iter()
        .enumerate()
        .map(|(i, url)| {
            let name = format!("provider-{}", i + 1);
            let key_env = format!("API_KEY_{}", i + 1);
            let api_key = std::env::var(&key_env).unwrap_or_else(|_| default_key.to_string());
            (url, api_key, name)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_models() {
        let config = Config::from_env();
        assert!(!config.models.is_empty());
    }

    #[test]
    fn parses_model_list() {
        let list = parse_list(Some("a,b,c".to_string()));
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn empty_list_returns_empty() {
        let list = parse_list(None);
        assert!(list.is_empty());
    }
}
