#!/bin/bash
# OpenCode Proxy CLI Management Tool
# Usage: ./run.sh <command> [args...]

CMD="${1:-start}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

case "$CMD" in
  start)
    echo "Starting OpenCode Proxy on port 3001..."
    exec ./target/release/opencode-proxy
    ;;
  
  dev)
    echo "Starting in development mode..."
    cargo run
    ;;
  
  build)
    echo "Building release binary..."
    cargo build --release
    echo "Binary ready: ./target/release/opencode-proxy"
    ;;
  
  test)
    echo "Running tests..."
    cargo test
    ;;
  
  health)
    echo "Checking proxy health..."
    curl -s http://127.0.0.1:3001/health | jq .
    ;;
  
  status)
    echo "Proxy Status:"
    curl -s -H "Authorization: Bearer $(cat .env | grep MANAGEMENT_TOKEN | cut -d= -f2)" \
      http://127.0.0.1:3001/metrics | jq .
    ;;
  
  setup-factory)
    echo "Setting up Factory Droid integration..."
    if [ ! -d "$HOME/.factory" ]; then
      echo "Factory not found at $HOME/.factory"
      exit 1
    fi
    
    # Add custom model to settings.json
    SETTINGS="$HOME/.factory/settings.json"
    if grep -q "deepseek-v4-flash-free.*OpenCode Proxy" "$SETTINGS"; then
      echo "✓ OpenCode Proxy model already configured"
    else
      echo "Adding OpenCode Proxy model to Factory settings..."
      # Note: Requires jq for JSON manipulation
      echo "Manual setup: Add this to ~/.factory/settings.json customModels:"
      cat <<'EOF'
{
  "model": "deepseek-v4-flash-free",
  "id": "custom:opencode-deepseek-v4-flash-free",
  "displayName": "DeepSeek V4 Flash [OpenCode Proxy]",
  "baseUrl": "http://127.0.0.1:3001/v1",
  "apiKey": "public",
  "provider": "generic-chat-completion-api"
}
EOF
    fi
    ;;
  
  doctor)
    echo "OpenCode Proxy Diagnostics"
    echo "=========================="
    echo ""
    echo "1. Proxy Status:"
    if curl -s http://127.0.0.1:3001/health > /dev/null 2>&1; then
      echo "   ✓ Proxy is running"
      curl -s http://127.0.0.1:3001/health | jq .
    else
      echo "   ✗ Proxy is NOT running"
      echo "   Start with: ./run.sh start"
    fi
    
    echo ""
    echo "2. Available Models:"
    curl -s http://127.0.0.1:3001/v1/models | jq '.data[] | {id, name}' 2>/dev/null || echo "   Unable to fetch models"
    
    echo ""
    echo "3. Configuration:"
    echo "   PORT: $(grep '^PORT=' .env | cut -d= -f2 || echo '3001')"
    echo "   UPSTREAM: $(grep '^UPSTREAM_URL=' .env | cut -d= -f2 || echo 'https://opencode.ai/zen/v1')"
    echo "   MODELS: $(grep '^MODELS=' .env | cut -d= -f2 || echo 'See .env')"
    ;;
  
  *)
    echo "OpenCode Proxy CLI"
    echo ""
    echo "Usage: ./run.sh <command>"
    echo ""
    echo "Commands:"
    echo "  start              - Start proxy server (port 3001)"
    echo "  dev                - Start in development mode (watch mode)"
    echo "  build              - Build release binary"
    echo "  test               - Run test suite"
    echo "  health             - Check proxy health"
    echo "  status             - Show proxy metrics"
    echo "  setup-factory      - Configure Factory Droid integration"
    echo "  doctor             - Run diagnostics"
    echo ""
    exit 1
    ;;
esac
