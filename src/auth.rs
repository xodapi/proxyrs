const MANAGEMENT_PATHS: &[&str] = &[
    "/dashboard",
    "/flow",
    "/metrics",
    "/usage",
    "/limits",
    "/diag",
    "/export",
];

pub fn is_management_path(path: &str) -> bool {
    MANAGEMENT_PATHS.contains(&path) || path.starts_with("/export/")
}

pub fn is_authorized(token: &str, header: Option<&str>) -> bool {
    if token.is_empty() {
        return true;
    }
    match header {
        Some(h) => {
            let h = h.strip_prefix("Bearer ").unwrap_or(h);
            token == h
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_is_management() {
        assert!(is_management_path("/dashboard"));
    }

    #[test]
    fn flow_is_management() {
        assert!(is_management_path("/flow"));
    }

    #[test]
    fn health_is_not_management() {
        assert!(!is_management_path("/health"));
    }

    #[test]
    fn empty_token_allows_all() {
        assert!(is_authorized("", None));
        assert!(is_authorized("", Some("anything")));
    }

    #[test]
    fn token_auth_works() {
        assert!(is_authorized("secret", Some("Bearer secret")));
        assert!(!is_authorized("secret", Some("wrong")));
    }
}
