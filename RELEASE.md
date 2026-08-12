# Release Checklist

Use this checklist when preparing a new release.

## Pre-Release

- [ ] All tests pass: `cargo test --release`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt --check`
- [ ] Dependencies up to date: `cargo outdated`
- [ ] Security audit passed: `cargo audit`
- [ ] Create feature branch: `git checkout -b release/v1.x.x`

## Documentation

- [ ] Update `CHANGELOG.md` with new version
- [ ] Update version in `Cargo.toml`
- [ ] Review README.md for outdated info
- [ ] Update SECURITY.md if applicable
- [ ] Add migration notes if breaking changes

## Testing

- [ ] Run integration tests: `cargo test --release`
- [ ] Test on Windows: `cargo build --release --target x86_64-pc-windows-msvc`
- [ ] Test on Linux (if applicable): `cargo build --release --target x86_64-unknown-linux-gnu`
- [ ] Test on macOS (if applicable): `cargo build --release --target x86_64-apple-darwin`
- [ ] Manual smoke test: Start proxy, call `/health`, verify `/metrics`

## Release Commit

```bash
git add -A
git commit -m "Release v1.x.x

- Feature 1
- Feature 2
- Fix: bug fix

Closes #123, #456"
```

## Tag and Push

```bash
git tag -a v1.x.x -m "Release version 1.x.x"
git push origin release/v1.x.x
git push origin v1.x.x
```

## GitHub Release

1. Go to https://github.com/ArtemPotapov52/opencode-proxy/releases/new
2. Select tag `v1.x.x`
3. Title: `Release v1.x.x`
4. Description: Copy from CHANGELOG.md
5. Upload binaries from GitHub Actions artifacts
6. Mark as latest release
7. Publish

## Post-Release

- [ ] Verify GitHub Actions ran successfully
- [ ] Verify release artifacts are available
- [ ] Merge PR to main branch if applicable
- [ ] Announce in discussions/social media (optional)
- [ ] Update version to next dev version: `1.x.x-dev` in Cargo.toml

## Rollback (if needed)

```bash
git tag -d v1.x.x
git push origin :refs/tags/v1.x.x
```

## Supported Platforms

- Windows (x86_64)
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64/M1)

## Binary Naming

- `opencode-proxy` (Linux/macOS)
- `opencode-proxy.exe` (Windows)
- Include target in filename: `opencode-proxy-v1.7.0-x86_64-pc-windows-msvc.zip`
