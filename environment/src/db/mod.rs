//! Database Cluster
//! 
//! Simulated primary-replica database cluster with configurable replication lag.

pub mod primary;
pub mod replica;

use crate::RequestType;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Database cluster
pub struct Cluster {
    primary: Arc<primary::PrimaryDb>,
    replicas: Vec<Arc<replica::ReplicaDb>>,
}

/// Shared cluster state
pub struct ClusterState {
    pub data: RwLock<HashMap<String, String>>,
    pub write_timestamps: RwLock<HashMap<String, u64>>,
}

impl Cluster {
    pub fn new(replication_lag_ms: u64) -> Self {
        let state = Arc::new(ClusterState {
            data: RwLock::new(HashMap::new()),
            write_timestamps: RwLock::new(HashMap::new()),
        });

        let primary = Arc::new(primary::PrimaryDb::new(state.clone()));
        let replicas: Vec<_> = (0..2)
            .map(|i| Arc::new(replica::ReplicaDb::new(state.clone(), i as u32, replication_lag_ms)))
            .collect();

        Self { primary, replicas }
    }

    pub async fn execute(&self, req: RequestType, use_primary: bool) -> crate::Response {
        match req {
            RequestType::Write { key, value } => self.primary.write(key, value).await,
            RequestType::Read { key } => {
                if use_primary {
                    self.primary.read(&key).await
                } else {
                    let idx = rand_idx(self.replicas.len());
                    self.replicas[idx].read(&key).await
                }
            }
            RequestType::BatchRead { keys } => {
                if use_primary {
                    self.primary.batch_read(&keys).await
                } else {
                    let idx = rand_idx(self.replicas.len());
                    self.replicas[idx].batch_read(&keys).await
                }
            }
            RequestType::Invalidate { key } => self.primary.invalidate(&key).await,
        }
    }
}

static mut CLUSTER: Option<Cluster> = None;

pub fn init_cluster() {
    unsafe {
        CLUSTER = Some(Cluster::new(50));
    }
}

pub fn get_cluster() -> &'static Cluster {
    unsafe {
        CLUSTER.as_ref().expect("Cluster not initialized")
    }
}

fn rand_idx(max: usize) -> usize {
    use std::time::SystemTime;
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    (now.subsec_nanos() as usize) % max
}
