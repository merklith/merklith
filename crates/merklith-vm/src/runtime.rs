//! WASM runtime for smart contract execution.
//!
//! Provides a safe, metered execution environment using wasmtime.

use wasmtime::{Config, Engine};
use bytes::Bytes;

use merklith_types::{Address, U256};
use crate::error::VmError;
use crate::gas_metering::{GasSchedule, GasTracker};
#[allow(unused_imports)]
use crate::reentrancy::ReentrancyGuard;
use crate::{MAX_CODE_SIZE, MAX_STACK_SIZE};

/// Execution context for a contract call.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Address of the contract being called
    pub contract_address: Address,
    /// Address of the caller (may be user or contract)
    pub caller: Address,
    /// Transaction origin (EOA that started the transaction)
    pub origin: Address,
    /// Value sent with the call
    pub value: U256,
    /// Gas limit for this call
    pub gas_limit: u64,
    /// Gas price
    pub gas_price: U256,
    /// Block number
    pub block_number: u64,
    /// Block timestamp
    pub block_timestamp: u64,
    /// Block hash
    pub block_hash: [u8; 32],
    /// Chain ID
    pub chain_id: u64,
    /// Whether this is a static call (no state changes)
    pub is_static: bool,
    /// Input data
    pub input: Bytes,
    /// Contract code
    pub code: Bytes,
    /// Code hash
    pub code_hash: [u8; 32],
}

impl ExecutionContext {
    /// Create a new execution context for a contract call.
    pub fn new_call(
        contract_address: Address,
        caller: Address,
        origin: Address,
        gas_limit: u64,
        input: Bytes,
    ) -> Self {
        Self {
            contract_address,
            caller,
            origin,
            value: U256::ZERO,
            gas_limit,
            gas_price: U256::ZERO,
            block_number: 0,
            block_timestamp: 0,
            block_hash: [0u8; 32],
            chain_id: 0,
            is_static: false,
            input,
            code: Bytes::new(),
            code_hash: [0u8; 32],
        }
    }

    /// Create a new execution context for contract creation.
    pub fn new_create(
        caller: Address,
        origin: Address,
        gas_limit: u64,
        code: Bytes,
    ) -> Result<Self, VmError> {
        if code.len() > MAX_CODE_SIZE {
            return Err(VmError::CodeSizeExceeded {
                size: code.len(),
                limit: MAX_CODE_SIZE,
            });
        }

        let hash = blake3::hash(&code);
        let mut code_hash = [0u8; 32];
        code_hash.copy_from_slice(hash.as_bytes());
        
        Ok(Self {
            contract_address: Address::ZERO, // Will be computed
            caller,
            origin,
            value: U256::ZERO,
            gas_limit,
            gas_price: U256::ZERO,
            block_number: 0,
            block_timestamp: 0,
            block_hash: [0u8; 32],
            chain_id: 0,
            is_static: false,
            input: Bytes::new(),
            code,
            code_hash,
        })
    }

    /// Set gas price.
    pub fn with_gas_price(mut self, gas_price: U256) -> Self {
        self.gas_price = gas_price;
        self
    }

    /// Set block info.
    pub fn with_block_info(
        mut self,
        number: u64,
        timestamp: u64,
        hash: [u8; 32],
    ) -> Self {
        self.block_number = number;
        self.block_timestamp = timestamp;
        self.block_hash = hash;
        self
    }

    /// Set value.
    pub fn with_value(mut self, value: U256) -> Self {
        self.value = value;
        self
    }

    /// Set as static call.
    pub fn as_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    /// Set chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }
}

/// Result of contract execution.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Return data
    pub data: Bytes,
    /// Gas used
    pub gas_used: u64,
    /// Gas refunded
    pub gas_refunded: u64,
    /// Logs emitted
    pub logs: Vec<LogEntry>,
    /// New contracts created
    pub created_contracts: Vec<(Address, Bytes)>,
    /// State changes
    pub state_changes: StateChanges,
}

/// A single log entry (event).
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    /// Contract address that emitted the log
    pub address: Address,
    /// Topics (event signature + indexed params)
    pub topics: Vec<[u8; 32]>,
    /// Data (non-indexed params)
    pub data: Bytes,
}

/// State changes made during execution.
#[derive(Debug, Clone, Default)]
pub struct StateChanges {
    /// Storage writes: (address, key) -> value
    pub storage: std::collections::HashMap<(Address, [u8; 32]), Option<[u8; 32]>>,
    /// Balance transfers
    pub transfers: Vec<(Address, Address, U256)>, // from, to, amount
}

impl ExecutionResult {
    /// Create a successful result.
    pub fn success(data: Bytes, gas_used: u64) -> Self {
        Self {
            success: true,
            data,
            gas_used,
            gas_refunded: 0,
            logs: Vec::new(),
            created_contracts: Vec::new(),
            state_changes: StateChanges::default(),
        }
    }

    /// Create a failed result.
    pub fn failure(error: VmError, gas_used: u64) -> Self {
        Self {
            success: false,
            data: Bytes::from(error.to_string()),
            gas_used,
            gas_refunded: 0,
            logs: Vec::new(),
            created_contracts: Vec::new(),
            state_changes: StateChanges::default(),
        }
    }

    /// Add a log entry.
    pub fn with_log(mut self, log: LogEntry) -> Self {
        self.logs.push(log);
        self
    }

    /// Add state changes.
    pub fn with_state_changes(mut self, changes: StateChanges) -> Self {
        self.state_changes = changes;
        self
    }
}

/// The main Merklith VM.
pub struct MerklithVM {
    #[allow(dead_code)]
    engine: Engine,
    gas_schedule: GasSchedule,
}

impl MerklithVM {
    /// Create a new VM with default settings.
    pub fn new() -> Result<Self, VmError> {
        let mut config = Config::new();
        config
            .wasm_bulk_memory(true)
            .wasm_multi_value(true)
            .wasm_reference_types(true)
            .cranelift_opt_level(wasmtime::OptLevel::Speed);

        let engine = Engine::new(&config)
            .map_err(|e| VmError::ExecutionError(format!("Failed to create engine: {}", e)))?;

        Ok(Self {
            engine,
            gas_schedule: GasSchedule::default(),
        })
    }

    /// Create with custom gas schedule.
    pub fn with_gas_schedule(mut self, schedule: GasSchedule) -> Self {
        self.gas_schedule = schedule;
        self
    }

    /// Execute a contract call.
    pub fn execute(
        &self,
        ctx: ExecutionContext,
    ) -> Result<ExecutionResult, VmError> {
        // Validate code
        if ctx.code.is_empty() {
            return Err(VmError::ContractNotFound(
                format!("Contract {} has no code", ctx.contract_address)
            ));
        }
        
        // Validate gas limit
        const MAX_GAS_LIMIT: u64 = 30_000_000; // Maximum gas limit (30M)
        const MIN_GAS_LIMIT: u64 = 21_000; // Minimum gas for a transfer
        
        if ctx.gas_limit > MAX_GAS_LIMIT {
            return Err(VmError::ExecutionError(
                format!("Gas limit {} exceeds maximum {}", ctx.gas_limit, MAX_GAS_LIMIT)
            ));
        }
        
        if ctx.gas_limit < MIN_GAS_LIMIT {
            return Err(VmError::ExecutionError(
                format!("Gas limit {} is below minimum {}", ctx.gas_limit, MIN_GAS_LIMIT)
            ));
        }

        // Create gas tracker
        let mut gas_tracker = GasTracker::new(ctx.gas_limit, self.gas_schedule);
        
        // Deduct base gas cost
        gas_tracker.charge(21000)?;

        // For simple contracts, we'll interpret the bytecode directly
        // This is a simplified interpreter, not full WASM
        
        // Check if this is a simple transfer (no code)
        if ctx.code.len() < 4 {
            return Ok(ExecutionResult::success(
                ctx.input,
                gas_tracker.used(),
            ));
        }

        // Simple bytecode interpreter
        let result = self.interpret_bytecode(&ctx.code, &ctx.input, &mut gas_tracker)?;

        Ok(ExecutionResult::success(
            result,
            gas_tracker.used(),
        ))
    }

    /// Helper function to safely push to stack with size limit check
    #[inline]
    fn safe_push(stack: &mut Vec<Vec<u8>>, value: Vec<u8>) -> Result<(), VmError> {
        if stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::ExecutionError("Stack overflow: maximum stack size exceeded".to_string()));
        }
        stack.push(value);
        Ok(())
    }

    /// Simple bytecode interpreter
    fn interpret_bytecode(
        &self,
        code: &[u8],
        input: &[u8],
        gas: &mut GasTracker,
    ) -> Result<Bytes, VmError> {
        let mut pc = 0;
        let mut stack: Vec<Vec<u8>> = Vec::new();
        let mut memory: Vec<u8> = vec![0; 1024];
        
        while pc < code.len() {
            let opcode = code[pc];
            pc += 1;
            
            match opcode {
                0x00 => {
                    // STOP
                    break;
                }
                0x01 => {
                    // ADD
                    gas.charge(3)?;
                    if stack.len() >= 2 {
                        let b = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        let a = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        // Simple addition (first byte only for simplicity)
                        let mut result = a;
                        if !result.is_empty() && !b.is_empty() {
                            result[0] = result[0].wrapping_add(b[0]);
                        }
                        Self::safe_push(&mut stack, result)?;
                    }
                }
                0x02 => {
                    // MUL
                    gas.charge(5)?;
                    if stack.len() >= 2 {
                        let b = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        let a = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        let mut result = vec![0u8; 32];
                        if !a.is_empty() && !b.is_empty() {
                            result[0] = a[0].wrapping_mul(b[0]);
                        }
                        Self::safe_push(&mut stack, result)?;
                    }
                }
                0x10 => {
                    // LT
                    gas.charge(3)?;
                    if stack.len() >= 2 {
                        let b = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        let a = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        let result = if !a.is_empty() && !b.is_empty() && a[0] < b[0] {
                            vec![1]
                        } else {
                            vec![0]
                        };
                        Self::safe_push(&mut stack, result)?;
                    }
                }
                0x14 => {
                    // EQ
                    gas.charge(3)?;
                    if stack.len() >= 2 {
                        let b = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        let a = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        let result = if a == b { vec![1] } else { vec![0] };
                        Self::safe_push(&mut stack, result)?;
                    }
                }
                0x35 => {
                    // CALLDATALOAD
                    gas.charge(3)?;
                    // Push input data to stack
                    Self::safe_push(&mut stack, input.to_vec())?;
                }
                0x36 => {
                    // CALLDATASIZE
                    gas.charge(2)?;
                    Self::safe_push(&mut stack, vec![input.len() as u8])?;
                }
                0x50 => {
                    // POP
                    gas.charge(2)?;
                    stack.pop();
                }
                0x51 => {
                    // MLOAD
                    gas.charge(3)?;
                    if !stack.is_empty() {
                        let offset = stack.last()
                            .and_then(|v| v.first().copied())
                            .unwrap_or(0) as usize;
                        if offset < memory.len() {
                            Self::safe_push(&mut stack, memory[offset..offset+32.min(memory.len()-offset)].to_vec())?;
                        }
                    }
                }
                0x52 => {
                    // MSTORE
                    gas.charge(3)?;
                    if stack.len() >= 2 {
                        let offset = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        let value = stack.pop().ok_or(VmError::ExecutionError("Stack underflow".to_string()))?;
                        if let Some(&off) = offset.first() {
                            let off = off as usize;
                            // Limit value write to 32 bytes max to prevent unbounded writes
                            let max_write = 32.min(value.len());
                            for (i, &byte) in value.iter().take(max_write).enumerate() {
                                if off + i < memory.len() {
                                    memory[off + i] = byte;
                                }
                            }
                        }
                    }
                }
                0x60..=0x7F => {
                    // PUSH1-PUSH32
                    let n = (opcode - 0x5F) as usize;
                    gas.charge(3)?;
                    if pc + n <= code.len() {
                        Self::safe_push(&mut stack, code[pc..pc+n].to_vec())?;
                        pc += n;
                    }
                }
                0xF0 => {
                    // CREATE - deploy new contract
                    gas.charge(32000)?;
                    // Return creation code
                    if let Some(code) = stack.pop() {
                        return Ok(Bytes::from(code));
                    }
                }
                0xF1 => {
                    // CALL
                    gas.charge(700)?;
                    // Simplified: just push success
                    Self::safe_push(&mut stack, vec![1])?;
                }
                0xFD => {
                    // REVERT
                    return Err(VmError::ExecutionError("Revert".to_string()));
                }
                0xFF => {
                    // SELFDESTRUCT
                    gas.charge(5000)?;
                    break;
                }
                _ => {
                    // Unknown opcode - skip
                    tracing::warn!("Unknown opcode: 0x{:02x}", opcode);
                }
            }
        }
        
        // Return top of stack or empty
        Ok(Bytes::from(stack.pop().unwrap_or_default()))
    }
}

impl Default for MerklithVM {
    fn default() -> Self {
        // Attempt to create VM, fall back to a basic instance on failure
        Self::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to create default VM: {}, using fallback", e);
            // Create a minimal fallback VM with default engine config
            let engine = Engine::default();
            Self {
                engine,
                gas_schedule: GasSchedule::default(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let vm = MerklithVM::new();
        assert!(vm.is_ok());
    }

    #[test]
    fn test_execution_context_creation() {
        let ctx = ExecutionContext::new_call(
            Address::ZERO,
            Address::ZERO,
            Address::ZERO,
            1_000_000,
            Bytes::from(vec![1, 2, 3]),
        );
        
        assert_eq!(ctx.gas_limit, 1_000_000);
        assert!(!ctx.input.is_empty());
    }

    #[test]
    fn test_execution_context_with_value() {
        let ctx = ExecutionContext::new_call(
            Address::ZERO,
            Address::ZERO,
            Address::ZERO,
            1_000_000,
            Bytes::new(),
        ).with_value(U256::from(1000u64));
        
        assert_eq!(ctx.value, U256::from(1000u64));
    }

    #[test]
    fn test_execution_result_success() {
        let data = Bytes::from(vec![0x01, 0x02]);
        let result = ExecutionResult::success(data.clone(), 50000);
        
        assert!(result.success);
        assert_eq!(result.data, data);
        assert_eq!(result.gas_used, 50000);
    }

    #[test]
    fn test_execution_result_failure() {
        let err = VmError::OutOfGas { used: 100, limit: 90 };
        let result = ExecutionResult::failure(err, 90);
        
        assert!(!result.success);
        assert_eq!(result.gas_used, 90);
    }

    #[test]
    fn test_log_entry() {
        let log = LogEntry {
            address: Address::ZERO,
            topics: vec![[1u8; 32]],
            data: Bytes::from(vec![0xab, 0xcd]),
        };
        
        assert_eq!(log.topics.len(), 1);
        assert_eq!(log.data.len(), 2);
    }

    #[test]
    fn test_execution_result_with_log() {
        let log = LogEntry {
            address: Address::ZERO,
            topics: vec![],
            data: Bytes::new(),
        };
        
        let result = ExecutionResult::success(Bytes::new(), 0)
            .with_log(log.clone());
        
        assert_eq!(result.logs.len(), 1);
        assert_eq!(result.logs[0], log);
    }

    #[test]
    fn test_state_changes() {
        let mut changes = StateChanges::default();
        
        // Add a storage write
        changes.storage.insert(
            (Address::ZERO, [1u8; 32]),
            Some([2u8; 32]),
        );
        
        assert_eq!(changes.storage.len(), 1);
    }

    #[test]
    fn test_contract_creation_too_large() {
        let large_code = vec![0u8; MAX_CODE_SIZE + 1];
        let result = ExecutionContext::new_create(
            Address::ZERO,
            Address::ZERO,
            1_000_000,
            Bytes::from(large_code),
        );
        
        assert!(matches!(result, Err(VmError::CodeSizeExceeded { .. })));
    }

    #[test]
    fn test_vm_default() {
        // This might panic if VM creation fails, but that's acceptable for default()
        let _vm = MerklithVM::default();
    }
}
