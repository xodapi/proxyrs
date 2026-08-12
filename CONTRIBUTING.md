# Contributing to opencode-proxy-rs

Thank you for your interest in contributing! This document provides guidelines and instructions.

## Code of Conduct

This project is committed to providing a welcoming and inclusive environment. All contributors are expected to treat each other with respect.

## Getting Started

### Prerequisites

- Rust 1.70+ (install via https://rustup.rs)
- Windows 10+, Linux, or macOS
- Git

### Development Setup

```bash
git clone https://github.com/ArtemPotapov52/opencode-proxy.git
cd opencode-proxy-rs
cargo build
cargo test
```

### Running Locally

```bash
# Default: localhost:3000
cargo run

# Custom port
PORT=3001 cargo run

# With debugging
RUST_LOG=debug cargo run
```

## Development Workflow

### 1. Branch Naming

Use descriptive names:
- `feature/add-websocket-support`
- `fix/memory-leak-in-store`
- `docs/update-readme`

### 2. Code Style

We follow Rust conventions:

```bash
# Format code
cargo fmt

# Check lints
cargo clippy -- -D warnings

# Both before committing
cargo fmt && cargo clippy -- -D warnings
```

### 3. Testing

All code changes must include tests:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_round_robin

# With output
cargo test -- --nocapture
```

**Test coverage**: Aim for 80%+ for new code.

### 4. Documentation

- Update README.md for user-facing changes
- Add doc comments for public functions:
  ```rust
  /// Forwards request to upstream OpenCode API.
  ///
  /// # Arguments
  /// * `req` - The incoming request
  ///
  /// # Returns
  /// Response from upstream or error
  pub async fn forward_request(req: Request) -> Result<Response> { ... }
  ```

### 5. Commits

- Use imperative mood: "Add feature" not "Added feature"
- Reference issues: "Fix #42: Handle null tokens"
- One logical change per commit
- Example:
  ```
  Add circuit breaker for falling models
  
  - Detect 5xx errors on model endpoint
  - Skip for 30 seconds before retry
  - Log when model recovers
  
  Fixes #123
  ```

### 6. Pull Requests

- Keep PRs focused (one feature/fix per PR)
- Provide clear description of changes
- Link related issues: "Closes #42"
- Ensure CI passes before requesting review
- Template:
  ```markdown
  ## Description
  Brief description of what this PR does
  
  ## Why
  Why is this change needed?
  
  ## How
  How does it solve the problem?
  
  ## Testing
  - [ ] Unit tests added
  - [ ] Existing tests pass
  - [ ] Manual testing completed
  
  Closes #
  ```

## Project Structure

```
opencode-proxy-rs/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root
│   ├── server.rs         # Axum routes
│   ├── proxy.rs          # Upstream forwarding
│   ├── config.rs         # ENV parsing
│   ├── auth.rs           # Token validation
│   ├── router.rs         # Load balancing
│   ├── circuit_breaker.rs # Error handling
│   ├── usage_store.rs    # JSONL storage
│   ├── models.rs         # Data structures
│   ├── export.rs         # CSV/JSON export
│   ├── metrics/          # Metrics aggregation
│   ├── templates/        # HTML dashboards
│   └── utils.rs          # Utilities
├── tests/
│   └── integration.rs    # Integration tests
├── scripts/
│   └── build-release.ps1 # Release build
├── Cargo.toml
├── build.rs              # Build script
└── README.md
```

## Architecture Notes

### Key Modules

- **server.rs**: Handles HTTP routing, serves static templates, implements auth
- **proxy.rs**: JSON and streaming passthrough to upstream
- **router.rs**: Round-robin or random model selection
- **metrics**: Per-request recording and window-based aggregation
- **usage_store.rs**: JSONL appending, daily pruning

### Async Model

Uses `tokio` for concurrency. All I/O is non-blocking.

## Common Tasks

### Add a New Endpoint

1. Define the handler in `server.rs`
2. Add route in `Router::new()`
3. If protected: add `check_token()` guard
4. Write tests in same file or `tests/integration.rs`

### Add a New Environment Variable

1. Add to `config.rs` struct with default
2. Parse in `Config::from_env()`
3. Document in `README.md`

### Add Metrics for a New Field

1. Update `models.rs` usage structure
2. Record in `proxy.rs` after upstream response
3. Aggregate in `metrics/snapshot.rs`
4. Expose in `/metrics` endpoint

## Performance Considerations

- Avoid allocations in hot paths
- Use streaming for large responses
- Pool connections with `reqwest::Client`
- Ring buffer instead of unbounded Vec for metrics

## Security Guidelines

- Never log full API keys or prompts
- Validate all user input (headers, query params)
- Use `serde` for JSON parsing (safe deserialization)
- Keep dependencies updated: `cargo outdated`

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` (if exists)
3. Run `cargo test --release`
4. Tag commit: `git tag v1.8.0`
5. Push tags: `git push --tags`
6. GitHub Actions builds and uploads binaries

## Questions?

- Check existing issues/discussions
- Open a new discussion for design questions
- Ask in PR comments for code-level help

## License

By contributing, you agree that your code will be licensed under MIT License.
