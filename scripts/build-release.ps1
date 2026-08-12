param(
    [string]$OutputDir = "."
)

$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$version = (Get-Content (Join-Path $repoRoot "Cargo.toml") | Select-String -Pattern '^version = "(.*)"').Matches.Groups[1].Value

Write-Host "Building opencode-proxy v$version..."

# Build
Set-Location $repoRoot
cargo build --release
if (-not $?) { exit 1 }

# Create output directory
$dist = Join-Path $OutputDir "opencode-proxy-$version"
New-Item -ItemType Directory -Path $dist -Force | Out-Null

# Copy binary
Copy-Item (Join-Path $repoRoot "target\release\opencode-proxy.exe") (Join-Path $dist "opencode-proxy.exe")

# Create README
@"
opencode-proxy v$version
========================
OpenCode Zen API proxy with monitoring dashboard.

Usage:
  opencode-proxy.exe

Environment variables:
  HOST                Bind address (default: 127.0.0.1)
  PORT                Listen port (default: 3000)
  MODELS              Comma-separated model list
  ROUTING             round-robin or random
  UPSTREAM_TIMEOUT    Upstream timeout in seconds (default: 30)
  MANAGEMENT_TOKEN    Auth token for /dashboard, /metrics, etc.

"@ | Set-Content (Join-Path $dist "README.txt")

# Zip
if (Get-Command Compress-Archive -ErrorAction SilentlyContinue) {
    $zip = Join-Path $OutputDir "opencode-proxy-$version.zip"
    Compress-Archive -Path $dist\* -DestinationPath $zip -Force
    Write-Host "Created $zip"
}

Write-Host "Release artifacts in $dist"
