# Aang - Kora Rent Reclaim TUI

A terminal user interface (TUI) and CLI tool for Kora operators to monitor and reclaim rent-locked SOL on Solana.

```
    _
   / \   __ _ _ __   __ _
  / _ \ / _` | '_ \ / _` |
 / ___ \ (_| | | | | (_| |
/_/   \_\__,_|_| |_|\__, |
                    |___/
  Kora Rent Reclaim Bot
```

## What is Kora?

[Kora](https://github.com/solana-foundation/kora) is a fee relayer and signing node built by the Solana Foundation. It enables gasless transactions on Solana, allowing users to pay transaction fees in SPL tokens (like USDC) instead of SOL.

### How Kora Works

1. **User initiates transaction** - User wants to interact with a dApp
2. **App constructs transaction** - Includes payment instruction to Kora operator in SPL tokens
3. **User signs** - Signs the transaction with their wallet
4. **Kora validates** - Checks rules, token payment, and security policies
5. **Kora co-signs as fee payer** - Pays the SOL transaction fees
6. **Transaction submitted** - App sends to Solana
7. **Settlement** - User pays tokens, operator pays SOL fees

### The Rent Problem

When Kora sponsors transactions that create new accounts, SOL is locked as **rent** - a deposit required by Solana to keep accounts active. This rent is approximately:

- **Basic account**: ~0.00089 SOL (890,880 lamports)
- **Token account**: ~0.00204 SOL (2,039,280 lamports)

Over time, many sponsored accounts become:
- **Inactive** - No transactions for extended periods
- **Empty** - Token accounts with zero balance
- **Closed** - Closed by users/programs but rent not reclaimed

**Without active monitoring, this rent SOL silently accumulates as locked capital.**

## Features

Aang solves this by providing:

### Dashboard
- Real-time rent statistics
- Total locked vs reclaimed SOL
- Account status distribution
- Configuration overview

### Account Monitoring
- Track sponsored accounts
- Detect status changes (active, inactive, empty, closed)
- Filter by program/owner
- Whitelist critical accounts

### Rent Reclaim
- Automatic detection of reclaimable accounts
- Safe reclaim with confirmation prompts
- Dry-run mode for testing
- Transaction audit trail

### Alerts & Logs
- Alerts for large idle rent
- Activity logs with reasons
- Export/import functionality
- JSON output for automation

### CLI Commands
```bash
# Start TUI
aang

# Scan accounts
aang scan --accounts <pubkeys> --json

# Reclaim rent
aang reclaim --all --dry-run

# View statistics
aang stats --json

# Initialize config
aang init

# Export/Import
aang export -o accounts.json
aang import accounts.json
```

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/superteamng/aang.git
cd aang

# Build
cargo build --release

# Install
cargo install --path .
```

### Dependencies

- Rust 1.75+
- Solana CLI (optional, for testing)

## Configuration

Generate a config file:

```bash
aang init -o aang.toml
```

Edit `aang.toml`:

```toml
[network]
rpc_url = "https://api.devnet.solana.com"
commitment = "confirmed"

[operator]
# Your Kora node's fee payer public key
fee_payer = "YourKoraFeePayer..."
# Treasury address for reclaimed rent
treasury = "YourTreasuryAddress..."
# Path to keypair file
keypair_path = "/path/to/keypair.json"
# Start in dry-run mode (recommended for testing)
dry_run = true

[monitoring]
scan_interval_secs = 300  # 5 minutes
auto_scan = false
inactivity_threshold_days = 30

[alerts]
enabled = true
large_idle_threshold_sol = 1.0
```

## Usage

### TUI Mode

```bash
# Start with default config
aang

# Specify config file
aang -c custom-config.toml

# Use different RPC
aang --rpc-url https://api.mainnet-beta.solana.com
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Switch tabs |
| `1-5` | Jump to tab |
| `s/S` | Scan accounts |
| `q/Esc` | Quit |
| `:` | Command mode |

#### Accounts Tab
| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down |
| `a` | Add account |
| `d` | Delete account |
| `r` | Reclaim selected |
| `Enter` | Confirm reclaim |

#### Logs Tab
| Key | Action |
|-----|--------|
| `j/k` | Scroll |
| `c` | Clear logs |

#### Alerts Tab
| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `Enter` | Acknowledge |
| `c` | Clear acknowledged |

### CLI Mode

```bash
# Scan specific accounts
aang scan -a "Pubkey1,Pubkey2,Pubkey3"

# Scan from file
aang scan -f accounts.txt

# Reclaim all eligible (dry-run first!)
aang reclaim --all --dry-run

# Reclaim for real
aang reclaim --all

# Get stats as JSON
aang stats --json
```

## Understanding Solana Rent

### What is Rent?

Solana charges rent to store data on-chain. Accounts must maintain a minimum balance (rent-exempt threshold) to persist.

### Rent Calculation

```
rent_exempt_minimum = (account_data_size + 128) * 3,480 lamports/byte-year * 2 years
```

For a basic account (0 bytes data):
```
128 * 3,480 * 2 = 890,880 lamports (~0.00089 SOL)
```

### When Can Rent Be Reclaimed?

1. **Token Accounts**: Close using `spl-token close`
2. **Program Accounts**: Via program's close instruction
3. **System Accounts**: Transfer all lamports out

### Safety Considerations

- **Never reclaim active accounts** - Check for recent transactions
- **Whitelist critical accounts** - Prevent accidental closure
- **Use dry-run mode** - Test before real reclaims
- **Verify treasury address** - Ensure funds go to correct destination

## Architecture

```
aang/
├── src/
│   ├── main.rs          # Entry point, CLI parsing
│   ├── app.rs           # Application state
│   ├── config.rs        # Configuration management
│   ├── types.rs         # Core data types
│   ├── solana/
│   │   ├── mod.rs
│   │   ├── client.rs    # Solana RPC client
│   │   ├── monitor.rs   # Account monitoring
│   │   └── reclaim.rs   # Rent reclaim logic
│   └── ui/
│       ├── mod.rs       # UI renderer
│       ├── dashboard.rs # Dashboard tab
│       ├── accounts.rs  # Accounts tab
│       ├── logs.rs      # Logs tab
│       ├── alerts.rs    # Alerts tab
│       ├── settings.rs  # Settings tab
│       └── widgets.rs   # UI helpers
├── Cargo.toml
└── README.md
```

## Devnet Testing

```bash
# 1. Generate a test config
aang init -o test-config.toml

# 2. Edit to use devnet
# rpc_url = "https://api.devnet.solana.com"
# dry_run = true

# 3. Add some devnet accounts to monitor
aang -c test-config.toml add "DevnetPubkey1,DevnetPubkey2"

# 4. Start TUI
aang -c test-config.toml
```

## Roadmap

- [ ] WebSocket subscriptions for real-time updates
- [ ] Telegram/Discord alerts
- [ ] Historical rent analytics
- [ ] Multi-treasury support
- [ ] Batch reclaim optimization
- [ ] Program-specific reclaim handlers

## Contributing

Contributions welcome! Please read our contributing guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Resources

- [Kora Documentation](https://launch.solana.com/docs/kora/operators)
- [Solana Rent Model](https://solana.com/docs/core/accounts)
- [Solana JSON RPC API](https://solana.com/docs/rpc)
- [Ratatui TUI Framework](https://ratatui.rs/)

## Credits

Built for the Kora Rent Reclaim Bounty by SuperteamNG.
