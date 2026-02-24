# MERKLITH TUI Block Explorer

Interactive terminal-based blockchain explorer for MERKLITH.

## Overview

The MERKLITH TUI (Terminal User Interface) Block Explorer provides a real-time, interactive view of the blockchain without requiring a web browser. It's perfect for:

- Node operators monitoring network activity
- Developers debugging transactions
- Validators tracking blocks and attestations
- Quick blockchain queries without API calls

## Features

### Real-time Monitoring
- Live block updates every 2-6 seconds
- Transaction flow visualization
- Network connection status
- Chain statistics

### Multiple Views
- **Blocks View**: Recent blocks with hashes, timestamps, transaction counts
- **Transactions View**: Pending and confirmed transactions
- **Accounts View**: Top accounts by balance
- **Search View**: Find blocks, transactions, or accounts
- **Block Detail**: Full block information with all transactions

### Interactive Navigation
- Vim-style keybindings (`j/k/h/l`)
- Mouse support (optional)
- Contextual help
- Smooth scrolling

## Launching the Explorer

### Basic Usage

```bash
# Connect to local node
merklith explorer

# Connect to specific node
merklith explorer --rpc http://localhost:8546

# Connect to remote node
merklith explorer --rpc http://your-node:8545
```

### From Source

```bash
cargo run --release -p merklith-cli -- explorer
```

## User Interface

### Layout

```
┌─────────────────────────────────────────────────────────────┐
│ MERKLITH Block Explorer    Chain ID: 1337    Block: 12345   ●  │  ← Header
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Recent Blocks                                                │
│  ┌──────────┬──────────────────────┬──────────┬────────────┐│  ← Main
│  │ Block #  │ Hash                 │ TXs      │ Timestamp  ││     Content
│  ├──────────┼──────────────────────┼──────────┼────────────┤│
│  │ 12345    │ 0xabc123...          │ 5        │ 2s ago     ││
│  │ 12344    │ 0xdef456...          │ 12       │ 8s ago     ││
│  │ 12343    │ 0x789abc...          │ 3        │ 14s ago    ││
│  │ 12342    │ 0x012345...          │ 0        │ 20s ago    ││
│  └──────────┴──────────────────────┴──────────┴────────────┘│
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ q:Quit  h:Help  r:Refresh  b:Blocks  t:TXs  a:Accounts      │  ← Footer
└─────────────────────────────────────────────────────────────┘
```

### Header Section

Displays essential network information:
- **Connection Status**: ● (green = connected, red = disconnected)
- **Chain ID**: Network identifier
- **Latest Block**: Current block number
- **Explorer Version**: Software version

### Main Content Area

Shows different data depending on current view:
- **Table format** for lists (blocks, transactions)
- **Detail view** for single items
- **Search interface** for queries

### Footer Section

Context-sensitive help showing available keyboard shortcuts.

## Keyboard Shortcuts

### Global Navigation

| Key | Action | Description |
|-----|--------|-------------|
| `q` | Quit | Exit the explorer |
| `Esc` | Quit | Alternative exit key |
| `h` | Help | Show help screen |
| `r` | Refresh | Manually refresh data |

### View Switching

| Key | View | Description |
|-----|------|-------------|
| `b` | Blocks | Recent blocks list |
| `t` | Transactions | Recent transactions |
| `a` | Accounts | Account overview |
| `s` | Search | Search interface |

### List Navigation

| Key | Action | Vim Equivalent |
|-----|--------|----------------|
| `↑` | Move up | `k` |
| `↓` | Move down | `j` |
| `←` | Previous page | `h` |
| `→` | Next page | `l` |
| `Enter` | Select item | - |
| `Backspace` | Go back | - |
| `Home` | First item | `gg` |
| `End` | Last item | `G` |
| `PgUp` | Page up | `Ctrl+u` |
| `PgDn` | Page down | `Ctrl+d` |

### Scrolling (Detail Views)

| Key | Action |
|-----|--------|
| `↑/k` | Scroll up |
| `↓/j` | Scroll down |

## Views

### Blocks View (Default)

```
Recent Blocks
┌─────────┬───────────────────┬──────────┬────────────┐
│ Block # │ Hash              │ TXs      │ Proposer   │
├─────────┼───────────────────┼──────────┼────────────┤
│ 12345   │ 0xabc123...       │ 5        │ 0xval...   │
│ 12344   │ 0xdef456...       │ 12       │ 0xval...   │
│ 12343   │ 0x789abc...       │ 3        │ 0xval...   │
└─────────┴───────────────────┴──────────┴────────────┘
```

Columns:
- **Block #**: Block height
- **Hash**: Block hash (truncated)
- **TXs**: Number of transactions
- **Proposer**: Validator that proposed the block
- **Timestamp**: Time since block

Press `Enter` on a block to view details.

### Block Detail View

```
Block Details
═══════════════════════════════════════════════════════════
Block Number: 12345
Hash: 0xabc123def4567890123456789012345678901234
Timestamp: 2024-01-15 14:30:25 UTC (2s ago)
Proposer: merklith1qxy2kgcygj5xvk8f3s9w4...
Transactions: 5
Gas Used: 105,000 / 30,000,000 (0.35%)
Parent Hash: 0xdef456abc7890123456789012345678901234567
State Root: 0x0123456789abcdef0123456789abcdef01234567
Transactions:
  1. 0x1234... → merklith1recip... : 1.5 ANV
  2. 0x5678... → merklith1recip... : 0.1 ANV
  ...
```

Use `↑/↓` to scroll through transaction list.

### Transactions View

```
Recent Transactions
┌───────────────────┬───────────────────┬───────────────────┬───────────────┬────────┐
│ Hash              │ From              │ To                │ Value (ANV)   │ Nonce  │
├───────────────────┼───────────────────┼───────────────────┼───────────────┼────────┤
│ 0x1234...         │ 0xabc1...         │ 0xdef2...         │ 1.500000      │ 42     │
│ 0x5678...         │ 0xghi3...         │ Contract Creation │ 0.000000      │ 15     │
│ 0x9abc...         │ 0xjkl4...         │ 0xmno5...         │ 100.000000    │ 7      │
└───────────────────┴───────────────────┴───────────────────┴───────────────┴────────┘
```

Shows the most recent transactions across all blocks.

### Accounts View

```
Top Accounts
┌───────────────────────────────┬─────────────────────┬─────────────┐
│ Address                       │ Balance (ANV)       │ Nonce       │
├───────────────────────────────┼─────────────────────┼─────────────┤
│ merklith1qxy2kgcygj5xvk8f3...    │ 1,000,000.000000    │ 42          │
│ merklith1abc2def3ghi4jkl5...     │ 500,000.000000      │ 12          │
│ merklith1xyz2uvw3rst4opq6...     │ 250,000.000000      │ 8           │
└───────────────────────────────┴─────────────────────┴─────────────┘
```

### Search View

```
Search
═══════════════════════════════════════════════════════════

Enter block number, hash, or address:
> 12345

[Search Button]

Recent Searches:
  • 0xabc123...
  • merklith1qxy2...
  • 12344
```

Search supports:
- Block numbers (e.g., `12345`)
- Block hashes (e.g., `0xabc123...`)
- Transaction hashes (e.g., `0xdef456...`)
- Account addresses (e.g., `merklith1qxy2...`)

### Help View

```
Help - Keyboard Shortcuts
═══════════════════════════════════════════════════════════

Navigation
  ↑ / k    - Move up
  ↓ / j    - Move down
  ← / h    - Previous page
  → / l    - Next page
  Enter    - Select item
  Backspace- Go back

Views
  b        - Blocks view
  t        - Transactions view
  a        - Accounts view
  s        - Search

Actions
  r        - Refresh data
  h        - Show this help
  q / Esc  - Quit
```

## Configuration

### Environment Variables

```bash
# Set default RPC endpoint
export MERKLITH_RPC_URL="http://localhost:8545"

# Run explorer
merklith explorer
```

### Config File

Edit `~/.merklith/config.toml`:

```toml
rpc_url = "http://localhost:8545"
refresh_interval = 5000  # milliseconds
```

## Customization

### Color Schemes

The explorer supports different color themes via environment variables:

```bash
# Dark theme (default)
MERKLITH_THEME=dark merklith explorer

# Light theme
MERKLITH_THEME=light merklith explorer
```

### Mouse Support

Enable mouse support for clicking on items:

```bash
# Enable mouse
merklith explorer --mouse
```

Note: Mouse support may vary by terminal emulator.

## Performance

### Data Refresh

- **Automatic refresh**: Every 2-5 seconds (configurable)
- **Manual refresh**: Press `r` anytime
- **Smart updates**: Only refreshes visible data

### Resource Usage

- **Memory**: ~50-100 MB
- **CPU**: Minimal (< 1% on modern systems)
- **Network**: ~1-2 KB/s for updates

### Large Networks

For mainnet-scale networks with thousands of TPS:

```bash
# Increase refresh interval to reduce load
merklith explorer --refresh-interval 10000  # 10 seconds
```

## Troubleshooting

### Connection Issues

**Problem**: Red dot (●) in header, no data loading

**Solutions**:
```bash
# Check if node is running
curl http://localhost:8545 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# Try different RPC endpoint
merklith explorer --rpc http://remote.node:8545

# Check firewall/network
ping localhost
telnet localhost 8545
```

### Display Issues

**Problem**: Garbled characters, layout broken

**Solutions**:
1. Use a modern terminal (iTerm2, Windows Terminal, Alacritty, Kitty)
2. Enable Unicode support
3. Set terminal to 256 colors minimum:
   ```bash
   export TERM=xterm-256color
   ```

### Performance Issues

**Problem**: Slow updates, laggy interface

**Solutions**:
1. Reduce refresh rate:
   ```bash
   merklith explorer --refresh-interval 10000
   ```
2. Disable animations:
   ```bash
   merklith explorer --no-animations
   ```
3. Close other terminal applications

### Key Binding Conflicts

**Problem**: Some keys don't work (e.g., in tmux)

**Solutions**:
```bash
# Use alternative keybindings
merklith explorer --keybindings vi

# Or disable conflicting shortcuts
merklith explorer --no-vim-keys
```

## Advanced Usage

### Multiple Explorers

Run multiple explorers for different networks:

```bash
# Terminal 1: Local dev
merklith explorer --rpc http://localhost:8545

# Terminal 2: Remote node
merklith explorer --rpc http://your-node:8545

# Terminal 3: Another node
merklith explorer --rpc http://another-node:8545
```

### Scripting

Capture explorer output for scripts:

```bash
# Not directly supported, but you can use CLI:
merklith query block latest
merklith account balance merklith1address...
```

### Remote Monitoring

Access explorer over SSH:

```bash
# On server
merklith explorer --bind 0.0.0.0:8080

# On local machine
ssh -L 8080:localhost:8080 user@server
# Then open browser: http://localhost:8080
```

## Tips and Tricks

### 1. Quick Block Inspection

```bash
# Instead of:
merklith query block 12345

# Use explorer:
merklith explorer
# Press 'b', navigate to block, press Enter
```

### 2. Monitor Your Transactions

```bash
# Terminal 1: Send transaction
merklith tx send merklith1recipient... 1.5

# Terminal 2: Watch for confirmation
merklith explorer
# Switch to 't' (transactions) view
```

### 3. Validator Monitoring

```bash
# Run explorer alongside your validator
merklith-node --validator &
merklith explorer

# Watch blocks you're producing
# Check your validator address in proposer column
```

### 4. Network Health Check

```bash
# Check connection status in header
# If red dot (●), node is down
# If green dot (●), all good
```

### 5. Finding Specific Transactions

```bash
# Use search 's'
# Enter tx hash or address
# Navigate with arrows
```

## Comparison with Web Explorer

| Feature | TUI Explorer | Web Explorer |
|---------|--------------|--------------|
| Resource Usage | Low | High |
| Setup | None | Browser + Server |
| Speed | Instant | Network dependent |
| Offline | Partial (cached) | No |
| Mobile | No | Yes |
| Visual Charts | No | Yes |
| Accessibility | Terminal only | Universal |

## Integration

### With tmux

```bash
# Create tmux session with explorer
tmux new-session -d -s merklith-explorer 'merklith explorer'
tmux attach -t merklith-explorer

# Or split window
tmux split-window -h 'merklith explorer'
tmux split-window -v 'merklith-node --validator'
```

### With Terminal Multiplexers

**Screen**:
```bash
screen -S explorer
merklith explorer
# Detach: Ctrl+A, D
# Reattach: screen -r explorer
```

**Zellij**:
```bash
zellij --layout explorer.kdl
```

### With systemd

Create `/etc/systemd/system/merklith-explorer@.service`:

```ini
[Unit]
Description=MERKLITH Block Explorer for %I
After=network.target

[Service]
Type=simple
User=%i
ExecStart=/usr/local/bin/merklith explorer --rpc http://localhost:8545
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable:
```bash
sudo systemctl enable merklith-explorer@$USER
sudo systemctl start merklith-explorer@$USER
```

## Development

### Building from Source

```bash
cd merklith/crates/merklith-cli
cargo build --release
./target/release/merklith explorer
```

### Running in Debug Mode

```bash
RUST_LOG=debug cargo run -- explorer
```

### Custom Themes

Edit `src/explorer/ui.rs`:

```rust
// Change colors
const PRIMARY_COLOR: Color = Color::Cyan;
const SECONDARY_COLOR: Color = Color::Yellow;
const ERROR_COLOR: Color = Color::Red;
```

## See Also

- [CLI Guide](CLI_GUIDE.md) - Complete CLI documentation
- [API Documentation](API.md) - JSON-RPC methods
- [Architecture](ARCHITECTURE.md) - System design