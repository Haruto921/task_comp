//! Storage Backends
//! 
//! This module provides different storage backends for the consistency token.
//! 
//! # Available Backends
//! 
//! 1. **ThreadLocalStorage**: Fastest, uses thread-local storage
//! 2. **ArcMutexStorage**: Thread-safe, uses Arc<Mutex<...>>
//! 3. **RwLockStorage**: Optimized for read-heavy workloads
//! 
//! # Configuration
//! 
//! The storage backend can be selected at compile time based on
//! performance requirements. ThreadLocalStorage is recommended for
//! high-throughput scenarios where each request is processed by
//! a single thread.

use std::sync::{Arc, Mutex};

/// Storage backend trait
pub trait StorageBackend: Send + Sync {
    fn set(&self, value: Option<u64>);
    fn get(&self) -> Option<u64>;
}

/// Arc + Mutex based storage
/// 
/// This provides thread-safety through Arc (shared ownership) and Mutex
/// (synchronized access). Suitable for scenarios where the same context
/// may be accessed from multiple threads.
pub struct ArcMutexStorage {
    value: Arc<Mutex<Option<u64>>>,
}

impl ArcMutexStorage {
    pub fn new() -> Self {
        Self {
            value: Arc::new(Mutex::new(None)),
        }
    }
}

impl StorageBackend for ArcMutexStorage {
    fn set(&self, value: Option<u64>) {
        *self.value.lock().unwrap() = value;
    }

    fn get(&self) -> Option<u64> {
        *self.value.lock().unwrap()
    }
}

impl Default for ArcMutexStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for storage backend selection
pub enum StorageConfig {
    /// Use thread-local storage (fastest)
    ThreadLocal,
    /// Use Arc<Mutex<...>> (thread-safe)
    ArcMutex,
    /// Use Arc<RwLock<...>> (optimized for reads)
    RwLock,
}

impl Default for StorageConfig {
    fn default() -> Self {
        // Default to thread-local for performance
        // Note: This is the root cause of the bug!
        StorageConfig::ThreadLocal
    }
}
