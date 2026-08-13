use crate::models::*;
use std::collections::HashMap;

pub struct ModelStatusBuilder;

impl ModelStatusBuilder {
    pub fn build(
        events: &[Event],
        all_models: &[String],
        primary_models: &[String],
    ) -> ModelStatus {
        let mut model_map: HashMap<String, ModelInfo> = HashMap::new();

        for model in all_models {
            model_map.insert(
                model.clone(),
                ModelInfo {
                    model: model.clone(),
                    state: "untested".to_string(),
                    last_seen_ts: None,
                    rate_limit_remaining: None,
                    rate_limit_limit: None,
                    limited: false,
                    error_type: None,
                    last_status: None,
                    today: None,
                    previous_day: None,
                },
            );
        }

        for e in events {
            let entry = model_map.entry(e.model.clone()).or_insert(ModelInfo {
                model: e.model.clone(),
                state: String::new(),
                last_seen_ts: None,
                rate_limit_remaining: None,
                rate_limit_limit: None,
                limited: false,
                error_type: None,
                last_status: None,
                today: None,
                previous_day: None,
            });

            entry.last_seen_ts = Some(e.ts);
            entry.last_status = Some(e.status);
            if !e.ok {
                entry.state = "error".to_string();
                entry.error_type = e.error_type.clone();
            } else if entry.state.is_empty() || entry.state == "untested" {
                entry.state = "available".to_string();
            }
            if e.rate_limited {
                entry.state = "limited".to_string();
                entry.limited = true;
            }
        }

        let mut all: Vec<ModelInfo> = model_map.into_values().collect();
        all.sort_by_key(|a| a.model.clone());

        let primary: Vec<ModelInfo> = primary_models
            .iter()
            .filter_map(|name| all.iter().find(|m| &m.model == name))
            .cloned()
            .collect();

        ModelStatus { primary, all }
    }
}
