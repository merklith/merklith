//! Reentrancy protection for contract calls.
//!
//! Prevents reentrancy attacks by tracking the call stack and
//! detecting when a contract is called multiple times within
//! the same transaction.

use std::collections::HashSet;
use crate::MAX_CALL_DEPTH;
use crate::error::VmError;

/// Tracks the call stack and prevents reentrancy attacks.
#[derive(Debug, Clone)]
pub struct ReentrancyGuard {
    /// Current call stack (addresses being called)
    stack: Vec<CallFrame>,
    /// Set of addresses in the call stack (for O(1) lookup)
    in_stack: HashSet<[u8; 20]>,
    /// Maximum call depth allowed
    max_depth: usize,
    /// Whether reentrancy is allowed (default: false)
    allow_reentrancy: bool,
}

/// A single frame in the call stack.
#[derive(Debug, Clone, PartialEq)]
pub struct CallFrame {
    /// Contract address being called
    pub address: [u8; 20],
    /// Caller address
    pub caller: [u8; 20],
    /// Value transferred in this call
    pub value: u128,
    /// Call depth (0 for top-level)
    pub depth: usize,
}

impl ReentrancyGuard {
    /// Create a new reentrancy guard with default settings.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            in_stack: HashSet::new(),
            max_depth: MAX_CALL_DEPTH,
            allow_reentrancy: false,
        }
    }

    /// Create a new guard with custom max depth.
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            stack: Vec::new(),
            in_stack: HashSet::new(),
            max_depth,
            allow_reentrancy: false,
        }
    }

    /// Allow reentrancy (not recommended for production).
    pub fn allow_reentrancy(mut self) -> Self {
        self.allow_reentrancy = true;
        self
    }

    /// Enter a new call frame.
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Call depth is exceeded
    /// - Reentrancy is detected (unless allowed)
    pub fn enter(&mut self, address: [u8; 20], caller: [u8; 20], value: u128) -> Result<usize, VmError> {
        let depth = self.stack.len();

        // Check call depth
        if depth >= self.max_depth {
            return Err(VmError::CallDepthExceeded { depth });
        }

        // Check for reentrancy
        if !self.allow_reentrancy && self.in_stack.contains(&address) {
            return Err(VmError::ReentrancyViolation(
                format!("Contract {:x?} is already in call stack", address)
            ));
        }

        // Add to stack
        self.in_stack.insert(address);
        self.stack.push(CallFrame {
            address,
            caller,
            value,
            depth,
        });

        Ok(depth)
    }

    /// Exit the current call frame.
    /// 
    /// # Errors
    /// Returns an error if trying to exit an empty stack.
    pub fn exit(&mut self) -> Result<CallFrame, VmError> {
        let frame = self.stack.pop()
            .ok_or_else(|| VmError::ReentrancyViolation("Call stack is empty".to_string()))?;
        
        self.in_stack.remove(&frame.address);
        Ok(frame)
    }

    /// Get the current call depth.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Check if currently in a call.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Get the current call frame (last in stack).
    pub fn current_frame(&self) -> Option<&CallFrame> {
        self.stack.last()
    }

    /// Get the full call stack.
    pub fn stack(&self) -> &[CallFrame] {
        &self.stack
    }

    /// Check if an address is in the current call stack.
    pub fn contains(&self, address: &[u8; 20]) -> bool {
        self.in_stack.contains(address)
    }

    /// Get the root caller (first frame in stack).
    pub fn root_caller(&self) -> Option<&CallFrame> {
        self.stack.first()
    }

    /// Check if the current caller is a contract (not EOA).
    /// Returns true if the caller is in the stack (is a contract).
    pub fn caller_is_contract(&self, caller: &[u8; 20]) -> bool {
        self.in_stack.contains(caller)
    }

    /// Reset the guard (clear all state).
    pub fn reset(&mut self) {
        self.stack.clear();
        self.in_stack.clear();
    }

    /// Get total value transferred through the call chain.
    pub fn total_value_transferred(&self) -> u128 {
        self.stack.iter().map(|f| f.value).sum()
    }

    /// Get the sender of the current transaction.
    /// This is the first caller in the stack, or the caller if empty.
    pub fn tx_origin(&self) -> Option<[u8; 20]> {
        self.stack.first().map(|f| f.caller)
    }
}

impl Default for ReentrancyGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// A scoped guard that automatically exits the call frame on drop.
pub struct CallScope<'a> {
    guard: &'a mut ReentrancyGuard,
    exited: bool,
}

impl<'a> CallScope<'a> {
    /// Create a new call scope and enter it.
    pub fn enter(
        guard: &'a mut ReentrancyGuard,
        address: [u8; 20],
        caller: [u8; 20],
        value: u128,
    ) -> Result<Self, VmError> {
        guard.enter(address, caller, value)?;
        Ok(Self { guard, exited: false })
    }

    /// Manually exit the scope.
    pub fn exit(mut self) -> Result<CallFrame, VmError> {
        self.exited = true;
        self.guard.exit()
    }

    /// Get current depth.
    pub fn depth(&self) -> usize {
        self.guard.depth()
    }
}

impl<'a> Drop for CallScope<'a> {
    fn drop(&mut self) {
        if !self.exited {
            // Auto-exit on drop, ignore errors
            let _ = self.guard.exit();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_addr(n: u8) -> [u8; 20] {
        let mut addr = [0u8; 20];
        addr[19] = n;
        addr
    }

    #[test]
    fn test_basic_enter_exit() {
        let mut guard = ReentrancyGuard::new();
        
        assert!(guard.is_empty());
        assert_eq!(guard.depth(), 0);

        // Enter first call
        guard.enter(test_addr(1), test_addr(0), 100).unwrap();
        assert_eq!(guard.depth(), 1);
        assert!(!guard.is_empty());

        // Exit
        let frame = guard.exit().unwrap();
        assert_eq!(frame.address, test_addr(1));
        assert_eq!(frame.value, 100);
        assert!(guard.is_empty());
    }

    #[test]
    fn test_call_depth_limit() {
        let mut guard = ReentrancyGuard::with_max_depth(3);
        
        // Enter 3 calls (max depth)
        guard.enter(test_addr(1), test_addr(0), 0).unwrap();
        guard.enter(test_addr(2), test_addr(1), 0).unwrap();
        guard.enter(test_addr(3), test_addr(2), 0).unwrap();
        
        assert_eq!(guard.depth(), 3);

        // 4th call should fail
        let result = guard.enter(test_addr(4), test_addr(3), 0);
        assert!(matches!(result, Err(VmError::CallDepthExceeded { depth: 3 })));
    }

    #[test]
    fn test_reentrancy_protection() {
        let mut guard = ReentrancyGuard::new();
        
        // Enter contract A
        guard.enter(test_addr(1), test_addr(0), 0).unwrap();
        
        // Try to reenter contract A - should fail
        let result = guard.enter(test_addr(1), test_addr(2), 0);
        assert!(matches!(result, Err(VmError::ReentrancyViolation(_))));
    }

    #[test]
    fn test_reentrancy_allowed() {
        let mut guard = ReentrancyGuard::with_max_depth(64).allow_reentrancy();
        
        // Enter contract A
        guard.enter(test_addr(1), test_addr(0), 0).unwrap();
        
        // Reenter contract A - should succeed
        guard.enter(test_addr(1), test_addr(2), 0).unwrap();
        assert_eq!(guard.depth(), 2);
    }

    #[test]
    fn test_call_scope() {
        let mut guard = ReentrancyGuard::new();
        
        {
            let scope = CallScope::enter(&mut guard, test_addr(1), test_addr(0), 100).unwrap();
            assert_eq!(scope.depth(), 1);
            // scope drops here, auto-exiting
        }
        
        assert!(guard.is_empty());
    }

    #[test]
    fn test_contains() {
        let mut guard = ReentrancyGuard::new();
        
        assert!(!guard.contains(&test_addr(1)));
        
        guard.enter(test_addr(1), test_addr(0), 0).unwrap();
        assert!(guard.contains(&test_addr(1)));
        assert!(!guard.contains(&test_addr(2)));
        
        guard.exit().unwrap();
        assert!(!guard.contains(&test_addr(1)));
    }

    #[test]
    fn test_total_value() {
        let mut guard = ReentrancyGuard::new();
        
        guard.enter(test_addr(1), test_addr(0), 100).unwrap();
        guard.enter(test_addr(2), test_addr(1), 200).unwrap();
        guard.enter(test_addr(3), test_addr(2), 50).unwrap();
        
        assert_eq!(guard.total_value_transferred(), 350);
    }

    #[test]
    fn test_tx_origin() {
        let mut guard = ReentrancyGuard::new();
        
        assert_eq!(guard.tx_origin(), None);
        
        let origin = test_addr(0);
        guard.enter(test_addr(1), origin, 0).unwrap();
        guard.enter(test_addr(2), test_addr(1), 0).unwrap();
        
        assert_eq!(guard.tx_origin(), Some(origin));
    }

    #[test]
    fn test_caller_is_contract() {
        let mut guard = ReentrancyGuard::new();
        
        // First call is from EOA (not in stack)
        assert!(!guard.caller_is_contract(&test_addr(0)));
        
        // Enter contract A
        guard.enter(test_addr(1), test_addr(0), 0).unwrap();
        
        // Contract A is now in the stack
        assert!(guard.caller_is_contract(&test_addr(1)));
    }

    #[test]
    fn test_reset() {
        let mut guard = ReentrancyGuard::new();
        
        guard.enter(test_addr(1), test_addr(0), 0).unwrap();
        guard.enter(test_addr(2), test_addr(1), 0).unwrap();
        
        guard.reset();
        
        assert!(guard.is_empty());
        assert!(!guard.contains(&test_addr(1)));
    }

    #[test]
    fn test_exit_empty_stack_fails() {
        let mut guard = ReentrancyGuard::new();
        
        let result = guard.exit();
        assert!(matches!(result, Err(VmError::ReentrancyViolation(_))));
    }
}
