A Rust microservice responsible for user session data is exhibiting intermittent "Read-Your-Writes" consistency violations in production. After a successful write operation, immediate subsequent reads occasionally return stale data from a read replica instead of the primary database.

The service uses an asynchronous runtime with a work-stealing scheduler. The database routing logic relies on a session-specific "consistency cookie" to determine whether a request must be routed to the primary database. When this cookie is present, reads should bypass replicas.

Investigate the session management and routing implementation. Ensure that once a write occurs, all subsequent operations within the same logical request context correctly observe the consistency requirement, even across asynchronous suspension points.

Fix the defect so that the integration test `test_read_your_writes` passes consistently without flakiness.
