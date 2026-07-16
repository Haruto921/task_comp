//! Replica Database Node

use crate::Response;
use super::ClusterState;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Instant};

pub struct ReplicaDb {
    state: Arc<ClusterState>,
    id: u32,
    lag_ms: u64,
    start_time: u64,
}

impl ReplicaDb {
    pub fn new(state: Arc<ClusterState>, id: u32, lag_ms: u64) -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        Self { state, id, lag_ms, start_time }
    }

    fn is_visible(&self, write_timestamp: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        write_timestamp + self.lag_ms <= now
    }

    pub async fn read(&self, key: &str) -> Response {
        let start = Instant::now();
        tokio::task::yield_now().await;
        
        let (value, stale) = {
            let timestamps = self.state.write_timestamps.read().unwrap();
            
            if let Some(&write_time) = timestamps.get(key) {
                if self.is_visible(write_time) {
                    let data = self.state.data.read().unwrap();
                    (data.get(key).cloned(), false)
                } else {
                    (None, true)
                }
            } else {
                let data = self.state.data.read().unwrap();
                (data.get(key).cloned(), false)
            }
        };
        
        let latency = start.elapsed().as_millis() as u64;
        
        if stale {
            Response::success(value, latency, &format!("replica{}-stale", self.id))
        } else {
            Response::success(value, latency, &format!("replica{}", self.id))
        }
    }

    pub async fn batch_read(&self, keys: &[String]) -> Response {
        let start = Instant::now();
        tokio::task::yield_now().await;
        
        let timestamps = self.state.write_timestamps.read().unwrap();
        let data = self.state.data.read().unwrap();
        
        let values: Vec<_> = keys
            .iter()
            .map(|key| {
                if let Some(&write_time) = timestamps.get(key) {
                    if self.is_visible(write_time) {
                        data.get(key).cloned()
                    } else {
                        None
                    }
                } else {
                    data.get(key).cloned()
                }
            })
            .collect();
        
        let latency = start.elapsed().as_millis() as u64;
        Response::batch_success(values, latency, &format!("replica{}", self.id))
    }
}
