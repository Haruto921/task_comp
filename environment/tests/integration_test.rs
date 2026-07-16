//! Integration Tests for Session Consistency
//! 
//! These tests verify that the session consistency layer correctly
//! maintains the read-your-writes guarantee across async operations.

use stale_read_fix::{RequestType, process_request, db::init_cluster, context::RequestContext};

/// Test basic read-your-writes consistency
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_read_your_writes_basic() {
    init_cluster();
    
    let ctx = RequestContext::new();
    
    let write_resp = process_request(
        RequestType::Write {
            key: "user:1001".to_string(),
            value: "session_data".to_string(),
        },
        &ctx
    ).await;
    
    assert!(write_resp.success, "Write should succeed");
    
    let read_resp = process_request(
        RequestType::Read {
            key: "user:1001".to_string(),
        },
        &ctx
    ).await;
    
    assert_eq!(
        read_resp.value,
        Some("session_data".to_string()),
        "Read should return the written value"
    );
}

/// Test with cross-task execution
/// 
/// This test spawns a new task that uses the context. The consistency
/// token must survive the task boundary.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_cross_task_token_persistence() {
    init_cluster();
    
    let ctx = RequestContext::new();
    
    // Write and set token
    let write_resp = process_request(
        RequestType::Write {
            key: "cross:task:key".to_string(),
            value: "cross_task_value".to_string(),
        },
        &ctx
    ).await;
    
    assert!(write_resp.success);
    
    // Verify token exists
    assert!(ctx.requires_primary(), "Token should be set after write");
    
    // Spawn a new task and verify token persists
    let ctx_for_task = ctx.clone();
    let result = tokio::spawn(async move {
        // In spawned task, token should still be accessible
        if !ctx_for_task.requires_primary() {
            return Err("Token lost in spawned task");
        }
        
        // Read from spawned task
        let read_resp = process_request(
            RequestType::Read {
                key: "cross:task:key".to_string(),
            },
            &ctx_for_task
        ).await;
        
        if read_resp.value != Some("cross_task_value".to_string()) {
            return Err("Read returned wrong value in spawned task");
        }
        
        Ok(())
    }).await.unwrap();
    
    result.expect("Cross-task consistency check failed");
}

/// Test with multiple yields
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_token_after_yields() {
    init_cluster();
    
    let ctx = RequestContext::new();
    
    // Write
    process_request(
        RequestType::Write {
            key: "yield:test".to_string(),
            value: "yield_value".to_string(),
        },
        &ctx
    ).await;
    
    // Multiple yields
    tokio::task::yield_now().await;
    tokio::task::yield_now().await;
    tokio::task::yield_now().await;
    
    // Token should persist
    assert!(ctx.requires_primary(), "Token should persist after yields");
    
    let read_resp = process_request(
        RequestType::Read {
            key: "yield:test".to_string(),
        },
        &ctx
    ).await;
    
    assert_eq!(read_resp.value, Some("yield_value".to_string()));
}

/// Test with spawn_blocking
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_token_after_spawn_blocking() {
    init_cluster();
    
    let ctx = RequestContext::new();
    
    // Write
    process_request(
        RequestType::Write {
            key: "blocking:test".to_string(),
            value: "blocking_value".to_string(),
        },
        &ctx
    ).await;
    
    // Verify token in blocking task
    let ctx_clone = ctx.clone();
    let has_token = tokio::task::spawn_blocking(move || {
        ctx_clone.requires_primary()
    }).await.unwrap();
    
    assert!(has_token, "Token should be accessible in spawn_blocking");
}

/// Test multiple contexts are isolated
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_context_isolation() {
    init_cluster();
    
    let ctx1 = RequestContext::new();
    let ctx2 = RequestContext::new();
    
    // Write with ctx1
    process_request(
        RequestType::Write {
            key: "shared".to_string(),
            value: "from_ctx1".to_string(),
        },
        &ctx1
    ).await;
    
    // ctx1 should have token
    assert!(ctx1.requires_primary());
    
    // ctx2 should NOT have token
    assert!(!ctx2.requires_primary());
    
    // ctx1 read should work
    let read1 = process_request(
        RequestType::Read { key: "shared".to_string() },
        &ctx1
    ).await;
    assert_eq!(read1.value, Some("from_ctx1".to_string()));
}

/// Test concurrent sessions
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_concurrent_sessions() {
    use futures::future::join_all;
    
    init_cluster();
    
    let contexts: Vec<_> = (0..10).map(|_| RequestContext::new()).collect();
    
    // Write in parallel
    let write_futures: Vec<_> = contexts.iter().enumerate().map(|(i, ctx)| {
        async move {
            let resp = process_request(
                RequestType::Write {
                    key: format!("concurrent:{}", i),
                    value: format!("value_{}", i),
                },
                ctx
            ).await;
            assert!(resp.success);
        }
    }).collect();
    
    join_all(write_futures).await;
    
    // All tokens should persist
    for (i, ctx) in contexts.iter().enumerate() {
        assert!(ctx.requires_primary(), "Session {} should have token", i);
        
        // Read should work
        let read_resp = process_request(
            RequestType::Read {
                key: format!("concurrent:{}", i),
            },
            ctx
        ).await;
        
        assert_eq!(
            read_resp.value,
            Some(format!("value_{}", i)),
            "Session {} should read its own value",
            i
        );
    }
}
