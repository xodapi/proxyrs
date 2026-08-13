use std::collections::HashMap;

use crate::usage_store::UsageStore;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportRecord {
    pub day: String,
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
}

pub fn generate_csv(store: &UsageStore) -> String {
    let records = collect_records(store);
    let mut lines = vec!["day,model,requests,ok,fail,rate_limited,total_tokens,prompt_tokens,completion_tokens,latency_ms_avg,cost".to_string()];
    for r in &records {
        lines.push(format!(
            "{},{},{},{},{},{},{},{},{},{},{}",
            r.day,
            r.model,
            r.requests,
            r.ok,
            r.fail,
            r.rate_limited,
            r.total_tokens,
            r.prompt_tokens,
            r.completion_tokens,
            r.latency_ms_avg,
            r.cost
        ));
    }
    lines.join("\n") + "\n"
}

pub fn generate_json(store: &UsageStore) -> String {
    let records = collect_records(store);
    serde_json::to_string_pretty(&records).unwrap_or_else(|_| "[]".to_string())
}

fn collect_records(store: &UsageStore) -> Vec<ExportRecord> {
    let events = store.read_events();
    let now = chrono::Utc::now();
    let retention_days = 30;
    let cutoff_ts = (now.timestamp_millis() as u64 - retention_days as u64 * 86400 * 1000) as i64;

    let mut day_model_map: HashMap<(String, String), Vec<&crate::usage_store::StoredEvent>> =
        HashMap::new();
    for e in &events {
        if e.ts < cutoff_ts {
            continue;
        }
        if let Some(dt) = chrono::DateTime::from_timestamp_millis(e.ts) {
            let day = dt.format("%Y-%m-%d").to_string();
            let key = (day, e.model.clone());
            day_model_map.entry(key).or_default().push(e);
        }
    }

    let mut records: Vec<ExportRecord> = day_model_map
        .into_iter()
        .map(|((day, model), events)| {
            let mut rec = ExportRecord {
                day,
                model,
                requests: 0,
                ok: 0,
                fail: 0,
                rate_limited: 0,
                total_tokens: 0,
                prompt_tokens: 0,
                completion_tokens: 0,
                latency_ms_avg: 0,
                cost: 0.0,
            };
            let mut latency_sum: u64 = 0;
            for e in &events {
                rec.requests += 1;
                if e.ok {
                    rec.ok += 1;
                } else {
                    rec.fail += 1;
                }
                if e.rate_limited {
                    rec.rate_limited += 1;
                }
                rec.total_tokens += e.total_tokens;
                rec.prompt_tokens += e.prompt_tokens;
                rec.completion_tokens += e.completion_tokens;
                latency_sum += e.latency_ms;
                rec.cost += e.cost;
            }
            if let Some(avg) = latency_sum.checked_div(rec.requests) {
                rec.latency_ms_avg = avg;
            }
            rec
        })
        .collect();

    records.sort_by_key(|r| (std::cmp::Reverse(r.day.clone()), r.model.clone()));
    records
}
