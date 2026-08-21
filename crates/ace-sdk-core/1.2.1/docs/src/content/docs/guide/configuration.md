---
title: Configuration
description: Config loading and XDG paths
---

## Config File Location

The SDK follows the XDG Base Directory Specification:

- **Linux/macOS**: `~/.config/ace/config.json`
- **Custom**: Set `XDG_CONFIG_HOME` environment variable

## Loading Config

```rust
use ace_sdk_core::config::{load_config, ConfigOverrides};

// Load with defaults
let config = load_config(ConfigOverrides::default());

// Or with overrides
let config = load_config(ConfigOverrides {
    server_url: Some("https://custom-server.example.com".to_string()),
    project_id: Some("prj_custom".to_string()),
    ..Default::default()
});
```

## Context Resolution

Priority: CLI arguments > Environment variables > Config file > Defaults

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ACE_SERVER_URL` | Override server URL |
| `ACE_API_TOKEN` | Override API token |
| `ACE_PROJECT_ID` | Override project ID |
| `ACE_ORG_ID` | Override organization ID |
| `XDG_CONFIG_HOME` | Custom config directory |
