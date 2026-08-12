use std::collections::HashMap;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::*;

const DEFAULT_READ_LIMIT_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
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
}

pub struct UsageStore {
    enabled: bool,
    path: PathBuf,
    retention_days: u32,
    inner: Mutex<UsageStoreInner>,
}

struct UsageStoreInner {
    pending_events: Vec<StoredEvent>,
    last_error: String,
    last_prune_ts: u64,
}

impl UsageStore {
    pub fn new(path: &str, retention_days: u32) -> Self {
        let enabled = !path.is_empty();
        let store_path = if enabled {
            PathBuf::from(path)
        } else {
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config").join("opencode-proxy").join("usage.jsonl")
        };

        Self {
            enabled,
            path: store_path,
            retention_days: if retention_days > 0 { retention_days } else { 30 },
            inner: Mutex::new(UsageStoreInner {
                pending_events: Vec::new(),
                last_error: String::new(),
                last_prune_ts: 0,
            }),
        }
    }

    pub fn record(&self, event: &Event) {
        if !self.enabled { return; }

        let stored = StoredEvent {
            ts: event.ts,
            model: event.model.clone(),
            ok: event.ok,
            status: event.status,
            latency_ms: event.latency_ms,
            prompt_tokens: event.prompt_tokens,
            completion_tokens: event.completion_tokens,
            total_tokens: event.total_tokens,
            error_type: event.error_type.clone(),
            rate_limited: event.rate_limited,
            finish_reason: event.finish_reason.clone(),
            cost: event.cost,
        };

        let mut inner = self.inner.lock().unwrap();
        inner.pending_events.push(stored.clone());

        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let line = serde_json::to_string(&stored).unwrap_or_default() + "\n";
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(line.as_bytes()) {
                    inner.last_error = e.to_string();
                } else {
                    inner.pending_events.retain(|e| e.ts != stored.ts || e.model != stored.model);
                    inner.last_error = String::new();
                    let now_ts = stored.ts as u64;
                    if now_ts > inner.last_prune_ts + 3_600_000 {
                        inner.last_prune_ts = now_ts;
                        drop(inner);
                        self.prune(now_ts);
                    }
                }
            }
            Err(e) => {
                inner.last_error = e.to_string();
            }
        }
    }

    pub fn summary(&self) -> UsageSummary {
        let events = self.read_events();
        let inner = self.inner.lock().unwrap();
        let all_stored: Vec<StoredEvent> = events.into_iter()
            .chain(inner.pending_events.clone())
            .collect();
        drop(inner);
        summarize_usage(&all_stored, self.retention_days, self.enabled, &self.path)
    }

    pub fn read_events(&self) -> Vec<StoredEvent> {
        if !self.enabled {
            return Vec::new();
        }

        let file = match std::fs::File::open(&self.path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let metadata = match file.metadata() {
            Ok(m) => m,
            Err(_) => return Vec::new(),
        };

        let file_len = metadata.len();
        let read_start = if file_len > DEFAULT_READ_LIMIT_BYTES {
            file_len - DEFAULT_READ_LIMIT_BYTES
        } else {
            0
        };

        let mut reader = BufReader::new(file);
        if read_start > 0 {
            if let Err(_) = reader.seek(SeekFrom::Start(read_start)) {
                return Vec::new();
            }
            let mut skip = String::new();
            let _ = reader.read_line(&mut skip);
        }

        let mut events = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() {
                    if let Ok(event) = serde_json::from_str::<StoredEvent>(&trimmed) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }

    fn prune(&self, now_ts: u64) {
        let cutoff = now_ts - (self.retention_days as u64 * 86400 * 1000);
        let events = self.read_events();
        let filtered: Vec<StoredEvent> = events.into_iter()
            .filter(|e| (e.ts as u64) >= cutoff)
            .collect();

        let lines: Vec<String> = filtered.iter()
            .map(|e| serde_json::to_string(e).unwrap_or_default() + "\n")
            .collect();

        let content = lines.join("");
        let _ = std::fs::write(&self.path, content);
    }
}

fn summarize_usage(
    events: &[StoredEvent],
    retention_days: u32,
    enabled: bool,
    path: &PathBuf,
) -> UsageSummary {
    let now = Utc::now();
    let today_str = now.format("%Y-%m-%d").to_string();
    let cutoff_ts = (now.timestamp_millis() as u64 - retention_days as u64 * 86400 * 1000) as i64;

    let recent: Vec<&StoredEvent> = events.iter().filter(|e| e.ts >= cutoff_ts).collect();

    let mut day_map: HashMap<String, Vec<&StoredEvent>> = HashMap::new();
    for e in &recent {
        if let Some(dt) = chrono::DateTime::from_timestamp_millis(e.ts) {
            let day = dt.format("%Y-%m-%d").to_string();
            day_map.entry(day).or_default().push(e);
        }
    }

    let mut by_day: Vec<DayUsage> = day_map.into_iter()
        .map(|(day, events)| aggregate_day(&day, &events))
        .collect();
    by_day.sort_by(|a, b| b.day.cmp(&a.day));

    let today_events: Vec<&StoredEvent> = recent.iter()
        .filter(|e| {
            chrono::DateTime::from_timestamp_millis(e.ts)
                .map(|dt| dt.format("%Y-%m-%d").to_string() == today_str)
                .unwrap_or(false)
        })
        .cloned().collect();

    let by_model_today = aggregate_by_model(&today_events);

    let cutoff_24h = (now.timestamp_millis() - 86400 * 1000) as i64;
    let events_24h: Vec<&StoredEvent> = recent.iter()
        .filter(|e| e.ts >= cutoff_24h)
        .cloned().collect();
    let by_model_24h = aggregate_by_model(&events_24h);

    let totals: Option<UsageTotals> = {
        let total_events: Vec<&StoredEvent> = recent.iter().cloned().collect();
        if total_events.is_empty() {
            None
        } else {
            Some(UsageTotals {
                requests: total_events.len() as u64,
                ok: total_events.iter().filter(|e| e.ok).count() as u64,
                fail: total_events.iter().filter(|e| !e.ok).count() as u64,
                total_tokens: total_events.iter().map(|e| e.total_tokens).sum(),
                cost: total_events.iter().map(|e| e.cost).sum(),
            })
        }
    };

    UsageSummary {
        enabled,
        path: if enabled { Some(path.to_string_lossy().to_string()) } else { None },
        today: today_str,
        totals,
        by_day,
        by_model_today,
        by_model_24h,
    }
}

fn aggregate_day(day: &str, events: &[&StoredEvent]) -> DayUsage {
    let mut usage = DayUsage {
        day: day.to_string(),
        requests: 0, ok: 0, fail: 0, rate_limited: 0,
        total_tokens: 0, prompt_tokens: 0, completion_tokens: 0,
        latency_ms_avg: 0, cost: 0.0,
    };
    let mut latency_sum: u64 = 0;
    for e in events {
        usage.requests += 1;
        if e.ok { usage.ok += 1; } else { usage.fail += 1; }
        if e.rate_limited { usage.rate_limited += 1; }
        usage.total_tokens += e.total_tokens;
        usage.prompt_tokens += e.prompt_tokens;
        usage.completion_tokens += e.completion_tokens;
        latency_sum += e.latency_ms;
        usage.cost += e.cost;
    }
    if usage.requests > 0 {
        usage.latency_ms_avg = latency_sum / usage.requests;
    }
    usage
}

fn aggregate_by_model(events: &[&StoredEvent]) -> Vec<ModelUsage> {
    let mut model_map: HashMap<String, Vec<&StoredEvent>> = HashMap::new();
    for e in events {
        model_map.entry(e.model.clone()).or_default().push(e);
    }

    let mut result: Vec<ModelUsage> = model_map.into_iter()
        .map(|(model, events)| {
            let mut mu = ModelUsage {
                model, requests: 0, ok: 0, fail: 0, rate_limited: 0,
                total_tokens: 0, prompt_tokens: 0, completion_tokens: 0,
                latency_ms_avg: 0, cost: 0.0,
                usage_reported: 0, usage_estimated: 0,
            };
            let mut latency_sum: u64 = 0;
            for e in &events {
                mu.requests += 1;
                if e.ok { mu.ok += 1; } else { mu.fail += 1; }
                if e.rate_limited { mu.rate_limited += 1; }
                mu.total_tokens += e.total_tokens;
                mu.prompt_tokens += e.prompt_tokens;
                mu.completion_tokens += e.completion_tokens;
                latency_sum += e.latency_ms;
                mu.cost += e.cost;
            }
            if mu.requests > 0 {
                mu.latency_ms_avg = latency_sum / mu.requests;
            }
            mu
        })
        .collect();

    result.sort_by(|a, b| b.requests.cmp(&a.requests));
    result
}
