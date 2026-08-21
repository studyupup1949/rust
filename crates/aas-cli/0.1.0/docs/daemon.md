# AAS Daemon Mode

Run AAS as a background service instead of a foreground process.

## Quick Start

```bash
# Start daemon
aas daemon start
✓ Daemon started (PID: 12345)
  View logs: aas daemon logs --follow

# Check status
aas daemon status
✓ Daemon running (PID: 12345)
  Logs: aas daemon logs --follow

# View logs
aas daemon logs --follow

# Stop daemon
aas daemon stop
✓ Daemon stopped
```

## How It Works

### Start

```bash
aas daemon start
```

1. Checks if daemon already running (reads PID file)
2. Spawns `aas run --daemon-mode` as background process
3. Stores PID in `~/.local/share/aas/.aas.pid`
4. Returns immediately

### Status

```bash
aas daemon status
```

1. Reads PID file
2. Checks if process still running (kill -0 on Unix, tasklist on Windows)
3. Reports running or stale

### Logs

```bash
aas daemon logs --follow
```

Streams agent output from `~/.local/share/aas/logs/aas.log`

### Stop

```bash
aas daemon stop
```

1. Reads PID file
2. Sends SIGTERM to process (kill on Unix, taskkill on Windows)
3. Removes PID file
4. Returns

## Foreground vs. Daemon

| Mode | Command | Exit With | Use Case |
|------|---------|-----------|----------|
| Foreground | `aas run` | Ctrl+C | Development, debugging |
| Foreground (timed) | `aas run --duration 5m` | After 5 minutes | Testing, CI/CD |
| Daemon | `aas daemon start` | Background | Production, server |

## Files

- **PID file**: `~/.local/share/aas/.aas.pid` — stores daemon process ID
- **Log file**: `~/.local/share/aas/logs/aas.log` — agent output
- **Config**: `~/.config/aas/config.toml` — used by daemon

## Troubleshooting

### Daemon won't start

Check if another instance is running:

```bash
ps aux | grep "aas run"
aas daemon status
```

If PID file is stale:

```bash
rm ~/.local/share/aas/.aas.pid
aas daemon start
```

### Can't see logs

```bash
# Check if log directory exists
ls -la ~/.local/share/aas/logs/

# Check if daemon is writing
tail -f ~/.local/share/aas/logs/aas.log

# Check daemon status
aas daemon status
```

### Kill stuck daemon

```bash
# Find process
ps aux | grep aas

# Kill by PID
kill -9 <PID>

# Clean up
rm ~/.local/share/aas/.aas.pid
aas daemon start
```

## Environment Variables

- `XDG_DATA_HOME` — override log/config location (defaults to `~/.local/share`)
- `RUST_LOG` — control log level (set in daemon process)
- `AAS_LLM_API_KEY` — LLM authentication (read by daemon)

## Systemd Integration (Linux)

To run AAS as a systemd service:

```ini
# ~/.config/systemd/user/aas.service
[Unit]
Description=Autonomous Agent System
After=network.target

[Service]
Type=forking
ExecStart=/path/to/aas daemon start
ExecStop=/path/to/aas daemon stop
Restart=on-failure
User=%u

[Install]
WantedBy=default.target
```

Then:

```bash
systemctl --user enable aas
systemctl --user start aas
systemctl --user status aas
```

## Launchd Integration (macOS)

To run AAS on login (macOS):

```xml
<!-- ~/Library/LaunchAgents/ca.thealxlabs.aas.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>ca.thealxlabs.aas</string>
    <key>Program</key>
    <string>/path/to/aas</string>
    <key>ProgramArguments</key>
    <array>
        <string>daemon</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Then:

```bash
launchctl load ~/Library/LaunchAgents/ca.thealxlabs.aas.plist
```
