//! Session Utilities
//! 
//! This module provides helper functions for session management.
//! 
//! # Common Patterns
//! 
//! Here are some common patterns for working with sessions:
//! 
//! ## Pattern 1: Context Cloning
//! 
//! ```ignore
//! let ctx = RequestContext::new();
//! let ctx_clone = ctx.clone();
//! tokio::spawn(async move {
//!     // Context clone can be used in spawned task
//! });
//! ```
//! 
//! ## Pattern 2: Token Extraction
//! 
//! ```ignore
//! let token = ctx.get_token().unwrap_or(0);
//! if token > 0 { /* primary routing */ }
//! ```
//! 
//! ## Pattern 3: Reset on Error
//! 
//! ```ignore
//! if let Err(e) = do_something().await {
//!     ctx.clear_token(); // Reset on error
//! }
//! ```

use crate::context::RequestContext;

/// Helper to check if a context is valid
/// 
/// This is a convenience function for validation.
pub fn is_valid_context(ctx: &RequestContext) -> bool {
    // A context is valid if it was created (always true for our impl)
    true
}

/// Helper to extract token with default
/// 
/// Returns the token or a default value if not set.
pub fn get_token_or_default(ctx: &RequestContext, default: u64) -> u64 {
    ctx.get_token().unwrap_or(default)
}

/// Reset a context after an error
/// 
/// Call this to clear the consistency token after a failed operation.
pub fn reset_on_error(ctx: &RequestContext) {
    ctx.clear_token();
}
