use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Strategy {
    RoundRobin,
    Random,
}

#[derive(Debug, Clone)]
pub struct Router {
    models: Arc<Vec<String>>,
    strategy: Strategy,
    counter: Arc<AtomicUsize>,
    auto_counter: Arc<AtomicUsize>,
}

impl Router {
    pub fn new(models: Vec<String>, strategy: Strategy) -> Self {
        Self {
            models: Arc::new(models),
            strategy,
            counter: Arc::new(AtomicUsize::new(0)),
            auto_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn select(&self, requested_model: &str) -> String {
        if self.models.contains(&requested_model.to_string()) {
            return requested_model.to_string();
        }

        if requested_model == "auto" || requested_model == "*" || requested_model.is_empty() {
            return self.select_next();
        }

        self.select_next()
    }

    pub fn select_next(&self) -> String {
        match self.strategy {
            Strategy::RoundRobin => {
                let idx = self.counter.fetch_add(1, Ordering::Relaxed) % self.models.len();
                self.models[idx].clone()
            }
            Strategy::Random => {
                let idx = rand::random::<usize>() % self.models.len();
                self.models[idx].clone()
            }
        }
    }

    pub fn auto_model(&self) -> String {
        let idx = self.auto_counter.fetch_add(1, Ordering::Relaxed) % self.models.len();
        self.models[idx].clone()
    }

    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    pub fn models(&self) -> &[String] {
        &self.models
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_models() -> Vec<String> {
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    }

    #[test]
    fn exact_match_returns_requested() {
        let router = Router::new(test_models(), Strategy::RoundRobin);
        assert_eq!(router.select("b"), "b");
    }

    #[test]
    fn unknown_model_selects_next() {
        let router = Router::new(test_models(), Strategy::RoundRobin);
        let selected = router.select("unknown");
        assert!(test_models().contains(&selected));
    }

    #[test]
    fn round_robin_rotates() {
        let router = Router::new(test_models(), Strategy::RoundRobin);
        let first = router.select_next();
        let second = router.select_next();
        let third = router.select_next();
        assert_ne!(first, second);
        assert_ne!(second, third);
        // 4th call cycles back to first
        let fourth = router.select_next();
        assert_eq!(first, fourth);
    }

    #[test]
    fn auto_model_rotates() {
        let router = Router::new(test_models(), Strategy::RoundRobin);
        let m = router.auto_model();
        assert!(test_models().contains(&m));
    }
}
