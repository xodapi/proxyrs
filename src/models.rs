use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub models: Vec<String>,
    pub primary_models: Vec<String>,
    pub upstream: String,
    pub routing: String,
    pub timeout_secs: u64,
    pub max_body_bytes: u64,
    pub metrics_max_events: usize,
    pub management_token: String,
    pub usage_db_path: String,
    pub usage_retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts: i64,
    pub model: String,
    pub ok: bool,
    pub status: u16,
    pub latency_ms: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub error_type: Option<String>,
    pub rate_limited: bool,
    pub finish_reason: Option<String>,
    pub cost: f64,
    pub usage_reported: bool,
    pub usage_estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub ts: i64,
    pub requests: u64,
    pub ok: u64,
    pub fail: u64,
    pub rate_limited: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub latency_ms_avg: u64,
    pub latency_ms_max: u64,
    pub cost: f64,
    pub by_model: Vec<ModelBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBucket {
    pub model: String,
    pub requests: u64,
    pub ok: u64,
    pub fail: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u8,
    pub generated_at: String,
    pub started_at: String,
    pub uptime_seconds: u64,
    pub window_ms: u64,
    pub total_events_kept: usize,
    pub summary: Summary,
    pub timeseries: Vec<Bucket>,
    pub limits: Vec<Limit>,
    pub model_status: ModelStatus,
    pub usage: UsageSummary,
    pub recent: Vec<Event>,
    pub privacy: Privacy,
    pub routing: String,
    pub app: String,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub all: WindowSummary,
    pub window: WindowSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSummary {
    pub requests: u64,
    pub ok: u64,
    pub fail: u64,
    pub rate_limited: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub latency_ms_avg: u64,
    pub latency_ms_max: u64,
    pub tokens_per_minute: f64,
    pub requests_per_minute: f64,
    pub uptime_seconds: Option<u64>,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limit {
    pub model: String,
    pub limited: bool,
    pub rate_limit_remaining: Option<u64>,
    pub rate_limit_limit: Option<u64>,
    pub reset_at: Option<String>,
    pub reset_in_seconds: Option<u64>,
    pub error_type: Option<String>,
    pub last_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub primary: Vec<ModelInfo>,
    pub all: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model: String,
    pub state: String,
    pub last_seen_ts: Option<i64>,
    pub rate_limit_remaining: Option<u64>,
    pub rate_limit_limit: Option<u64>,
    pub limited: bool,
    pub error_type: Option<String>,
    pub last_status: Option<u16>,
    pub today: Option<ModelDayAgg>,
    pub previous_day: Option<ModelDayAgg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDayAgg {
    pub requests: u64,
    pub ok: u64,
    pub fail: u64,
    pub total_tokens: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub enabled: bool,
    pub path: Option<String>,
    pub today: String,
    pub totals: Option<UsageTotals>,
    pub by_day: Vec<DayUsage>,
    pub by_model_today: Vec<ModelUsage>,
    pub by_model_24h: Vec<ModelUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTotals {
    pub requests: u64,
    pub ok: u64,
    pub fail: u64,
    pub total_tokens: u64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayUsage {
    pub day: String,
    pub requests: u64,
    pub ok: u64,
    pub fail: u64,
    pub rate_limited: u64,
    pub total_tokens: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub latency_ms_avg: u64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    pub requests: u64,
    pub ok: u64,
    pub fail: u64,
    pub rate_limited: u64,
    pub total_tokens: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub latency_ms_avg: u64,
    pub cost: f64,
    pub usage_reported: u64,
    pub usage_estimated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Privacy {
    pub stores_prompts: bool,
    pub stores_responses: bool,
    pub stores_api_keys: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagResponse {
    pub version: String,
    pub app: String,
    pub uptime_seconds: u64,
    pub uptime_human: String,
    pub generated_at: String,
    pub routing: String,
    pub providers: Vec<DiagProvider>,
    pub models_count: usize,
    pub primary_models_count: usize,
    pub primary_models: Vec<DiagModel>,
    pub window_5min: DiagWindow,
    pub health: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagProvider {
    pub name: String,
    pub url: String,
    pub state: String,
    pub circuit: String,
    pub total_requests: u64,
    pub total_failures: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagModel {
    pub model: String,
    pub state: String,
    pub limited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagWindow {
    pub requests: u64,
    pub ok: u64,
    pub fail: u64,
    pub rate_limited: u64,
    pub latency_ms_avg: u64,
    pub latency_ms_max: u64,
    pub tokens_per_minute: f64,
}
