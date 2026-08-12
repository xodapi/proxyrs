# Testing Guide

## How to Test opencode-proxy-rs

### 1. Unit Tests

Run all tests:
```bash
cargo test
```

Run with output:
```bash
cargo test -- --nocapture
```

Run specific test:
```bash
cargo test test_health_returns_ok
```

**Current test coverage:**
- Router: round-robin, random strategies
- Auth: token validation
- Config: ENV parsing
- Metrics: recording and aggregation
- Export: CSV/JSON generation
- Integration: full endpoint cycle

### 2. Integration Tests

Test all endpoints locally:

```bash
# 1. Start the proxy
cargo run --release &

# 2. Wait for startup
sleep 2

# 3. Test health check
curl http://127.0.0.1:3000/health

# 4. Test models endpoint
curl http://127.0.0.1:3000/v1/models | jq '.data | length'

# 5. Test dashboard (should fail without token)
curl -i http://127.0.0.1:3000/dashboard
# Expected: 401 Unauthorized

# 6. Test with token
curl -H "Authorization: Bearer test123" \
  http://127.0.0.1:3000/dashboard | head -20

# 7. Stop proxy
pkill opencode-proxy
```

### 3. Load Testing

Install wrk (load testing tool):
```bash
# Windows: choco install wrk
# macOS: brew install wrk
# Linux: apt install wrk
```

Run benchmark:
```bash
# Start proxy
cargo run --release &

# Wait 2 seconds
sleep 2

# Run 30-second load test
wrk -t4 -c100 -d30s http://127.0.0.1:3000/health

# Expected output:
# Requests/sec: ~1000+
# Latency avg: <100ms

# Stop proxy
pkill opencode-proxy
```

### 4. Streaming Test

Test SSE streaming response:

```bash
# Start proxy
cargo run --release &

# Wait for startup
sleep 2

# Test streaming
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Say hello"}],
    "stream": true
  }' | head -5

# Expected: data: {"choices":[{"delta":...}]}
# one per line

pkill opencode-proxy
```

### 5. Configuration Testing

Test with different configurations:

```bash
# Test custom port
PORT=3001 cargo run --release &
sleep 2
curl http://127.0.0.1:3001/health
pkill opencode-proxy

# Test custom models
MODELS=gpt-4,claude-3 cargo run --release &
sleep 2
curl http://127.0.0.1:3000/v1/models | jq '.data | length'
pkill opencode-proxy

# Test auth token
MANAGEMENT_TOKEN=mysecret cargo run --release &
sleep 2
curl http://127.0.0.1:3000/metrics
# Should fail (401)
curl -H "Authorization: Bearer mysecret" \
  http://127.0.0.1:3000/metrics | jq '.version'
pkill opencode-proxy
```

### 6. Error Path Testing

Test error scenarios:

```bash
# Start proxy
cargo run --release &
sleep 2

# Test invalid JSON
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"invalid": json}'
# Expected: 400 Bad Request

# Test missing model
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages": [{"role": "user", "content": "hi"}]}'
# Expected: error about missing model

# Test unauthorized endpoint
curl http://127.0.0.1:3000/metrics
# Expected: 401 if MANAGEMENT_TOKEN is set

pkill opencode-proxy
```

### 7. Performance Testing

Measure performance characteristics:

```bash
# Build release binary
cargo build --release

# Check binary size
ls -lh target/release/opencode-proxy
# Expected: ~2.6 MB

# Measure startup time
time ./target/release/opencode-proxy --help

# Monitor memory during operation
./target/release/opencode-proxy &
PID=$!
sleep 2

# Windows
Get-Process -Id $PID | Select-Object WorkingSet

# Linux
ps aux | grep opencode-proxy | grep -v grep

# Stop
kill $PID
```

### 8. Code Quality Checks

Format check:
```bash
cargo fmt --check
```

Linting:
```bash
cargo clippy -- -D warnings
```

Security audit:
```bash
cargo audit
```

### 9. Docker Testing

Build and test Docker image:

```bash
# Build image
docker build -t opencode-proxy:test .

# Run container
docker run -p 3000:3000 \
  -e MANAGEMENT_TOKEN=secret \
  opencode-proxy:test &

sleep 3

# Test endpoints
curl http://127.0.0.1:3000/health

curl -H "Authorization: Bearer secret" \
  http://127.0.0.1:3000/metrics | jq '.version'

# Stop container
docker stop $(docker ps -q --filter ancestor=opencode-proxy:test)
```

### 10. Automated Test Suite

Run full test suite:

```bash
#!/bin/bash
set -e

echo "1. Running unit tests..."
cargo test

echo "2. Checking format..."
cargo fmt --check

echo "3. Running clippy..."
cargo clippy -- -D warnings

echo "4. Building release..."
cargo build --release

echo "5. Running integration tests..."
cargo run --release &
PID=$!
sleep 2

# Test endpoints
curl -f http://127.0.0.1:3000/health
curl -f http://127.0.0.1:3000/v1/models
curl -f http://127.0.0.1:3000/playground

# Cleanup
kill $PID

echo "✅ All tests passed!"
```

## Testing with Different Upstream APIs

### Mock Upstream (for CI/CD)

Create `tests/mock_upstream.rs`:

```rust
use axum::{routing::post, Router};
use serde_json::json;

#[tokio::test]
async fn test_with_mock_upstream() {
    // Start mock upstream on :8001
    let mock = Router::new()
        .route("/v1/chat/completions", 
               post(|| async { json!({"choices": [{"message": {"content": "test"}}]}) }));
    
    // Set UPSTREAM_URL=http://127.0.0.1:8001
    // Run proxy
    // Test proxy
}
```

### Real Upstream (manual testing)

```bash
UPSTREAM_URL=https://opencode.ai/zen/v1 \
MODELS=gpt-4 \
cargo run --release
```

## Continuous Testing (GitHub Actions)

The CI/CD pipeline automatically runs:
- ✅ Tests on Windows, Linux, macOS
- ✅ Format check (cargo fmt)
- ✅ Linting (cargo clippy -D warnings)
- ✅ Security audit (cargo audit)
- ✅ Multi-target builds
- ✅ Coverage reporting

View results: https://github.com/xodapi/opencode-proxy/actions

## Performance Regression Testing

Compare current vs previous performance:

```bash
# Build baseline
cargo build --release
baseline_size=$(ls -l target/release/opencode-proxy | awk '{print $5}')

# Make changes
# ...

# Build current
cargo build --release
current_size=$(ls -l target/release/opencode-proxy | awk '{print $5}')

# Compare
echo "Baseline: $baseline_size bytes"
echo "Current: $current_size bytes"
if [ $current_size -gt $baseline_size ]; then
    echo "⚠️ Binary grew by $((current_size - baseline_size)) bytes"
fi
```

## Test Coverage

Generate coverage report (requires cargo-tarpaulin):

```bash
cargo install cargo-tarpaulin

cargo tarpaulin --out Html
```

Open `tarpaulin-report.html` in browser.

## Troubleshooting Tests

### Tests hang
```bash
# Kill hanging processes
pkill -9 opencode-proxy
```

### Port already in use
```bash
# Use different port in test
PORT=3001 cargo test
```

### Timeout errors
```bash
# Increase timeout
RUST_TEST_THREADS=1 cargo test -- --test-threads=1 --nocapture
```

## Next Steps

Once tests pass:
1. ✅ Run full test suite
2. ✅ Check performance (no regressions)
3. ✅ Verify on all platforms (Windows, Linux, macOS)
4. ✅ Create GitHub release with binaries
5. ✅ Deploy to production

See [PERFORMANCE.md](PERFORMANCE.md) for benchmarking details.
