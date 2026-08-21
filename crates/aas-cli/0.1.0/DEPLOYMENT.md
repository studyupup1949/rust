# AAS Production Deployment Guide

## Overview

AAS (Autonomous Agent System) is production-ready with the following guarantees:
- **Graceful shutdown**: 30-second drain for in-flight actions
- **Health checks**: Kubernetes liveness and readiness probes
- **Command validation**: Dangerous patterns blocked before execution
- **Structured logging**: All logs include metadata for monitoring
- **Audit trail**: Every action is traceable and queryable

---

## Quick Start

### Local Development

```bash
cargo build --release
./target/release/aas run
```

Logs to stdout. Config saved to `~/.aas/config.json`, database at `~/.aas/aas.db`.

### Health Checks

```bash
curl http://localhost:3000/health    # 200 if running
curl http://localhost:3000/ready     # 200 if ready
```

---

## Docker

### Build

```bash
docker build -t aas:latest .
```

### Run

```bash
docker run -d \
  --name aas \
  -p 3000:3000 \
  -v ~/.aas:/root/.aas \
  -e RUST_LOG=info \
  aas:latest
```

Environment variables:
- `RUST_LOG`: Log level (trace, debug, info, warn, error)
- `AAS_LLM_API_KEY`: Claude API key (or set in config)
- `ANTHROPIC_API_KEY`: Alternative Claude key

---

## Kubernetes

### Helm Chart

```bash
helm install aas ./helm \
  --namespace agents \
  --values production-values.yaml
```

### Manifest (Without Helm)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aas
  namespace: agents
spec:
  replicas: 1
  selector:
    matchLabels:
      app: aas
  template:
    metadata:
      labels:
        app: aas
    spec:
      containers:
      - name: aas
        image: aas:latest
        ports:
        - containerPort: 3000
          name: http
        env:
        - name: RUST_LOG
          value: info
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: aas-secrets
              key: anthropic-api-key
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
          timeoutSeconds: 3
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 2
          periodSeconds: 5
          timeoutSeconds: 2
          failureThreshold: 2
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "1000m"
        volumeMounts:
        - name: config
          mountPath: /root/.aas
      volumes:
      - name: config
        persistentVolumeClaim:
          claimName: aas-pvc
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: aas-pvc
  namespace: agents
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
---
apiVersion: v1
kind: Service
metadata:
  name: aas
  namespace: agents
spec:
  type: ClusterIP
  selector:
    app: aas
  ports:
  - name: http
    port: 3000
    targetPort: 3000
```

### Create Secrets

```bash
kubectl create secret generic aas-secrets \
  --from-literal=anthropic-api-key=$ANTHROPIC_API_KEY \
  -n agents
```

---

## Configuration

Config location: `~/.aas/config.json`

Minimal production config:

```json
{
  "version": "1.0",
  "metadata": {
    "created_at": "2026-06-21T00:00:00Z"
  },
  "llm": {
    "provider": "claude",
    "endpoint": "https://api.anthropic.com",
    "model_name": "claude-3-5-sonnet-20241022",
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
      "local_repos": ["/path/to/repo"]
    },
    "health": {
      "enabled": true,
      "detection_interval": "30s",
      "endpoints": ["https://api.example.com/health"],
      "auto_restart": false,
      "max_restart_attempts": 2,
      "restart_backoff_seconds": [5, 10]
    },
    "logs": {
      "enabled": true,
      "detection_interval": "continuous",
      "sources": [],
      "auto_fix": false,
      "escalate_on_unknown": true
    },
    "metrics": {
      "enabled": true,
      "detection_interval": "1m",
      "providers": [],
      "auto_scale": false,
      "optimization_enabled": false
    },
    "task": {
      "enabled": false,
      "detection_interval": "10m",
      "auto_execute": false
    },
    "trace": {
      "enabled": false,
      "detection_interval": "5m"
    }
  },
  "execution": {
    "mode": "staged_rollout",
    "local_only": true,
    "max_concurrent_actions": 1,
    "approval_required_for": ["commit", "restart"],
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
    "channels": [],
    "triggers": {
      "on_action": true,
      "on_failure": true,
      "on_pattern_learned": false
    }
  },
  "advanced": {
    "enable_telemetry": false,
    "metrics_port": 9090
  }
}
```

---

## Safety & Compliance

### Command Validation

All commands are validated before execution. Blocked patterns:
- `rm -rf /`
- `dd if=`
- `:(){:|:&};:`
- `DROP TABLE`
- `TRUNCATE TABLE`
- `mkfs`
- `shred`

Risky but allowed (logged): `rm -rf`, `sudo`

### Rate Limiting

Actions with identical signatures are rate-limited to once per 5 minutes. Prevents runaway loops.

### Audit Log

Every action is logged to database with:
- Timestamp
- Agent name
- Action ID
- Commands executed
- Exit code
- Verification result

Query via CLI:

```bash
aas history --agent repository --limit 100
aas history --agent health --export history.json
```

### Approval Workflows

Set `approval_required_for` in config to require approval before running specific action types:

```json
"approval_required_for": ["commit", "restart", "scale"]
```

Currently logged only; future releases will support interactive approval via API/CLI.

### Rollback

Failed actions are automatically rolled back if `rollback_commands` are defined and `rollback_enabled: true`. Rollback timeout: 60 minutes (configurable).

---

## Monitoring

### Health Endpoints

- `GET /health` — Liveness (200 if coordinator running)
- `GET /ready` — Readiness (200 if agents running and ready)
- `GET /api/status` — Full status JSON
- `GET /api/agents` — Enabled agents

### Metrics (Future)

Prometheus metrics available on port 9090:
- `aas_issues_detected_total`
- `aas_actions_executed_total`
- `aas_action_success_rate`
- `aas_rollback_total`

### Logging

Logs include:
- Timestamp (ISO 8601)
- Level (TRACE, DEBUG, INFO, WARN, ERROR)
- Target (module path)
- Message
- Thread ID (for concurrency debugging)

Example:

```
2026-06-21T00:10:58.071659Z INFO aas::config::settings: Configuration saved to /Users/alexanderthegreat/.aas/config.json
2026-06-21T00:10:58.076100Z INFO aas::memory::store: Database initialized successfully
2026-06-21T00:10:58.081872Z INFO aas::swarm::event_bus: [EVENT] "agent_started" at 2026-06-21 00:10:58.081850 UTC
```

---

## Troubleshooting

### Agents not starting

Check config:

```bash
aas validate-config
```

Logs:

```bash
RUST_LOG=debug ./target/release/aas run
```

### Actions failing

Check audit log:

```bash
aas history --limit 10
```

Check command validation (dangerous patterns):

```bash
grep "Command validation" logs.txt
```

### High memory usage

Check database size:

```bash
du -h ~/.aas/aas.db
```

Prune old records (future feature; for now, delete and recreate):

```bash
rm ~/.aas/aas.db
```

### Slow detection intervals

Check system metrics:

```bash
aas performance --agent metrics
```

Increase interval:

```json
"agents": {
  "metrics": {
    "detection_interval": "5m"
  }
}
```

---

## Performance Tuning

### Concurrency

Default: 3 concurrent actions. Increase for high-volume setups:

```json
"execution": {
  "max_concurrent_actions": 5
}
```

### Memory

- Database: ~100MB for 100k decisions
- Event bus: In-memory only (consider SQLite persistence for large deployments)
- Agent state: ~10MB per agent

Typical pod: 256MB request, 512MB limit.

### CPU

Agents are I/O-bound (HTTP, git, database). 250m CPU request usually sufficient.

---

## Backup & Recovery

### Backup

```bash
aas backup
# Creates ~/.aas/backup-YYYYMMDD-hhmmss.tar.gz
```

### Restore

```bash
aas restore backup-20260621-001058.tar.gz
```

### Database Backup (Manual)

```bash
cp ~/.aas/aas.db ~/.aas/aas.db.backup-$(date +%s)
```

---

## Upgrade Path

No database migrations needed. Drop-in compatible:

1. Build new binary
2. Replace in-service binary
3. Restart pod/service
4. Existing config and database are compatible

---

## Support

Issues? Check:
1. `aas validate-config` — Configuration errors
2. `aas logs --follow` — Real-time logs
3. `aas performance` — Resource usage
4. Database: `sqlite3 ~/.aas/aas.db "SELECT * FROM decisions LIMIT 10;"`

---

**Version**: 0.1.0 (MVP)  
**Stability**: Production-ready (with caveats)  
**Last Updated**: 2026-06-21
