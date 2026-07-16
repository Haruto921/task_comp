//! Stale Read Fix - Session-Consistent Database Router
//! 
//! This crate implements a session-consistent routing layer for replicated databases.
//! 
//! # Architecture
//! 
//! The system consists of several layers:
//! 
//! 1. **Session Pool**: Manages reusable session contexts
//! 2. **Context Manager**: Handles request-scoped state propagation
//! 3. **Middleware Chain**: Applies request/response transformations
//! 4. **Router**: Routes requests based on consistency requirements
//! 5. **Database Cluster**: Primary-replica with configurable replication lag
//! 
//! # Consistency Model
//! 
//! After a write operation, all subsequent reads within the same logical session
//! are routed to the primary database to ensure read-your-writes consistency.
//! This is achieved by tracking a "consistency token" in the session context.

pub mod context;
pub mod db;
pub mod router;
pub mod middleware;
pub mod pool;
pub mod session;

use serde::{Deserialize, Serialize};

/// Request types handled by the router
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    /// Write operation - always routed to primary
    Write { key: String, value: String },
    /// Read operation - routed based on consistency requirements
    Read { key: String },
    /// Batch read for efficiency
    BatchRead { keys: Vec<String> },
    /// Cache invalidation
    Invalidate { key: String },
}

/// Response from database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub value: Option<String>,
    pub values: Option<Vec<Option<String>>>,
    pub latency_ms: u64,
    pub routed_to: String,
    pub error: Option<String>,
}

impl Response {
    pub fn success(value: Option<String>, latency_ms: u64, routed_to: &str) -> Self {
        Self {
            success: true,
            value,
            values: None,
            latency_ms,
            routed_to: routed_to.to_string(),
            error: None,
        }
    }

    pub fn batch_success(values: Vec<Option<String>>, latency_ms: u64, routed_to: &str) -> Self {
        Self {
            success: true,
            value: None,
            values: Some(values),
            latency_ms,
            routed_to: routed_to.to_string(),
            error: None,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            value: None,
            values: None,
            latency_ms: 0,
            routed_to: "none".to_string(),
            error: Some(msg),
        }
    }
}

/// Process a request through the full middleware + routing pipeline
pub async fn process_request(
    req: RequestType,
    ctx: &context::RequestContext,
) -> Response {
    let req = middleware::apply_preprocessing(req, ctx).await;
    let resp = router::handle_request(req, ctx).await;
    middleware::apply_postprocessing(resp, ctx).await
}

/// Initialize the database cluster
pub fn init_cluster() {
    db::init_cluster();
}
