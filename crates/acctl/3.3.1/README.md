# acctl - AutoCore Control Tool

A command-line tool for managing AutoCore projects, control programs, and deployments.

## Installation

### From crates.io (when published)

```bash
cargo install acctl
```

### From source

```bash
git clone https://github.com/automateddesign/autocore-server.git
cd autocore-server
cargo install --path acctl
```

## Quick Start

```bash
# 1. List available projects on a server
acctl clone 192.168.1.100 --list

# 2. Clone a project to start working on it
acctl clone 192.168.1.100 my_project

# 3. Enter the project directory
cd my_project

# 4. Edit your control program
#    (edit control/src/program.rs)

# 5. Build, deploy, and start
acctl push control --start

# 6. Watch the logs
acctl logs --follow
```

## Commands

### clone

Clone a project from an AutoCore server into a new local directory.

```bash
# List available projects
acctl clone 192.168.1.100 --list

# Clone the currently active project
acctl clone 192.168.1.100

# Clone a specific project by name
acctl clone 192.168.1.100 my_project

# Clone with a custom directory name
acctl clone 192.168.1.100 my_project --directory my_local_copy

# Clone from a server on a different port
acctl clone 192.168.1.100 my_project -P 11970
```

This creates a local directory with:
- `control/` - Rust control program source
- `project.json` - Project configuration
- `www/` - Web interface files (if present)
- `acctl.toml` - Local configuration pointing to the server

The `autocore-std` library is pulled from crates.io automatically when you build.

### set-target

Set or update the target server for the current directory.

```bash
# Set target server IP
acctl set-target 192.168.1.100

# Set target with custom port
acctl set-target 192.168.1.100 --port 11970
```

Configuration is saved to `./acctl.toml` (local) or `~/.acctl.toml` (global).
Local config takes precedence.

### push

Push files to the server.

#### push control

Build and deploy the control program.

```bash
# Build, upload, and start the control program
acctl push control --start

# Build and upload without starting
acctl push control

# Upload without rebuilding (use existing binary)
acctl push control --no-build --start
```

#### push project

Push project.json configuration changes.

```bash
# Push project.json
acctl push project

# Push and restart the server to apply changes
acctl push project --restart
```

#### push www

Push web interface files.

```bash
# Push www/dist/ (production build)
acctl push www

# Push full www/ directory (including source)
acctl push www --source
```

### pull

Download the current project from the server.

```bash
# Download as zip file
acctl pull

# Download and extract
acctl pull --extract
```

### status

Show server and control program status.

```bash
acctl status
```

Example output:
```
Control Program Status:
  Status: Running (PID: 12345)

Projects Directory: /srv/autocore/projects
Available Projects:
  - my_project (valid)
  - test_project (valid)
```

### logs

View control program logs.

```bash
# Show recent logs
acctl logs

# Stream logs continuously (Ctrl+C to stop)
acctl logs --follow
```

### control

Manage the control program lifecycle.

```bash
# Start the control program
acctl control start

# Stop the control program
acctl control stop

# Restart the control program
acctl control restart

# Check status
acctl control status
```

### codegen

Regenerate the `gm.rs` (global memory mappings) file from the server.

```bash
acctl codegen
```

This downloads the latest generated code based on the project configuration.

### switch

Switch the server to a different project.

```bash
# Switch to another project
acctl switch other_project

# Switch and restart the server
acctl switch other_project --restart
```

## Configuration

acctl looks for configuration in two places (in order of precedence):

1. `./acctl.toml` - Local project configuration
2. `~/.acctl.toml` - Global user configuration

### Configuration File Format

```toml
[server]
host = "192.168.1.100"
port = 11969

[build]
release = true
```

### Command-Line Overrides

You can override configuration on any command:

```bash
# Override host
acctl --host 192.168.1.200 status

# Override port
acctl --port 11970 status

# Override both
acctl --host 192.168.1.200 --port 11970 status
```

## Typical Workflows

### Starting a New Project

```bash
# See what's available
acctl clone 192.168.1.100 --list

# Clone the project you want to work on
acctl clone 192.168.1.100 my_machine

# Start working
cd my_machine
```

### Development Cycle

```bash
# Edit control/src/program.rs

# Deploy and test
acctl push control --start

# Watch logs for issues
acctl logs --follow

# Make changes, repeat
```

### Deploying Web Interface Updates

```bash
# Build your web interface
cd www
npm run build

# Deploy to server
cd ..
acctl push www
```

### Working with Multiple Servers

Each project directory can have its own `acctl.toml` pointing to a different server:

```bash
# Project A points to server 1
cd project_a
cat acctl.toml
# [server]
# host = "192.168.1.100"

# Project B points to server 2
cd ../project_b
cat acctl.toml
# [server]
# host = "192.168.1.101"
```

### Updating Project Configuration

```bash
# Edit project.json locally

# Push changes and restart
acctl push project --restart
```

## Troubleshooting

### Connection refused

- Verify the server is running: `systemctl status autocore_server`
- Check the IP address and port
- Ensure firewall allows connections on port 11969

### Build failures

- Ensure Rust toolchain is installed: `rustup show`
- Check that the target architecture matches the server
- For cross-compilation, install the appropriate target:
  ```bash
  rustup target add aarch64-unknown-linux-gnu
  ```

### Permission denied on server

- Check that the autocore_server has write permissions to the project directory
- Verify the control binary has execute permissions after upload

## License

**Proprietary - Licensed AutoCore Users Only**

This software is licensed exclusively for use with validly licensed AutoCore Server installations. You may not use acctl to connect to or manage systems that are not licensed for AutoCore.

See the [LICENSE](LICENSE) file for complete terms.

For licensing inquiries: support@automateddesign.com
