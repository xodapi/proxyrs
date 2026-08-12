use serde::Serialize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderState {
    Healthy,
    Degraded,
    Down,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct Provider {
    pub url: String,
    pub api_key: String,
    pub name: String,
    state: ProviderState,
    circuit: CircuitState,
    consecutive_failures: u32,
    last_failure: Option<Instant>,
    total_requests: u64,
    total_failures: u64,
    total_tokens: u64,
    max_failures: u32,
    reset_timeout: Duration,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderSnapshot {
    pub name: String,
    pub url: String,
    pub state: String,
    pub circuit: String,
    pub consecutive_failures: u32,
    pub total_requests: u64,
    pub total_failures: u64,
    pub total_tokens: u64,
}

#[derive(Clone)]
pub struct ProviderPool {
    providers: Arc<Mutex<Vec<Provider>>>,
    counter: Arc<AtomicUsize>,
    strategy: RoutingStrategy,
}

#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    RoundRobin,
    Random,
    Fallback,
}

impl Provider {
    pub fn new(
        url: String,
        api_key: String,
        name: String,
        max_failures: u32,
        reset_timeout: Duration,
    ) -> Self {
        Self {
            url,
            api_key,
            name,
            state: ProviderState::Healthy,
            circuit: CircuitState::Closed,
            consecutive_failures: 0,
            last_failure: None,
            total_requests: 0,
            total_failures: 0,
            total_tokens: 0,
            max_failures,
            reset_timeout,
        }
    }

    pub fn is_available(&self) -> bool {
        match &self.circuit {
            CircuitState::Closed => true,
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.reset_timeout {
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.state = ProviderState::Healthy;
        self.circuit = CircuitState::Closed;
        self.total_requests += 1;
    }

    pub fn record_failure(&mut self, status: u16) {
        self.consecutive_failures += 1;
        self.last_failure = Some(Instant::now());
        self.total_requests += 1;
        self.total_failures += 1;

        match status {
            429 => {
                self.state = ProviderState::Degraded;
                if self.consecutive_failures >= self.max_failures {
                    self.circuit = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                    self.state = ProviderState::Down;
                }
            }
            500..=599 => {
                if self.consecutive_failures >= self.max_failures {
                    self.circuit = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                    self.state = ProviderState::Down;
                } else {
                    self.state = ProviderState::Degraded;
                }
            }
            _ => {
                if self.consecutive_failures >= self.max_failures {
                    self.circuit = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                    self.state = ProviderState::Down;
                }
            }
        }
    }

    pub fn add_tokens(&mut self, tokens: u64) {
        self.total_tokens += tokens;
    }

    pub fn snapshot(&self) -> ProviderSnapshot {
        ProviderSnapshot {
            name: self.name.clone(),
            url: self.url.clone(),
            state: format!("{:?}", self.state),
            circuit: format!("{:?}", self.circuit),
            consecutive_failures: self.consecutive_failures,
            total_requests: self.total_requests,
            total_failures: self.total_failures,
            total_tokens: self.total_tokens,
        }
    }
}

impl ProviderPool {
    pub fn new(
        upstreams: Vec<(String, String, String)>,
        strategy: RoutingStrategy,
        max_failures: u32,
        reset_secs: u64,
    ) -> Self {
        let reset_timeout = Duration::from_secs(reset_secs);
        let providers: Vec<Provider> = upstreams
            .into_iter()
            .map(|(url, api_key, name)| {
                Provider::new(url, api_key, name, max_failures, reset_timeout)
            })
            .collect();

        Self {
            providers: Arc::new(Mutex::new(providers)),
            counter: Arc::new(AtomicUsize::new(0)),
            strategy,
        }
    }

    pub async fn select(&self) -> Option<(String, String, String)> {
        let providers = self.providers.lock().await;

        match &self.strategy {
            RoutingStrategy::RoundRobin => {
                let len = providers.len();
                if len == 0 {
                    return None;
                }

                for _ in 0..len {
                    let idx = self.counter.fetch_add(1, Ordering::Relaxed) % len;
                    if providers[idx].is_available() {
                        let p = &providers[idx];
                        return Some((p.url.clone(), p.api_key.clone(), p.name.clone()));
                    }
                }
                None
            }
            RoutingStrategy::Random => {
                let available: Vec<&Provider> =
                    providers.iter().filter(|p| p.is_available()).collect();
                if available.is_empty() {
                    return None;
                }
                let idx = rand::random::<usize>() % available.len();
                let p = available[idx];
                Some((p.url.clone(), p.api_key.clone(), p.name.clone()))
            }
            RoutingStrategy::Fallback => {
                for p in providers.iter() {
                    if p.is_available() {
                        return Some((p.url.clone(), p.api_key.clone(), p.name.clone()));
                    }
                }
                None
            }
        }
    }

    pub async fn record_result(&self, name: &str, ok: bool, status: u16, tokens: u64) {
        let mut providers = self.providers.lock().await;
        for p in providers.iter_mut() {
            if p.name == name {
                if ok {
                    p.record_success();
                } else {
                    p.record_failure(status);
                }
                p.add_tokens(tokens);
                break;
            }
        }
    }

    pub async fn snapshots(&self) -> Vec<ProviderSnapshot> {
        let providers = self.providers.lock().await;
        providers.iter().map(|p| p.snapshot()).collect()
    }

    pub async fn probe(&self) {
        let mut providers = self.providers.lock().await;
        for p in providers.iter_mut() {
            if let CircuitState::Open { opened_at } = &p.circuit {
                if opened_at.elapsed() >= p.reset_timeout {
                    p.circuit = CircuitState::HalfOpen;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_provider(max_failures: u32, reset_secs: u64) -> Provider {
        Provider::new(
            "https://test.example.com/v1".to_string(),
            "test-key".to_string(),
            "test".to_string(),
            max_failures,
            Duration::from_secs(reset_secs),
        )
    }

    #[test]
    fn provider_starts_healthy() {
        let p = make_provider(3, 30);
        assert!(p.is_available());
        assert_eq!(p.state, ProviderState::Healthy);
    }

    #[test]
    fn single_failure_keeps_healthy() {
        let mut p = make_provider(3, 30);
        p.record_failure(500);
        assert!(p.is_available());
        assert_eq!(p.state, ProviderState::Degraded);
    }

    #[test]
    fn max_failures_opens_circuit() {
        let mut p = make_provider(2, 30);
        p.record_failure(500);
        p.record_failure(500);
        assert!(!p.is_available());
        assert_eq!(p.state, ProviderState::Down);
    }

    #[test]
    fn success_resets_circuit() {
        let mut p = make_provider(2, 30);
        p.record_failure(500);
        p.record_failure(500);
        assert!(!p.is_available());
        p.record_success();
        assert!(p.is_available());
        assert_eq!(p.state, ProviderState::Healthy);
    }

    #[test]
    fn rate_limit_counts_as_failure() {
        let mut p = make_provider(3, 30);
        p.record_failure(429);
        assert_eq!(p.consecutive_failures, 1);
        assert_eq!(p.state, ProviderState::Degraded);
    }

    #[tokio::test]
    async fn pool_selects_available_provider() {
        let upstreams = vec![
            (
                "https://a.example.com".to_string(),
                "key-a".to_string(),
                "a".to_string(),
            ),
            (
                "https://b.example.com".to_string(),
                "key-b".to_string(),
                "b".to_string(),
            ),
        ];
        let pool = ProviderPool::new(upstreams, RoutingStrategy::RoundRobin, 2, 30);
        let selected = pool.select().await;
        assert!(selected.is_some());
    }

    #[tokio::test]
    async fn pool_skips_down_provider() {
        let upstreams = vec![
            (
                "https://a.example.com".to_string(),
                "key-a".to_string(),
                "a".to_string(),
            ),
            (
                "https://b.example.com".to_string(),
                "key-b".to_string(),
                "b".to_string(),
            ),
        ];
        let pool = ProviderPool::new(upstreams, RoutingStrategy::Fallback, 1, 30);

        pool.record_result("a", false, 500, 0).await;

        let selected = pool.select().await;
        assert!(selected.is_some());
        let (_, _, name) = selected.unwrap();
        assert_eq!(name, "b");
    }
}
