use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::Event;

pub struct MetricsStore {
    events: VecDeque<Event>,
    max_events: usize,
    started_at: SystemTime,
}

impl MetricsStore {
    pub fn new(max_events: usize) -> Self {
        let max = if max_events > 0 { max_events } else { 2000 };
        Self {
            events: VecDeque::with_capacity(max),
            max_events: max,
            started_at: SystemTime::now(),
        }
    }

    pub fn record(&mut self, event: Event) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn events(&self) -> &VecDeque<Event> {
        &self.events
    }

    pub fn recent(&self, n: usize) -> Vec<Event> {
        self.events.iter().rev().take(n).cloned().collect()
    }

    pub fn window_events(&self, window_ms: u64) -> Vec<Event> {
        let since = now_ms() - window_ms;
        self.events
            .iter()
            .filter(|e| e.ts as u64 >= since)
            .cloned()
            .collect()
    }

    pub fn total_events(&self) -> usize {
        self.events.len()
    }

    pub fn started_at(&self) -> SystemTime {
        self.started_at
    }

    pub fn started_at_ms(&self) -> u64 {
        self.started_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().unwrap_or_default().as_secs()
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn records_events() {
        let mut store = MetricsStore::new(100);
        store.record(dummy_event());
        assert_eq!(store.total_events(), 1);
    }

    #[test]
    fn respects_max_events() {
        let mut store = MetricsStore::new(5);
        for _ in 0..10 {
            store.record(dummy_event());
        }
        assert_eq!(store.total_events(), 5);
    }

    #[test]
    fn recent_returns_newest_first() {
        let mut store = MetricsStore::new(100);
        for i in 0..5 {
            let mut e = dummy_event();
            e.latency_ms = i * 100;
            store.record(e);
        }
        let recent = store.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].latency_ms, 400);
    }
}
