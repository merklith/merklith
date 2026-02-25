# MERKLITH Project Bug Report

This document summarizes the bugs, inconsistencies, and issues found in the MERKLITH project during the comprehensive scan.

## 1. Rust Codebase Issues

### 1.1 Broken Benchmarks
The benchmarks in `benches/` are currently failing to compile due to several issues:
- **Unresolved Imports**: Many imports from `merklith-crypto`, `merklith-storage`, and `merklith-consensus` are missing or moved (e.g., `merklith_storage::StateDB` should be `merklith_storage::state_db::StateDB`).
- **Struct Inconsistencies**:
    - `Transaction` struct is missing the `gas_price` field (uses `max_fee_per_gas` instead).
    - `BlockHeader` struct is missing the `logs_bloom` field.
- **Method/Function Signatures**:
    - `Database::open` is used but does not exist; `Database::new` should be used.
    - `Account::new` is called with 2 arguments but takes 0.
    - `vrf_verify` is called with 4 arguments but takes 3.
- **Type Mismatches**: Several `U256` operations in `throughput.rs` have incorrect argument types (passing `&mut Bencher` instead of `&U256`).
- **Missing Functions**: `black_box` is used without being imported (should be `std::hint::black_box` or `criterion::black_box`).

### 1.2 Extensive Warnings
There are over 100 compilation warnings across the workspace, primarily:
- Unused imports and variables.
- Unused fields in structs (e.g., `StateDB::db`, `WasmRuntime::config`).
- Unnecessary parentheses and mutable variables.

## 2. Web Explorer Issues

- **Dependency Conflict**: `npm install` requires `--legacy-peer-deps` because `react-scripts@5.0.1` has a peer dependency conflict with the installed `typescript@^5.0.0`.
- **Build Failure**: `npm run build` fails with `MODULE_NOT_FOUND: Cannot find module 'ajv/dist/compile/codegen'`, likely due to dependency resolution issues.
- **Missing Scripts**: `package.json` is missing a `lint` script, although it is common for such projects.

## 3. Web Wallet Issues

- **Missing Public Directory**: The `public/` directory and `index.html` are missing, causing `react-scripts build` to fail immediately.
- **Security Vulnerabilities**: `npm audit` reports 68 vulnerabilities (53 high, 2 critical).

## 4. TypeScript SDK Issues

- **Missing Dependencies**: `package.json` is missing `devDependencies` (typescript, jest, eslint, prettier) required for the defined scripts.
- **Missing Files**: `src/index.ts` exports `transaction.ts` and `block.ts`, but these files do not exist in the `src/` directory.
- **Build Failure**: `npm run build` fails because `tsc` is not installed/found.

## 5. Configuration Issues

- **Genesis JSON**: `config/genesis.json` contains duplicate keys in the `alloc` object (`merklith1qgp0pltq0r7xz5uq5j9qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq`), which is invalid JSON.

## 6. Scripting Issues

- **Monitor Script**: `monitor.ps1` uses hardcoded PIDs in a `switch` block to identify ports, which is highly unreliable as PIDs change every run.
- **Test Scripts**: `test_transactions.ps1` and others do not check if binaries in `target/release/` exist or attempt to build them before execution.

## 7. Documentation Inconsistencies

- **Storage Layer**: `ARCHITECTURE.md` claims RocksDB is "production-ready," but it is also listed under "Future Improvements." The default implementation remains JSON-based.
- **WASM VM**: The documentation describes a sophisticated VM, but `crates/merklith-vm/src/wasm_runtime.rs` contains a placeholder implementation that always returns success.

## 8. Security Notes

- **Vulnerabilities**: Both web projects have a high number of critical and high-severity vulnerabilities in their dependency trees.
- **Private Keys**: While the keys found in `bot/faucet_bot.py` and `test_merklith.sh` appear to be dummy/testing keys, there is no centralized secret management strategy visible.

## 9. Implementation Gaps (TODOs & FIXMEs)

- **Missing Logic**: Several critical components have `TODO` markers indicating missing implementation:
    - **SDK Macros**: `merklith-sdk-derive` is essentially empty with `TODO: Implement macro` in all files.
    - **Signature Verification**: `erc20.rs` and `bridge.rs` examples contain `TODO: Implement proper signature verification`.
    - **RPC Methods**: `commands.rs` has TODOs for `eth_getBlockByHash`, contract deployment, and transaction sending.
    - **Consensus Logic**: `delegation.rs` has a `TODO` for fixing transitive delegation power calculation.

## 10. Docker & Infrastructure Issues

- **Inappropriate Binary Mounting**: `docker-compose.yml` mounts local Windows/host binaries (`./target/release/merklith-node`) into a Debian-based container. This will fail due to architecture and library differences (DLLs vs Shared Objects).
- **Environment Inefficiency**: Each container runs `apt-get update && apt-get install` every time it starts, which is slow and requires internet access. This should be part of a custom Docker image.
- **Bootnode Logic**: The nodes in `docker-compose.yml` are not configured with a bootnode, meaning they won't automatically discover each other unless manually peered.

## 11. Component Maturity

- **merklith-sdk-derive**: This crate is part of the workspace but is currently non-functional (contains only `TODO` comments).
- **Placeholder Runtimes**: As noted in Documentation Inconsistencies, the WASM runtime is a skeletal placeholder despite being part of the "production-ready" core.
