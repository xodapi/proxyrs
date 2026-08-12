# Installation Guide

## Requirements

- Windows 10+, Linux (glibc 2.31+), or macOS 10.13+
- 50 MB disk space for binary
- 15-50 MB RAM during operation

## Option 1: Download Binary (Fastest)

### Windows

```powershell
# Download latest release
$version = "1.7.0"
$url = "https://github.com/ArtemPotapov52/opencode-proxy/releases/download/v$version/opencode-proxy-v$version-x86_64-pc-windows-msvc.zip"
Invoke-WebRequest -Uri $url -OutFile opencode-proxy.zip
Expand-Archive opencode-proxy.zip -DestinationPath .
rm opencode-proxy.zip

# Run
.\opencode-proxy.exe
```

### Linux

```bash
version="1.7.0"
url="https://github.com/ArtemPotapov52/opencode-proxy/releases/download/v${version}/opencode-proxy-v${version}-x86_64-unknown-linux-gnu"
wget "$url" -O opencode-proxy
chmod +x opencode-proxy
./opencode-proxy
```

### macOS (Intel)

```bash
version="1.7.0"
url="https://github.com/ArtemPotapov52/opencode-proxy/releases/download/v${version}/opencode-proxy-v${version}-x86_64-apple-darwin"
curl -L "$url" -o opencode-proxy
chmod +x opencode-proxy
./opencode-proxy
```

### macOS (Apple Silicon/M1)

```bash
version="1.7.0"
url="https://github.com/ArtemPotapov52/opencode-proxy/releases/download/v${version}/opencode-proxy-v${version}-aarch64-apple-darwin"
curl -L "$url" -o opencode-proxy
chmod +x opencode-proxy
./opencode-proxy
```

## Option 2: Build from Source

### Prerequisites

- Install Rust: https://rustup.rs
- Git

### Build

```bash
git clone https://github.com/ArtemPotapov52/opencode-proxy.git opencode-proxy-rs
cd opencode-proxy-rs
cargo build --release
```

Binary location:
- Windows: `target\release\opencode-proxy.exe`
- Linux/macOS: `target/release/opencode-proxy`

## Option 3: Docker

### Build

```bash
docker build -t opencode-proxy:1.7.0 .
```

### Run

```bash
docker run -p 3000:3000 \
  -e MANAGEMENT_TOKEN=your-secret-token \
  opencode-proxy:1.7.0
```

## Option 4: Systemd Service (Linux)

### 1. Copy binary

```bash
sudo cp target/release/opencode-proxy /usr/local/bin/opencode-proxy
sudo chmod +x /usr/local/bin/opencode-proxy
```

### 2. Create service file

```bash
sudo tee /etc/systemd/system/opencode-proxy.service > /dev/null <<EOF
[Unit]
Description=OpenCode Proxy
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/opencode-proxy
Restart=on-failure
RestartSec=5
Environment="PORT=3000"
Environment="MANAGEMENT_TOKEN=your-secret-token"
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
```

### 3. Enable and start

```bash
sudo systemctl daemon-reload
sudo systemctl enable opencode-proxy
sudo systemctl start opencode-proxy

# Check status
sudo systemctl status opencode-proxy

# View logs
sudo journalctl -u opencode-proxy -f
```

## Option 5: Windows Service (using NSSM)

### 1. Install NSSM

```powershell
choco install nssm
```

### 2. Install service

```powershell
$exePath = "C:\opencode-proxy\opencode-proxy.exe"
nssm install OpenCodeProxy "$exePath"
nssm set OpenCodeProxy AppEnvironmentExtra PORT=3000
nssm set OpenCodeProxy AppEnvironmentExtra MANAGEMENT_TOKEN=your-secret-token
```

### 3. Start service

```powershell
nssm start OpenCodeProxy
```

## Option 6: Homebrew (macOS)

```bash
# Add tap (when available)
brew tap ArtemPotapov52/opencode-proxy
brew install opencode-proxy

# Run
opencode-proxy
```

## Verify Installation

```bash
# Check if binary works
./opencode-proxy --version  # (if implemented)

# Or start and check health
PORT=3001 ./opencode-proxy &
sleep 2
curl http://127.0.0.1:3001/health
```

Expected output:
```json
{"status":"ok"}
```

## Uninstall

### From Binary

Simply delete the executable file.

### From Source

```bash
rm -rf opencode-proxy-rs
```

### From Systemd Service

```bash
sudo systemctl stop opencode-proxy
sudo systemctl disable opencode-proxy
sudo rm /etc/systemd/system/opencode-proxy.service
sudo systemctl daemon-reload
sudo rm /usr/local/bin/opencode-proxy
```

### From Docker

```bash
docker rmi opencode-proxy:1.7.0
```

## Troubleshooting

### "Port already in use"

```bash
# Use different port
PORT=3001 ./opencode-proxy
```

### "Command not found"

Make sure the binary is in PATH or use full path:
```bash
./opencode-proxy        # current directory
/usr/local/bin/opencode-proxy  # if installed to PATH
```

### Permission denied (Linux/macOS)

```bash
chmod +x opencode-proxy
```

### "Connection refused" to upstream

Check `UPSTREAM_URL` environment variable and network connectivity:
```bash
curl https://opencode.ai/zen/v1/models -H "Authorization: Bearer $OPENCODE_API_KEY"
```

## Next Steps

1. See [README.md](README.md) for configuration
2. See [SECURITY.md](SECURITY.md) for production deployment
3. See [CONTRIBUTING.md](CONTRIBUTING.md) for development
