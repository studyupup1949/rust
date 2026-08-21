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

# Upload the entire control source directory (instead of binary)
acctl push control --source
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

### upload

Upload a file to the project directory on the server. This is useful for uploading support files like IODD device descriptors, ESI files, or other resources that modules need.

```bash
# Upload a file to the default location (lib/<filename>)
acctl upload MyDevice.xml

# Upload to a specific path relative to the project directory
acctl upload MyDevice.xml --dest lib/iodd/MyDevice.xml

# Upload an IODD zip file
acctl upload BALLUFF-BNI_IOL-302-xxx-Z01x-20231201-IODD1.1.zip --dest lib/balluff.zip
```

**Security notes:**
- The destination path must be relative (no leading `/`)
- Path traversal (`..`) is not allowed
- Files are uploaded to paths relative to the project directory on the server

**Common use case - IODD file for IO-Link configuration:**

```bash
# 1. Upload the IODD file to the server
acctl upload BNI_IOL-302.xml

# 2. Configure an EtherCAT slot to use it
acctl cmd ethercat.configure --device IMPACT67_0 ImportIodd --slot 0 --file lib/BNI_IOL-302.xml
```

The `lib/` directory is included when you clone or pull a project, so IODD files and other support files are preserved across development machines.

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

### cmd

Send a command to the server, similar to the AutoCore console. This provides direct access to all server endpoints.

```bash
# Basic syntax: acctl cmd <domain.command> [args...]
acctl cmd system.get_domains

# Get EtherCAT status
acctl cmd ethercat.get_status

# List devices configured in the project
acctl cmd ethercat.list_devices

# Configure an EtherCAT device (with arguments)
acctl cmd ethercat.configure --device RC8_0 ListProfiles
acctl cmd ethercat.configure --device RC8_0 SelectProfile --profile Standard

# Control program management
acctl cmd system.control --action status
acctl cmd system.control --action start
acctl cmd system.control --action stop

# Project management on the server
acctl cmd system.list_projects
acctl cmd system.new_project --project_name my_new_project

# Read/write variables
acctl cmd gm.read --name my_variable
acctl cmd gm.write --name my_variable --value 42

# Generate linked variables from EtherCAT PDOs
acctl cmd ethercat.generate_variables
```

Arguments use the same syntax as the AutoCore console:
- `--name value` or `-n value` for named arguments
- Positional arguments are collected into `action` if single, or `_args` array if multiple
- Values are auto-parsed as numbers, booleans, or JSON when possible

### export-vars

Export all variables from `project.json` to a CSV file.

```bash
# Export to default file (variables.csv)
acctl export-vars

# Export to a specific file
acctl export-vars --output my_variables.csv
```

The CSV contains columns: `name`, `type`, `direction`, `link`, `description`, `initial`.

### import-vars

Import variables from a CSV file into `project.json`.

```bash
# Import from default file (variables.csv)
acctl import-vars

# Import from a specific file
acctl import-vars --input my_variables.csv
```

During import, variables that already exist are updated and new variables are added. If a variable's link is already used by a different variable in the project, the row is skipped with a warning to prevent duplicate links. The link comparison is case-insensitive.

### dedup-vars

Find and resolve variables in `project.json` that share the same hardware link. When two or more variables point to the same link (e.g., `ethercat.rc8_0.statusword`), only one of them will actually receive updates at runtime — the others are silently overwritten. This command detects those duplicates and lets you choose which to keep.

```bash
acctl dedup-vars
```

Example output:

```
Found 1 duplicate link(s):

Duplicate link: ethercat.rc8_0.statusword
  [1] RC8_0_Inputs_Standard_Statusword  (type: u16, direction: status)
  [2] rc8_0_inputs_standard_statusword  (type: u16, direction: status)
Keep which? [1/2]: 1
  Keeping 'RC8_0_Inputs_Standard_Statusword', removing 'rc8_0_inputs_standard_statusword'

Removed 1 duplicate variable(s).
```

This typically happens when `generate_variables` is run after variables were already created manually or with different naming conventions. The comparison is case-insensitive on the link value.

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

### Deploying a Project to a New Server

If you have an existing project (e.g., developed for one machine) and want to deploy it to a different AutoCore server:

```bash
# 1. Point acctl at the new server
acctl set-target 192.168.1.200

# 2. Check what projects already exist on the server
acctl status

# 3. Create a new project on the server
acctl cmd system.new_project --project_name my_new_project

# 4. Switch the server to the new project
acctl switch my_new_project

# 5. Push your local project.json into it
acctl push project --restart

# 6. Build and deploy the control program
acctl push control --start
```

`system.new_project` creates the standard directory structure on the server (`control/`, `www/`, `datastore/`, `scripts/`) with an empty `project.json`. Once you switch to it and push, your local `project.json` replaces the empty one.

If the project already exists on the server, skip step 3 — just `switch` and `push`.

### Updating Project Configuration

```bash
# Edit project.json locally

# Push changes and restart
acctl push project --restart
```

### Configuring IO-Link Devices with IODD Files

IO-Link devices require IODD (IO Device Description) files for proper configuration. Here's how to set up an IO-Link device on a modular EtherCAT I/O hub:

```bash
# 1. Download the IODD file from the device manufacturer's website
#    (Usually a .xml file or .zip containing the XML)

# 2. Upload the IODD file to the server's lib directory
acctl upload BNI_IOL-302-xxx.xml

# 3. List available slots on your modular device
acctl cmd ethercat.configure --device IMPACT67_0 ListSlots

# 4. Import the IODD to configure a specific slot
acctl cmd ethercat.configure --device IMPACT67_0 ImportIodd --slot 0 --file lib/BNI_IOL-302-xxx.xml

# 5. Verify the configuration
acctl cmd ethercat.configure --device IMPACT67_0 Show

# 6. Push the updated project and restart to apply changes
acctl push project --restart
```

The IODD import automatically:
- Detects the IO-Link device's process data size
- Selects the appropriate IO-Link module type (with smart fallback if exact match unavailable)
- Configures vendor ID, device ID, and communication parameters
- Updates the PDO mappings in project.json

#### Module Selection

The ImportIodd command automatically selects a module based on the device's process data size. If the ideal module isn't available for the slot, it automatically finds the smallest available module that can accommodate the required IO sizes.

You can also manually override the module selection:

```bash
# Override automatic module selection (e.g., use 0x0808 instead of auto-selected 0x0108)
acctl cmd ethercat.configure --device IMPACT67_0 ImportIodd \
  --slot 0 --file lib/device.xml --module 0x0808
```

This is useful when:
- The auto-selected module isn't supported by your specific slot
- You know from another tool (like TwinCAT) which module works best
- You need a larger module than the minimum required for your device

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
