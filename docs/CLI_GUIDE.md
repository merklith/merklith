# MERKLITH CLI Guide

Complete guide for using the MERKLITH command-line interface.

## Table of Contents

- [Installation](#installation)
- [Overview](#overview)
- [Global Options](#global-options)
- [Wallet Commands](#wallet-commands)
- [Account Commands](#account-commands)
- [Transaction Commands](#transaction-commands)
- [Query Commands](#query-commands)
- [Contract Commands](#contract-commands)
- [Explorer Command](#explorer-command)
- [Configuration](#configuration)
- [Examples](#examples)

## Installation

The CLI is included when you build MERKLITH from source:

```bash
cargo build --release
./target/release/merklith --help
```

Or install directly:

```bash
cargo install --path ./crates/merklith-cli
```

## Overview

```
merklith [OPTIONS] <COMMAND>
```

The MERKLITH CLI provides a complete set of tools for:
- Wallet management and key storage
- Account queries and balance checks
- Transaction creation and signing
- Blockchain exploration
- Smart contract interaction
- Node management

## Global Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--rpc` | `-r` | RPC endpoint URL | `http://localhost:8545` |
| `--chain-id` | | Chain ID | 1 |
| `--verbose` | `-v` | Enable verbose output | false |

Example:
```bash
merklith --rpc http://remote.node:8545 --chain-id 1337 wallet list
```

## Wallet Commands

### Create a New Wallet

```bash
merklith wallet create [OPTIONS]
```

Create a new encrypted wallet with a password-protected keystore.

Options:
- `--name, -n`: Wallet name (optional, will prompt if not provided)

Example:
```bash
# Interactive creation
merklith wallet create

# With name
merklith wallet create --name "My Validator Wallet"
```

Output:
```
Wallet name: My Validator Wallet
Set password: ********
Confirm password: ********
✓ Created wallet 'My Validator Wallet'
Address: merklith1qxy2kgcygj5xv...

IMPORTANT: Save your recovery phrase in a safe place!
Your wallet is encrypted with AES-256-GCM and stored in the keystore.
```

### List Wallets

```bash
merklith wallet list
```

Display all wallets in the keystore with their addresses and default status.

Example:
```bash
merklith wallet list
```

Output:
```
Wallets:
  • My Validator Wallet - merklith1qxy2kgcygj5xv... (default)
  • Backup Wallet - merklith1abc2def3ghi...
  • Test Wallet - merklith1xyz2uvw3rst...

Total: 3 wallet(s)
```

### Import Wallet

```bash
merklith wallet import <PRIVATE_KEY> [OPTIONS]
```

Import a wallet from a private key.

Arguments:
- `PRIVATE_KEY`: Hex-encoded private key (with or without 0x prefix)

Options:
- `--name, -n`: Wallet name

Example:
```bash
merklith wallet import 0x1234567890abcdef... --name "Imported Wallet"
```

### Export Wallet

```bash
merklith wallet export <ADDRESS>
```

⚠️ **WARNING**: This exposes the private key! Use with extreme caution.

Arguments:
- `ADDRESS`: Wallet address to export

Example:
```bash
merklith wallet export merklith1qxy2kgcygj5xv...

WARNING: This will expose the private key for merklith1qxy2kgcygj5xv... Continue? [y/N]: y
Enter wallet password: ********

⚠️  NEVER share your private key with anyone!

Private Key: 0x1234567890abcdef...
Store this in a secure, offline location.
```

### Remove Wallet

```bash
merklith wallet remove <ADDRESS>
```

Permanently delete a wallet from the keystore.

Arguments:
- `ADDRESS`: Wallet address to remove

Example:
```bash
merklith wallet remove merklith1qxy2kgcygj5xv...

⚠️  Remove wallet merklith1qxy2kgcygj5xv...? This cannot be undone! [y/N]: y
Enter wallet password to confirm removal: ********
✓ Removed wallet merklith1qxy2kgcygj5xv...
```

## Account Commands

### Check Balance

```bash
merklith account balance [ADDRESS]
```

Query the balance of an address. If no address provided, uses the default wallet.

Arguments:
- `ADDRESS`: (Optional) Address to query

Example:
```bash
# Query specific address
merklith account balance merklith1qxy2kgcygj5xv...

# Query default wallet
merklith account balance
```

Output:
```
Address: merklith1qxy2kgcygj5xv...
Balance: 1,234.567890 ANV
```

### List All Account Balances

```bash
merklith account balances
```

Display balances for all wallets in the keystore.

Example:
```bash
merklith account balances
```

Output:
```
Account Balances:
============================================================
  My Validator Wallet [default]
    Address: merklith1qxy2kgcygj5xv...
    Balance: 1,234.567890 ANV

  Backup Wallet
    Address: merklith1abc2def3ghi...
    Balance: 500.000000 ANV

============================================================
Total Balance: 1,734.567890 ANV
```

### Get Transaction Count (Nonce)

```bash
merklith account nonce <ADDRESS>
```

Query the nonce (transaction count) for an address.

Example:
```bash
merklith account nonce merklith1qxy2kgcygj5xv...

Address: merklith1qxy2kgcygj5xv...
Nonce:   42
```

## Transaction Commands

### Send Transaction

```bash
merklith tx send <TO> <AMOUNT> [OPTIONS]
```

Send ANV tokens to an address. The transaction is signed locally with your wallet.

Arguments:
- `TO`: Recipient address
- `AMOUNT`: Amount in ANV (e.g., 1.5, 100, 0.001)

Options:
- `--from, -f`: Sender address (uses default if not specified)
- `--gas-price, -g`: Gas price in wei (optional)
- `--gas-limit, -l`: Gas limit (default: 21000)

Example:
```bash
# Send from default wallet
merklith tx send merklith1recipient... 1.5

# Send from specific wallet
merklith tx send merklith1recipient... 100 --from merklith1sender...

# With custom gas
merklith tx send merklith1recipient... 50 --gas-price 2000000000 --gas-limit 50000
```

Output:
```
Sending 1.500000 ANV to merklith1recipient...
From: merklith1sender...
Gas Price: 1 Gwei
Gas Limit: 21000
Enter wallet password to sign transaction: ********
✓ Transaction sent successfully!
Transaction Hash: 0xabc123...

View transaction:
  merklith tx get 0xabc123...
```

### Get Transaction Details

```bash
merklith tx get <HASH>
```

Retrieve details of a transaction by its hash.

Example:
```bash
merklith tx get 0xabc123...
```

Output:
```
Transaction: 0xabc123...
Status: Confirmed (Block #12345)
From: merklith1sender...
To: merklith1recipient...
Value: 1.500000 ANV
Gas Used: 21,000
Gas Price: 1 Gwei
Fee: 0.000021 ANV
Nonce: 42
Timestamp: 2024-01-15 14:30:25 UTC
```

### Wait for Transaction Confirmation

```bash
merklith tx wait <HASH> [OPTIONS]
```

Wait for a transaction to be confirmed, with optional timeout.

Options:
- `--timeout, -t`: Timeout in seconds (default: 60)

Example:
```bash
merklith tx wait 0xabc123... --timeout 120
```

Output:
```
Waiting for confirmation...
[████████████████████] 100%

Confirmed!
Block: #12345
Gas Used: 21,000
Status: Success
```

## Query Commands

### Get Block by Number

```bash
merklith query block <NUMBER>
```

Query a block by its number. Use "latest" for the most recent block.

Example:
```bash
# Get specific block
merklith query block 12345

# Get latest block
merklith query block latest
```

Output:
```
Block #12345
Hash: 0xdef456...
Timestamp: 2024-01-15 14:30:25 UTC
Transactions: 5
Proposer: merklith1validator...
Gas Used: 105,000 / 30,000,000
Parent Hash: 0x789abc...
State Root: 0x012345...
```

### Get Block by Hash

```bash
merklith query block-hash <HASH>
```

Query a block by its hash.

Example:
```bash
merklith query block-hash 0xdef456...
```

### Get Current Block Number

```bash
merklith query block-number
```

Display the current block number.

Example:
```bash
merklith query block-number

Current block number: 12345
```

### Get Chain ID

```bash
merklith query chain-id
```

Display the chain ID.

Example:
```bash
merklith query chain-id

Chain ID: 1337
```

### Get Gas Price

```bash
merklith query gas-price
```

Display the current gas price.

Example:
```bash
merklith query gas-price

Gas Price: 0.000000001 ANV (1 Gwei / 1000000000 wei)
```

### Get Node Info

```bash
merklith query node-info
```

Display information about the connected node.

Example:
```bash
merklith query node-info

Node Information
==================================================
{
  "version": "0.1.0",
  "chain_id": 1337,
  "syncing": false,
  "peers": 8,
  "latest_block": 12345,
  "validator": true
}
```

## Contract Commands

### Deploy Contract

```bash
merklith contract deploy <BYTECODE_FILE> [OPTIONS]
```

Deploy a smart contract from bytecode.

Arguments:
- `BYTECODE_FILE`: Path to bytecode file (.bin)

Options:
- `--args, -a`: Constructor arguments (hex)
- `--gas-limit, -l`: Gas limit (default: 1,000,000)

Example:
```bash
merklith contract deploy ./MyContract.bin --gas-limit 2000000
```

### Call Contract (Read-only)

```bash
merklith contract call <ADDRESS> <DATA>
```

Call a contract function without creating a transaction (read-only).

Arguments:
- `ADDRESS`: Contract address
- `DATA`: Function call data (hex)

Example:
```bash
merklith contract call merklith1contract... 0x70a08231000000000000000000000000...

Result: 0x0000000000000000000000000000000000000000000000056bc75e2d63100000
```

### Send Transaction to Contract

```bash
merklith contract send <ADDRESS> <DATA> [OPTIONS]
```

Send a transaction to a contract function.

Arguments:
- `ADDRESS`: Contract address
- `DATA`: Function call data (hex)

Options:
- `--value, -v`: Value to send in ANV (default: 0)
- `--gas-limit, -l`: Gas limit

Example:
```bash
merklith contract send merklith1contract... 0xa9059cbb... --value 0.1 --gas-limit 100000
```

### Get Contract Code

```bash
merklith contract code <ADDRESS>
```

Retrieve the bytecode of a contract.

Example:
```bash
merklith contract code merklith1contract...

Code at merklith1contract...: 2456 bytes
0x608060405234801561001057600080fd5b50...
```

## Explorer Command

### Launch TUI Block Explorer

```bash
merklith explorer [OPTIONS]
```

Launch the interactive terminal-based block explorer.

Options:
- `--rpc`: RPC endpoint URL

Example:
```bash
# Connect to local node
merklith explorer

# Connect to remote node
merklith explorer --rpc http://remote.node:8545
```

Once launched, use these keyboard shortcuts:

| Key | Action |
|-----|--------|
| `b` | View blocks list |
| `t` | View transactions |
| `a` | View accounts |
| `s` | Search |
| `h` | Show help |
| `r` | Refresh data |
| `↑/k` | Move up |
| `↓/j` | Move down |
| `←/h` | Previous page |
| `→/l` | Next page |
| `Enter` | Select item |
| `Backspace` | Go back |
| `q/Esc` | Quit |

## Configuration

The CLI stores configuration in `~/.merklith/config.toml`:

```toml
rpc_url = "http://localhost:8545"
chain_id = 1337
gas_price = 1000000000
gas_limit = 100000
keystore_dir = "/home/user/.merklith/keystore"
default_account = "merklith1qxy2kgcygj5xv..."
```

### Config Commands

```bash
# Show current configuration
merklith config show

# Set a configuration value
merklith config set rpc "http://remote.node:8545"
merklith config set chain_id 17001

# Get a specific value
merklith config get rpc

# Reset to defaults
merklith config reset
```

## Examples

### Complete Workflow

```bash
# 1. Create a new wallet
merklith wallet create --name "My Wallet"

# 2. Check balance
merklith account balance

# 3. Send some ANV
merklith tx send merklith1recipient... 10.5

# 4. Check transaction
merklith tx wait 0xabc123...

# 5. View in explorer
merklith explorer
```

### Validator Setup

```bash
# Create validator wallet
merklith wallet create --name "Validator"

# Check you have enough balance
merklith account balance merklith1validator...

# Start validator node
merklith-node --chain-id 1337 --validator

# Monitor blocks
merklith explorer
```

### Multi-Account Management

```bash
# Create multiple wallets
merklith wallet create --name "Savings"
merklith wallet create --name "Trading"
merklith wallet create --name "Gas"

# Check all balances
merklith account balances

# Set default wallet
merklith wallet use merklith1savings...  # Makes this the default

# Send from specific wallet
merklith tx send merklith1recipient... 100 --from merklith1trading...
```

### Contract Development

```bash
# Deploy contract
merklith contract deploy ./Token.bin --gas-limit 2000000

# Interact with contract
merklith contract call merklith1token... 0x70a08231...  # balanceOf
merklith contract send merklith1token... 0xa9059cbb... --value 0  # transfer

# Check contract code
merklith contract code merklith1token...
```

### Automation Scripts

```bash
#!/bin/bash
# daily-check.sh

echo "=== Daily Balance Check ==="
merklith account balances

echo ""
echo "=== Recent Transactions ==="
for tx_hash in $(cat tx-history.txt); do
    merklith tx get $tx_hash
done

echo ""
echo "=== Latest Block ==="
merklith query block latest
```

## Troubleshooting

### Connection Issues

```bash
# Test RPC connection
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# Check if node is running
merklith query node-info

# Use different RPC endpoint
merklith --rpc http://backup.node:8545 account balance
```

### Wallet Issues

```bash
# List wallets to verify address
merklith wallet list

# Check keystore location
merklith config show | grep keystore

# Verify wallet password
merklith wallet export merklith1address...
```

### Transaction Issues

```bash
# Check transaction status
merklith tx get 0xhash...

# Check nonce
merklith account nonce merklith1address...

# Wait for confirmation
merklith tx wait 0xhash... --timeout 300
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `MERKLITH_RPC_URL` | Default RPC endpoint | `http://localhost:8545` |
| `MERKLITH_CHAIN_ID` | Default chain ID | `1337` |
| `MERKLITH_KEYSTORE_DIR` | Custom keystore location | `/secure/keystore` |
| `RUST_LOG` | Logging level | `debug`, `info`, `warn` |

Example:
```bash
export MERKLITH_RPC_URL="http://remote.node:8545"
merklith account balance  # Uses remote node
```

## Tips and Best Practices

1. **Always use --verbose for debugging**: `merklith -v tx send ...`
2. **Set default wallet**: Use `merklith wallet use` to avoid typing address repeatedly
3. **Check gas prices**: Run `merklith query gas-price` before transactions
4. **Monitor transactions**: Use `merklith tx wait` for important transactions
5. **Backup keystore**: Regularly backup `~/.merklith/keystore/` directory
6. **Use explorer**: The TUI explorer is great for monitoring network activity

## See Also

- [API Documentation](API.md) - JSON-RPC methods
- [Explorer Guide](EXPLORER.md) - TUI block explorer
- [Architecture](ARCHITECTURE.md) - System design