# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.7.0] - 2026-07-02

### Added
- Initial Rust implementation of OpenCode proxy
- OpenAI-compatible `/v1/chat/completions` endpoint
- `/v1/models` list endpoint
- Streaming SSE passthrough with usage parsing
- Load balancing: round-robin and random strategies
- Management token (`MANAGEMENT_TOKEN`) authentication
- Dashboard UI with real-time metrics
- Flow visualization page
- Usage analytics with JSONL persistence
- Daily retention policy for usage data
- CSV/JSON export endpoints
- Circuit breaker for fallback handling
- Security headers (CSP, X-Content-Type-Options, X-Frame-Options)
- Health check endpoint `/health`
- Diagnostics endpoint `/diag`
- Metrics endpoint `/metrics` with window-based aggregation
- 17 unit tests covering core functionality
- Release build script with optimization (2.6 MB binary)
- GitHub Actions CI/CD pipeline
- Comprehensive documentation (README, SECURITY, CONTRIBUTING)

### Fixed
- Rate limit header extraction from upstream responses
- SSE parsing for multi-line data chunks
- Memory usage in metrics aggregation

### Performance
- Single-threaded Node.js → multi-threaded Tokio
- 50MB → 2.6MB binary size
- Zero external dependencies (fully vendored)

## [Unreleased]

### Planned
- WebSocket support for real-time metrics
- Improved circuit breaker with per-model tracking
- Retry logic with exponential backoff
- Prometheus `/metrics/prometheus` export
- Linux and macOS binary releases
- Self-update mechanism
- Binary code signing (Authenticode for Windows)
- Semantic caching layer
- Support for Claude API (`/v1/messages`)

---

## Versioning

We use [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking API changes or major features
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, no new features
