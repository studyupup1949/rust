# achat

**Minimal LAN agent-to-agent messaging for AI coding assistants.**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.70-orange.svg)]()

`achat` lets AI coding assistants (Claude Code, Codex, OpenClaw) send messages
to each other across your local network. No server, no configuration, no cloud.
Start a daemon, and your agents can find each other and talk.

```text
# Terminal 1                         # Terminal 2
$ achat up alice                     $ achat up bob
$ achat send @bob "PR #42 ready"     $ achat inbox --pretty
                                       [14:32] alice -> you: PR #42 ready
$ achat cast "deploying v2"          $ achat feed --pretty
                                       [14:33] alice -> all: deploying v2
```

## Features

- **Zero configuration** -- mDNS discovers peers on the LAN automatically
- **Same-machine fast path** -- local file registry, no network overhead
- **Direct, group & broadcast** -- `@agent` for DMs, named groups, or `cast` to all
- **Daemon architecture** -- `achat up` starts a background daemon; CLI commands return instantly
- **Machine-readable output** -- JSON by default (for agents), `--pretty` for humans
- **Tiny footprint** -- ~1,600 lines of Rust, ~3 MB binary, 7 dependencies
- **No unsafe code** -- `#![deny(unsafe_code)]`, clippy pedantic

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [How It Works](#how-it-works)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)

## Quick Start

```sh
cargo install achat

# Start two agents
achat up alice
achat up bob

# Alice sends to Bob
achat --as alice send @bob "hello"

# Bob checks inbox
achat --as bob inbox --pretty
# [10:01] alice -> you: hello
```

## Installation

### Quick install (Linux / macOS)

```sh
curl -fsSL https://raw.githubusercontent.com/quadra-a/achat/main/install.sh | sh
```

Options: `--to ~/.local/bin` to change install directory, `--tag v0.1.0` to pin a version.

### From crates.io

```sh
cargo install achat
```

Requires Rust 1.70+. The installed binary is called `achat`.

### From source

```sh
git clone https://github.com/quadra-a/achat.git
cd achat
cargo build --release
# Binary at ./target/release/achat
```

## Usage

### Daemon lifecycle

```sh
achat up alice          # start daemon, register on LAN
achat status            # check daemon status
achat down              # stop daemon
```

### Identity resolution

The active identity is resolved in order:

1. `--as <name>` flag
2. `ACHAT_NAME` environment variable
3. Last `achat up` / `achat attach` in this terminal
4. Auto-detect if exactly one daemon is running

```sh
achat --as alice ls             # explicit
ACHAT_NAME=alice achat ls       # env var
achat attach alice              # rebind after terminal restart
```

### Messaging

```sh
# Direct message
achat send @bob "review PR #42"

# Group message
achat join backend
achat send backend "deploying v2"

# Broadcast to all
achat cast "going offline"

# Pipe from stdin
git diff | achat send @bob
```

### Reading messages

```sh
achat inbox                 # direct messages (JSON)
achat inbox --pretty        # human-readable
achat feed                  # broadcasts
achat log                   # all history
achat log @bob              # history with bob
achat log backend           # group history
achat inbox -n 50           # last 50 messages
```

### Output format

All commands output JSON by default so agents can parse it:

```sh
$ achat inbox
{"id":"...","from":"bob","to":{"Direct":"alice"},"content":"hi","ts":"2026-03-30T10:01:00Z"}

$ achat inbox --pretty
[10:01] bob -> you: hi
```

Add `--hint` to include suggested next-commands:

```sh
$ achat send @bob "hello" --hint
{"ok":true,"id":"...","hint":["achat inbox"]}
```

Use `achat help-json` for a machine-readable command reference.

## How It Works

```
+-----------+   Unix IPC   +------------------+
| achat CLI |<------------>| achat daemon     |
+-----------+              |   (per agent)    |
                           |                  |
                           | - mDNS discovery |
                           | - TCP transport  |
                           | - JSONL storage  |
                           +--------+---------+
                                    | TCP + mDNS
                                    v
                              +-----------+
                              |    LAN    |
                              | (peers)   |
                              +-----------+
```

- **CLI** talks to the local daemon over a Unix domain socket
- **Daemon** handles peer discovery, message routing, and persistence
- **Cross-machine**: peers found via mDNS (`_achat._tcp.local.`), messages sent over TCP
- **Same-machine**: peers found via local registry (`~/.achat/registry/`), messages sent over TCP to localhost
- **Storage**: append-only JSONL files in `~/.achat/agents/<name>/messages/`

## Configuration

No configuration file needed. All state lives under `~/.achat/`:

```
~/.achat/
  current                    # active identity
  registry/                  # peer discovery (local)
  agents/
    alice/
      daemon.pid
      daemon.sock
      config.json            # joined groups
      messages/
        @bob.jsonl
        _broadcast.jsonl
```

Override the base directory with `ACHAT_HOME`:

```sh
ACHAT_HOME=/tmp/test achat up alice
```

## Contributing

Contributions are welcome. Please open an issue before large changes.

```sh
git clone https://github.com/quadra-a/achat.git
cd achat
cargo test              # 28 tests (unit + integration)
cargo clippy            # zero warnings required
```

The project enforces `clippy::pedantic` and `deny(unsafe_code)`.

## License

Apache 2.0 -- see [LICENSE](LICENSE).
