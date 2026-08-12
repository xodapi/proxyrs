# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in opencode-proxy-rs, please email security concerns to the maintainers rather than using the public issue tracker. This allows us to address the issue before public disclosure.

## Security Considerations

### Data Handling

- **No Persistence of Sensitive Data**: Prompts and API responses are never persisted to disk
- **Metrics Storage**: Only aggregated usage metrics (tokens, model names, timestamps) are stored in JSONL format
- **API Keys**: Management tokens and upstream API keys are handled only in memory and never logged
- **Usage Database**: Located at `~/.config/opencode-proxy/usage.jsonl` with restricted file permissions (not enforced on Windows)

### Authentication

- **Management Token**: Optional MANAGEMENT_TOKEN environment variable protects:
  - `/dashboard` endpoint
  - `/metrics` endpoint  
  - `/diag` endpoint
  - `/usage` endpoint
  - `/export/{csv,json}` endpoint
- **Upstream Auth**: Credentials passed via Authorization header to OpenCode Zen API
- **No Token Validation**: Token is string-matched; implement rate-limiting at deployment level

### Network Security

- **CORS**: Configured for development (`GET *`)
- **CSP Headers**: Enabled for dashboard/flow pages
  ```
  default-src 'self'
  script-src 'self' 'unsafe-inline' cdn.jsdelivr.net
  style-src 'self' 'unsafe-inline' cdn.jsdelivr.net
  ```
- **HTTPS**: Not enforced in binary; use reverse proxy (nginx/caddy) for TLS in production
- **Rate Limiting**: Not implemented; use reverse proxy or API gateway

### Production Deployment Recommendations

1. **Reverse Proxy**: Place behind nginx/caddy with:
   - TLS termination
   - Rate limiting
   - IP whitelisting

2. **Environment Variables**:
   - Set `MANAGEMENT_TOKEN` to strong random value (min 32 chars)
   - Use absolute path for `USAGE_DB_PATH`
   - Set `UPSTREAM_TIMEOUT` based on your network latency

3. **Logging**:
   - Set `RUST_LOG=opencode_proxy=warn` to reduce verbosity
   - Send logs to syslog/ELK stack (not included in binary)

4. **Monitoring**:
   - Check `/health` endpoint regularly
   - Monitor `/metrics` for anomalies in token usage
   - Set alerts on `/limits` rate-limit threshold

### Known Limitations

- Circuit breaker is per-process; not distributed across instances
- No request signing or mTLS support
- Dashboard credentials cached in browser session (use HTTPS)
- User-Agent header passed through to upstream without sanitization

## Supported Versions

| Version | Status |
|---------|--------|
| 1.7.x   | Active |
| < 1.7   | Not supported |

## Updates

Subscribe to GitHub releases for security updates: https://github.com/ArtemPotapov52/opencode-proxy/releases

## Security Checklist

When deploying in production:

- [ ] Set strong `MANAGEMENT_TOKEN` (32+ chars)
- [ ] Use HTTPS via reverse proxy
- [ ] Configure rate limiting
- [ ] Enable CSP in browser (default: enabled)
- [ ] Set `RUST_LOG=warn` for production
- [ ] Monitor `/health` regularly
- [ ] Backup usage database regularly
- [ ] Review upstream API key permissions
- [ ] Test failover scenarios
