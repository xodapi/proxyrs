# Troubleshooting Guide

## Common Issues and Solutions

### Port Already in Use

**Problem**: `Address already in use` error on startup

**Solution**: Use a different port
```bash
PORT=3001 cargo run --release
```

**Check what's using the port**:
```bash
# Windows
netstat -ano | findstr :3000

# Linux/macOS
lsof -i :3000
```

---

### Connection Refused to Upstream

**Problem**: `Error: Failed to connect to upstream`

**Solution**: Verify upstream URL and network connectivity

```bash
# Test upstream API directly
curl https://opencode.ai/zen/v1/models \
  -H "Authorization: Bearer YOUR_API_KEY"

# Check UPSTREAM_URL in .env
cat .env | grep UPSTREAM_URL
```

**Common causes**:
- VPN not connected
- Firewall blocking access
- Wrong UPSTREAM_URL (typo)
- Upstream API is down

---

### Dashboard Returns 401 Unauthorized

**Problem**: `/dashboard` endpoint returns 401 error

**Solution**: Set MANAGEMENT_TOKEN and include it in request

**Option 1: Add to .env**
```bash
MANAGEMENT_TOKEN=my-secret-token-123
cargo run --release
```

**Option 2: Pass via environment**
```bash
MANAGEMENT_TOKEN=my-secret-token-123 cargo run --release
```

**Option 3: Test with curl**
```bash
curl -H "Authorization: Bearer my-secret-token-123" \
  http://127.0.0.1:3000/dashboard
```

---

### Streaming Responses Not Working

**Problem**: Chat completions with `stream: true` don't stream

**Causes**:
- Upstream doesn't support streaming
- Client not handling SSE format
- Network buffering issue

**Test streaming manually**:
```bash
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Say hello"}],
    "stream": true
  }' | grep "data:"
```

**Expected output**:
```
data: {"choices":[{"delta":{"content":"Hello"}}]}
data: {"choices":[{"delta":{"content":" "}}]}
...
data: [DONE]
```

If not streaming, check:
1. Upstream supports streaming
2. Client properly handles SSE
3. No proxy between you and opencode-proxy stripping headers

---

### Models List is Empty

**Problem**: `/v1/models` returns empty array

**Solution**: Check MODELS environment variable

```bash
# See current models
echo $MODELS

# Set models in .env
MODELS=gpt-4,gpt-4-turbo,gpt-3.5-turbo
cargo run --release
```

**Test**:
```bash
curl http://127.0.0.1:3000/v1/models | jq '.data | length'
# Should return: 3 (or number of models)
```

---

### Health Check Fails

**Problem**: `/health` returns error

**Solution**: Binary startup issue

```bash
# Test basic connectivity
curl http://127.0.0.1:3000/health

# Check binary is running
ps aux | grep opencode-proxy

# If not running, check error logs
cargo run --release  # Run in foreground to see errors
```

**Expected response**:
```json
{"status":"ok"}
```

---

### Metrics Endpoint Locked

**Problem**: `/metrics` returns 401 without token

**Solution**: This is expected behavior for security

**If you didn't set MANAGEMENT_TOKEN**:
```bash
# No token required if not set
curl http://127.0.0.1:3000/metrics
```

**If you set MANAGEMENT_TOKEN**:
```bash
# Token required
curl -H "Authorization: Bearer your-token" \
  http://127.0.0.1:3000/metrics
```

---

### Usage Database Not Persisting

**Problem**: Usage data resets after restart

**Solution**: Check USAGE_DB_PATH permissions

```bash
# Check file exists and is writable
ls -la ~/.config/opencode-proxy/usage.jsonl

# Ensure directory exists
mkdir -p ~/.config/opencode-proxy

# Check permissions
chmod 755 ~/.config/opencode-proxy
chmod 644 ~/.config/opencode-proxy/usage.jsonl
```

**Set custom path if needed**:
```bash
USAGE_DB_PATH=/tmp/opencode-usage.jsonl cargo run --release
```

---

### Performance Issues / High Latency

**Problem**: Requests are slow

**Causes**:
1. Upstream API is slow
2. Network latency to upstream
3. Local resource constraints

**Debug**:
```bash
# Measure request time
time curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4","messages":[{"role":"user","content":"hi"}]}'

# Check metrics for latency stats
curl http://127.0.0.1:3000/metrics | jq '.window'
```

**Solutions**:
- Check upstream performance
- Verify network connectivity
- Monitor system resources (CPU, RAM, disk)

---

### Proxy Won't Start on Docker

**Problem**: Container exits immediately

**Solution**: Check Docker logs and environment variables

```bash
# View logs
docker logs opencode-proxy

# Run with environment variables
docker run -p 3000:3000 \
  -e MANAGEMENT_TOKEN=secret \
  -e UPSTREAM_URL=https://opencode.ai/zen/v1 \
  opencode-proxy:latest

# Or with .env file
docker run -p 3000:3000 \
  --env-file .env \
  opencode-proxy:latest
```

---

### SSL/TLS Certificate Errors

**Problem**: `certificate verify failed` errors

**Solution**: This means using HTTPS to upstream

```bash
# If upstream uses self-signed cert (dev only):
UPSTREAM_URL=https://your-upstream.local opencode-proxy

# For production, ensure cert is valid and in system trust store
```

---

### Still Having Issues?

**Gather diagnostic info**:
```bash
./target/release/opencode-proxy --health 2>&1 | tee diagnostic.log
```

**Or manually check**:
```bash
# 1. Verify binary runs
./target/release/opencode-proxy

# 2. Test in another terminal
curl http://127.0.0.1:3000/health

# 3. Check configuration
env | grep -i opencode
cat .env

# 4. View logs with debug level
RUST_LOG=debug ./target/release/opencode-proxy
```

**Report issues** with:
- Error message (full text)
- Configuration (without secrets)
- Steps to reproduce
- Operating system and Rust version
