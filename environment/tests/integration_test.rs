use stale_read_fix::{RequestType, process_request, db::init_cluster};
use stale_read_fix::context::RequestContext;
use stale_read_fix::router::handle_request;

/// Custom test harness to force thread migration on every await.
/// This reproduces the bug deterministically.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_read_your_writes() {
    init_cluster();

    let ctx = RequestContext::new();
    
    // 1. Perform a Write
    let write_req = RequestType::Write {
        key: "user:123".to_string(),
        value: "active".to_string(),
    };

    let write_resp = handle_request(write_req, &ctx).await;
    assert!(write_resp.success, "Write operation failed");

    // YIELD POINT: Explicitly yield to allow the Tokio scheduler to migrate the task.
    // In the buggy version, this migration causes the thread_local cookie to be lost.
    tokio::task::yield_now().await;
    
    // Additional yields to increase probability of migration in larger stacks
    tokio::task::yield_now().await;
    tokio::task::yield_now().await;

    // 2. Perform a Read immediately after
    let read_req = RequestType::Read {
        key: "user:123".to_string(),
    };

    let read_resp = handle_request(read_req, &ctx).await;

    // ASSERTION:
    // If the cookie was preserved, the router uses Primary -> Returns "active".
    // If the cookie was lost (bug), the router uses Replica -> Returns None.
    assert_eq!(
        read_resp.value, 
        Some("active".to_string()), 
        "Read-Your-Writes violation: Expected 'active' but got stale data. The consistency cookie was likely lost during async suspension."
    );
}