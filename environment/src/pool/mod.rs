//! Session Pool
//! 
//! This module provides session pooling for efficient context reuse.

use crate::context::RequestContext;
use std::collections::VecDeque;
use std::sync::Mutex;

/// Session pool for context reuse
pub struct SessionPool {
    pool: Mutex<VecDeque<RequestContext>>,
    config: PoolConfig,
}

#[derive(Debug, Clone)]
struct PoolConfig {
    max_size: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
        }
    }
}

impl SessionPool {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(VecDeque::new()),
            config: PoolConfig::default(),
        }
    }

    pub fn acquire(&self) -> RequestContext {
        RequestContext::new()
    }

    pub fn release(&self, mut ctx: RequestContext) {
        ctx.clear_token();
        if let Ok(mut pool) = self.pool.try_lock() {
            if pool.len() < self.config.max_size {
                pool.push_back(ctx);
            }
        }
    }
}

impl Default for SessionPool {
    fn default() -> Self {
        Self::new()
    }
}
