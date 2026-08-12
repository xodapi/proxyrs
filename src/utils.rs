pub fn unique_strings(values: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    values
        .iter()
        .filter(|v| {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() || seen.contains(&trimmed) {
                false
            } else {
                seen.insert(trimmed.clone());
                true
            }
        })
        .cloned()
        .collect()
}

pub fn number_or_null(value: Option<f64>) -> Option<f64> {
    match value {
        Some(v) if v.is_finite() => Some(v),
        _ => None,
    }
}

pub fn positive_integer(value: Option<usize>, fallback: usize) -> usize {
    match value {
        Some(v) if v > 0 => v,
        _ => fallback,
    }
}

pub fn round(value: f64, digits: u32) -> f64 {
    let factor = 10f64.powi(digits as i32);
    (value * factor).round() / factor
}

pub fn format_short_num(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn format_duration(seconds: u64) -> String {
    let d = seconds / 86400;
    let h = (seconds % 86400) / 3600;
    let m = (seconds % 3600) / 60;
    if d > 0 {
        format!("{}д {}ч", d, h)
    } else if h > 0 {
        format!("{}ч {}м", h, m)
    } else {
        format!("{}м", m)
    }
}

pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
