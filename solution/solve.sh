#!/bin/bash
# Solution Script for Stale Read Fix
# 
# The bug: The TokenManager uses thread_local storage which doesn't
# survive async task migration.
#
# The fix: Store the token directly in the RequestContext struct.

set -euo pipefail

cd /workspace/project/stale-read-hard/environment

echo "=== Stale Read Fix - Applying Solution ==="

# Fix the token manager to use struct-based storage instead of thread_local

cat > src/context/token.rs << 'EOF'
//! Token Manager
//! 
//! This module manages the consistency token that tracks whether a write
//! has occurred in the current session.

use std::sync::{Arc, Mutex};

/// Token manager for consistency tracking
/// 
/// This stores the token directly in the struct, not in thread-local storage.
/// This ensures the token persists across async task migration.
pub struct TokenManager {
    /// The consistency token
    token: Arc<Mutex<Option<u64>>>,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new() -> Self {
        Self {
            token: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the consistency token
    pub fn set_token(&self, token: u64) {
        *self.token.lock().unwrap() = Some(token);
    }

    /// Get the current token
    pub fn get_token(&self) -> Option<u64> {
        *self.token.lock().unwrap()
    }

    /// Clear the token
    pub fn clear(&self) {
        *self.token.lock().unwrap() = None;
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_basic() {
        let manager = TokenManager::new();
        
        assert!(manager.get_token().is_none());
        
        manager.set_token(12345);
        assert_eq!(manager.get_token(), Some(12345));
        
        manager.clear();
        assert!(manager.get_token().is_none());
    }
}
EOF

echo "Fixed: Replaced thread_local storage with struct-based Arc<Mutex<...>>"

echo ""
echo "=== Running Tests ==="
cargo test --test integration_test --release -- --nocapture

echo ""
echo "=== All Tests Passed ==="
echo "The read-your-writes consistency bug has been fixed."
