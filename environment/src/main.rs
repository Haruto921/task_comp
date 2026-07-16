//! Stale Read Fix - Main Entry Point

use stale_read_fix::{RequestType, context::RequestContext, process_request};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    stale_read_fix::db::init_cluster();
    
    println!("Stale Read Fix - Session Consistency Layer");
    println!("===========================================\n");
    
    let ctx = RequestContext::new();
    
    // Example: Write then read
    let write_resp = process_request(
        RequestType::Write {
            key: "user:1".to_string(),
            value: "test".to_string(),
        },
        &ctx
    ).await;
    
    println!("Write response: {:?}", write_resp);
    
    // Read back
    let read_resp = process_request(
        RequestType::Read {
            key: "user:1".to_string(),
        },
        &ctx
    ).await;
    
    println!("Read response: {:?}", read_resp);
}
