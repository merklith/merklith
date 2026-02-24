//! Merklith VM - WASM-based virtual machine for smart contract execution.
//!
//! This crate provides:
//! - WASM runtime using wasmtime
//! - Gas metering and fuel tracking
//! - Host function API
//! - Reentrancy protection
//! - Precompiled contracts

pub mod error;
pub mod gas_metering;
pub mod runtime;
pub mod reentrancy;
pub mod wasm_runtime;
pub mod merkle_trie;

pub use error::VmError;
pub use gas_metering::{GasSchedule, GasTracker};
pub use runtime::{MerklithVM, ExecutionContext, ExecutionResult};
pub use reentrancy::ReentrancyGuard;
pub use wasm_runtime::{WasmRuntime, WasmRuntimeConfig, HostState, LogEntry};
pub use merkle_trie::{MerkleTrie, StateManager, TrieNode};

/// VM version constant
pub const VM_VERSION: u32 = 1;

/// Maximum WASM memory per contract (16 MB)
pub const MAX_MEMORY_BYTES: usize = 16 * 1024 * 1024;

/// Maximum call depth
pub const MAX_CALL_DEPTH: usize = 64;

/// Maximum contract code size (128 KB)
pub const MAX_CODE_SIZE: usize = 128 * 1024;

/// Maximum stack size (1024 items)
pub const MAX_STACK_SIZE: usize = 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_constants() {
        assert_eq!(VM_VERSION, 1);
        assert_eq!(MAX_MEMORY_BYTES, 16 * 1024 * 1024);
        assert_eq!(MAX_CALL_DEPTH, 64);
    }
}
