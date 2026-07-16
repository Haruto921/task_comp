//! Primary Database Node

use crate::{RequestType, Response};
use super::ClusterState;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Instant};

pub struct PrimaryDb {
    state: Arc<ClusterState>,
}

impl PrimaryDb {
    pub fn new(state: Arc<ClusterState>) -> Self {
        Self { state }
    }

    pub async fn write(&self, key: String, value: String) -> Response {
        let start = Instant::now();
        tokio::task::yield_now().await;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        {
            let mut data = self.state.data.write().unwrap();
            data.insert(key.clone(), value);
        }
        
        {
            let mut timestamps = self.state.write_timestamps.write().unwrap();
            timestamps.insert(key, timestamp);
        }
        
        let latency = start.elapsed().as_millis() as u64;
        Response::success(None, latency, "primary")
    }

    pub async fn read(&self, key: &str) -> Response {
        let start = Instant::now();
        tokio::task::yield_now().await;
        
        let value = {
            let data = self.state.data.read().unwrap();
            data.get(key).cloned()
        };
        
        let latency = start.elapsed().as_millis() as u64;
        Response::success(value, latency, "primary")
    }

    pub async fn batch_read(&self, keys: &[String]) -> Response {
        let start = Instant::now();
        tokio::task::yield_now().await;
        
        let values: Vec<_> = {
            let data = self.state.data.read().unwrap();
            keys.iter().map(|k| data.get(k).cloned()).collect()
        };
        
        let latency = start.elapsed().as_millis() as u64;
        Response::batch_success(values, latency, "primary")
    }

    pub async fn invalidate(&self, _key: &str) -> Response {
        let start = Instant::now();
        tokio::task::yield_now().await;
        let latency = start.elapsed().as_millis() as u64;
        Response::success(None, latency, "primary")
    }
}
