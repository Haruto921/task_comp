//! Session Context Management
//! 
//! This module provides the request context for session-consistent routing.

pub mod storage;
pub mod token;

use std::sync::Arc;

/// Request context for session consistency
/// 
/// This struct provides the interface for managing session-scoped state.
#[derive(Clone)]
pub struct RequestContext {
    /// Internal storage handle
    inner: Arc<ContextInner>,
}

/// Internal context storage
struct ContextInner {
    /// The token manager
    token_manager: token::TokenManager,
}

impl RequestContext {
    /// Create a new request context
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ContextInner {
                token_manager: token::TokenManager::new(),
            }),
        }
    }

    /// Set the consistency token after a write operation
    /// 
    /// When a token is set, subsequent reads will be routed to the primary
    /// to ensure read-your-writes consistency.
    pub fn set_token(&self, token: u64) {
        self.inner.token_manager.set_token(token);
    }

    /// Get the current consistency token
    pub fn get_token(&self) -> Option<u64> {
        self.inner.token_manager.get_token()
    }

    /// Clear the consistency token
    pub fn clear_token(&self) {
        self.inner.token_manager.clear();
    }

    /// Check if primary routing is required
    pub fn requires_primary(&self) -> bool {
        self.get_token().is_some()
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}
