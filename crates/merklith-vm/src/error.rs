use thiserror::Error;

/// Errors that can occur during VM execution.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum VmError {
    #[error("Out of gas: used {used}, limit {limit}")]
    OutOfGas { used: u64, limit: u64 },

    #[error("Invalid WASM module: {0}")]
    InvalidWasm(String),

    #[error("WASM execution error: {0}")]
    ExecutionError(String),

    #[error("Memory limit exceeded: {size} > {limit}")]
    MemoryLimitExceeded { size: usize, limit: usize },

    #[error("Call depth exceeded: {depth}")]
    CallDepthExceeded { depth: usize },

    #[error("Code size exceeded: {size} > {limit}")]
    CodeSizeExceeded { size: usize, limit: usize },

    #[error("Reentrancy violation: {0}")]
    ReentrancyViolation(String),

    #[error("Unknown host function: {0}")]
    UnknownHostFunction(String),

    #[error("Invalid host function input: {0}")]
    InvalidHostInput(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Contract creation failed: {0}")]
    ContractCreationFailed(String),

    #[error("Contract call failed: {0}")]
    ContractCallFailed(String),

    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    #[error("Execution reverted: {reason:?}")]
    Reverted { reason: Option<Vec<u8>> },

    #[error("Stack overflow")]
    StackOverflow,

    #[error("Divide by zero")]
    DivideByZero,

    #[error("Integer overflow")]
    IntegerOverflow,

    #[error("Invalid memory access")]
    InvalidMemoryAccess,

    #[error("Trap: {0}")]
    Trap(String),

    #[error("Linker error: {0}")]
    LinkerError(String),

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),
}

impl From<wasmtime::Error> for VmError {
    fn from(e: wasmtime::Error) -> Self {
        VmError::ExecutionError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = VmError::OutOfGas { used: 100, limit: 90 };
        assert!(err.to_string().contains("Out of gas"));
    }
}
