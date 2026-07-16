//! Request Router
//! 
//! This module handles request routing to the appropriate database node.

use crate::{RequestType, Response, db::get_cluster, context::RequestContext};

/// Handle a database request
/// 
/// Routes requests based on:
/// - Request type (writes always go to primary)
/// - Consistency token (if set, reads go to primary)
pub async fn handle_request(req: RequestType, ctx: &RequestContext) -> Response {
    match req {
        RequestType::Write { key, value } => {
            let resp = get_cluster().execute(
                RequestType::Write { key, value }, 
                true
            ).await;
            
            // Set consistency token on successful write
            if resp.success {
                let token = generate_token();
                ctx.set_token(token);
            }
            
            resp
        }
        RequestType::Read { key } => {
            let use_primary = ctx.requires_primary();
            
            get_cluster().execute(
                RequestType::Read { key },
                use_primary
            ).await
        }
        RequestType::BatchRead { keys } => {
            let use_primary = ctx.requires_primary();
            
            get_cluster().execute(
                RequestType::BatchRead { keys },
                use_primary
            ).await
        }
        RequestType::Invalidate { key } => {
            get_cluster().execute(
                RequestType::Invalidate { key },
                false
            ).await
        }
    }
}

/// Generate a unique consistency token
fn generate_token() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    now.wrapping_mul(31).wrapping_add(counter)
}
