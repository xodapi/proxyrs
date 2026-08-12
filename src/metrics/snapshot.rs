use std::collections::HashMap;
use chrono::Utc;

use crate::models::*;
use crate::metrics::store::MetricsStore;

pub struct SnapshotBuilder;

impl SnapshotBuilder {
    pub fn build(
        store: &MetricsStore,
        window_ms: u64,
        model_status: &ModelStatus,
        usage: UsageSummary,
        _primary_models: &[String],
        routing: &str,
    ) -> Snapshot {
        let window_events = store.window_events(window_ms);
        let all_events: Vec<Event> = store.events().iter().cloned().collect();

        let window_summary = Self::summarize_events(&window_events);
        let all_summary = Self::summarize_events(&all_events);

        let timeseries = Self::build_timeseries(&all_events);

        let limits = Self::build_limits(model_status);

        let recent = store.recent(20);

        let generated_at = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let started_at = chrono::DateTime::from_timestamp_millis(store.started_at_ms() as i64)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
            .unwrap_or_else(|| generated_at.clone());

        Snapshot {
            version: 1,
            generated_at,
            started_at,
            uptime_seconds: store.uptime_secs(),
            window_ms,
            total_events_kept: store.total_events(),
            summary: Summary {
                all: all_summary,
                window: window_summary,
            },
            timeseries,
            limits,
            model_status: model_status.clone(),
            usage,
            recent,
            privacy: Privacy {
                stores_prompts: false,
                stores_responses: false,
                stores_api_keys: false,
                note: "Prompts, responses, API keys, session IDs, and local paths are never stored.".to_string(),
            },
            routing: routing.to_string(),
            app: "opencode-proxy".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn summarize_events(events: &[Event]) -> WindowSummary {
        let mut total = WindowSummary {
            requests: 0, ok: 0, fail: 0, rate_limited: 0,
            prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
            latency_ms_avg: 0, latency_ms_max: 0,
            tokens_per_minute: 0.0, requests_per_minute: 0.0,
            uptime_seconds: None, cost: 0.0,
        };

        if events.is_empty() {
            return total;
        }

        let mut latency_sum: u64 = 0;
        let mut first_ts: i64 = i64::MAX;
        let mut last_ts: i64 = 0;

        for e in events {
            total.requests += 1;
            if e.ok { total.ok += 1; } else { total.fail += 1; }
            if e.rate_limited { total.rate_limited += 1; }
            total.prompt_tokens += e.prompt_tokens;
            total.completion_tokens += e.completion_tokens;
            total.total_tokens += e.total_tokens;
            latency_sum += e.latency_ms;
            if e.latency_ms > total.latency_ms_max { total.latency_ms_max = e.latency_ms; }
            total.cost += e.cost;
            if e.ts < first_ts { first_ts = e.ts; }
            if e.ts > last_ts { last_ts = e.ts; }
        }

        total.latency_ms_avg = if total.requests > 0 { latency_sum / total.requests } else { 0 };

        let duration_min = if last_ts > first_ts {
            ((last_ts - first_ts) as f64 / 60000.0).max(0.5)
        } else {
            0.5
        };

        total.tokens_per_minute = total.total_tokens as f64 / duration_min;
        total.requests_per_minute = total.requests as f64 / duration_min;

        total
    }

    fn build_timeseries(events: &[Event]) -> Vec<Bucket> {
        let mut buckets: HashMap<i64, Bucket> = HashMap::new();

        for e in events {
            let bucket_ts = (e.ts as i64 / 60000) * 60000;
            let entry = buckets.entry(bucket_ts).or_insert(Bucket {
                ts: bucket_ts,
                requests: 0, ok: 0, fail: 0, rate_limited: 0,
                prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
                latency_ms_avg: 0, latency_ms_max: 0,
                cost: 0.0, by_model: Vec::new(),
            });

            entry.requests += 1;
            if e.ok { entry.ok += 1; } else { entry.fail += 1; }
            if e.rate_limited { entry.rate_limited += 1; }
            entry.prompt_tokens += e.prompt_tokens;
            entry.completion_tokens += e.completion_tokens;
            entry.total_tokens += e.total_tokens;
            if e.latency_ms > entry.latency_ms_max { entry.latency_ms_max = e.latency_ms; }
            entry.cost += e.cost;

            let model_entry = entry.by_model.iter_mut().find(|m| m.model == e.model);
            match model_entry {
                Some(m) => {
                    m.requests += 1;
                    if e.ok { m.ok += 1; } else { m.fail += 1; }
                    m.total_tokens += e.total_tokens;
                }
                None => {
                    entry.by_model.push(ModelBucket {
                        model: e.model.clone(),
                        requests: 1,
                        ok: if e.ok { 1 } else { 0 },
                        fail: if e.ok { 0 } else { 1 },
                        total_tokens: e.total_tokens,
                    });
                }
            }
        }

        for entry in buckets.values_mut() {
            if entry.requests > 0 {
                let latency_sum: u64 = events.iter()
                    .filter(|e| (e.ts as i64 / 60000) * 60000 == entry.ts)
                    .map(|e| e.latency_ms)
                    .sum();
                entry.latency_ms_avg = latency_sum / entry.requests;
            }
        }

        let mut result: Vec<Bucket> = buckets.into_values().collect();
        result.sort_by_key(|b| b.ts);
        result
    }

    fn build_limits(model_status: &ModelStatus) -> Vec<Limit> {
        model_status
            .all
            .iter()
            .filter(|m| m.rate_limit_remaining.is_some() || m.limited || m.error_type.is_some())
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
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::store::{MetricsStore, now_ms};

    fn dummy_event() -> Event {
        Event {
            ts: now_ms() as i64,
            model: "test".to_string(),
            ok: true,
            status: 200,
            latency_ms: 100,
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
            error_type: None,
            rate_limited: false,
            finish_reason: Some("stop".to_string()),
            cost: 0.0,
            usage_reported: true,
            usage_estimated: false,
        }
    }

    #[test]
    fn empty_snapshot_is_valid() {
        let store = MetricsStore::new(100);
        let ms = ModelStatus { primary: vec![], all: vec![] };
        let usage = UsageSummary {
            enabled: false, path: None, today: String::new(),
            totals: None, by_day: vec![], by_model_today: vec![], by_model_24h: vec![],
        };
        let snap = SnapshotBuilder::build(&store, 300000, &ms, usage, &[], "round-robin");
        assert_eq!(snap.version, 1);
        assert_eq!(snap.summary.window.requests, 0);
    }

    #[test]
    fn snapshot_has_recent_events() {
        let mut store = MetricsStore::new(100);
        for _ in 0..5 {
            store.record(dummy_event());
        }
        let ms = ModelStatus { primary: vec![], all: vec![] };
        let usage = UsageSummary {
            enabled: false, path: None, today: String::new(),
            totals: None, by_day: vec![], by_model_today: vec![], by_model_24h: vec![],
        };
        let snap = SnapshotBuilder::build(&store, 300000, &ms, usage, &[], "round-robin");
        assert_eq!(snap.recent.len(), 5);
    }
}
