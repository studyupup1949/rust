# AAS Quick Start

Get AAS running in 5 minutes.

## Prerequisites

- Rust 1.75+ (or Docker)
- SQLite3
- Git (for repo agent)

## Local Development

### 1. Build

```bash
cargo build --release
```

### 2. Configure

```bash
# Default config will be auto-created, but customize if needed
mkdir -p ~/.aas
cat > ~/.aas/config.json <<'EOF'
{
  "version": "1.0",
  "metadata": {},
  "llm": {
    "provider": "mock",
    "endpoint": "http://localhost:5000",
    "model_name": "hermes-2-pro",
    "timeout_seconds": 30,
    "fallback_provider": "mock"
  },
  "agents": {
    "repository": {
      "enabled": true,
      "detection_interval": "5m",
      "platforms": ["github"],
      "auto_commit": false,
      "require_pr_approval": true,
      "max_actions_per_run": 1,
      "local_repos": []
    },
    "health": {
      "enabled": true,
      "detection_interval": "30s",
      "endpoints": [],
      "auto_restart": false,
      "max_restart_attempts": 2,
      "restart_backoff_seconds": [5, 10]
    },
    "logs": { "enabled": false },
    "metrics": { "enabled": false },
    "task": { "enabled": false },
    "trace": { "enabled": false }
  },
  "execution": {
    "mode": "staged_rollout",
    "max_concurrent_actions": 1,
    "approval_required_for": [],
    "rollback_enabled": true,
    "rollback_timeout_minutes": 60
  },
  "learning": {
    "enabled": true,
    "storage": "sqlite",
    "db_path": "/root/.aas/aas.db",
    "prediction_enabled": true,
    "prediction_confidence_threshold": 0.85,
    "memory_retention_days": 90,
    "auto_learn": true
  },
  "notifications": {
    "channels": []
  },
  "advanced": {
    "enable_telemetry": false,
    "metrics_port": 9090
  }
}
EOF
```

### 3. Run

```bash
./target/release/aas run
```

Output:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Starting Autonomous Agent System
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  LLM Provider: mock
  Agents: repository, health
  Database: /Users/you/.aas/aas.db
  Dashboard: http://localhost:3000

[INFO] Agent 'repository' started (interval: 5m)
[INFO] Agent 'health' started (interval: 30s)
```

### 4. Test

In another terminal:

```bash
# Check status
curl http://localhost:3000/api/status

# View history
./target/release/aas history

# Trigger a specific agent
./target/release/aas trigger --agent health

# View coordinator action count
./target/release/aas status
```

---

## Docker

### 1. Build Image

```bash
docker build -t aas:latest .
```

### 2. Run Container

```bash
docker run -d \
  --name aas \
  -p 3000:3000 \
  -v ~/.aas:/root/.aas \
  -e RUST_LOG=info \
  aas:latest
```

### 3. View Logs

```bash
docker logs -f aas
```

### 4. Stop

```bash
docker stop aas
```

---

## Kubernetes (Helm)

### 1. Create Namespace

```bash
kubectl create namespace agents
```

### 2. Create Secret (if using Claude)

```bash
kubectl create secret generic aas-secrets \
  --from-literal=anthropic-api-key=$ANTHROPIC_API_KEY \
  -n agents
```

### 3. Install Helm Chart

```bash
helm install aas ./helm \
  --namespace agents \
  --set image.repository=aas \
  --set image.tag=latest \
  --set secrets.anthropicApiKey=$(base64 -w0 <<<$ANTHROPIC_API_KEY)
```

### 4. Check Status

```bash
kubectl rollout status deployment/aas -n agents
kubectl get pods -n agents
kubectl logs deployment/aas -n agents
```

### 5. Forward Port

```bash
kubectl port-forward -n agents svc/aas 3000:3000
```

### 6. Test API

```bash
curl http://localhost:3000/api/status
```

---

## Common Tasks

### Configure GitHub Integration

Edit `~/.aas/config.json`:

```json
"repository": {
  "enabled": true,
  "github": {
    "organization": "my-org",
    "token": "ghp_xxxxxxxxxxxx",
    "repositories": ["repo-1", "repo-2"],
    "private_repos": true
  },
  "auto_commit": true
}
```

Then restart:

```bash
curl -X POST http://localhost:3000/api/trigger/repository
```

### Monitor Health Endpoints

Edit `~/.aas/config.json`:

```json
"health": {
  "enabled": true,
  "detection_interval": "30s",
  "endpoints": [
    "https://api.example.com/health",
    "https://www.example.com"
  ],
  "auto_restart": false
}
```

Agent will check every 30 seconds and detect failures.

### Use Claude API

Set env variable:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

Edit config:

```json
"llm": {
  "provider": "claude",
  "model_name": "claude-3-5-sonnet-20241022"
}
```

Restart.

### Enable Approval Workflow

Edit config:

```json
"execution": {
  "approval_required_for": ["commit", "restart"]
}
```

Now actions of type "commit" or "restart" will log a requirement for approval (interactive approval coming in v0.2).

---

## Troubleshooting

### Port Already in Use

```bash
# Use different port
./target/release/aas run --port 3001
```

### Database Locked

```bash
# Stop the process, then rebuild
rm ~/.aas/aas.db
./target/release/aas run
```

### No Agents Starting

```bash
# Validate config
./target/release/aas validate-config

# Enable debug logging
RUST_LOG=debug ./target/release/aas run
```

### Memory Usage High

```bash
# Check database size
du -h ~/.aas/aas.db

# If large, backup and reset
cp ~/.aas/aas.db ~/.aas/aas.db.backup
rm ~/.aas/aas.db
```

---

## Next Steps

1. **Read** [DEPLOYMENT.md](DEPLOYMENT.md) for production setup
2. **Check** [PRODUCTION_CHECKLIST.md](PRODUCTION_CHECKLIST.md) before going live
3. **Configure** agents for your use case
4. **Monitor** health checks and logs
5. **Review** audit log regularly

---

## Help

```bash
./target/release/aas --help
./target/release/aas run --help
./target/release/aas history --help
```

---

**v0.1.0** | June 2026 | Production-Ready MVP
