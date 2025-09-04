use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricPoint {
    pub timestamp: i64,
    pub hostname: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub disk_io_ops: u32,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricQuery {
    pub start_time: i64,
    pub end_time: i64,
    pub hostname_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStatistics {
    pub count: u64,
    pub avg_cpu_percent: f32,
    pub avg_memory_bytes: u64,
    pub avg_disk_io_ops: f32,
    pub time_range_seconds: i64,
}

pub trait MetricsService {
    type Error;
    
    async fn submit_metric(&self, metric: MetricPoint) -> Result<(), Self::Error>;
    async fn query_metrics(&self, query: MetricQuery) -> Result<Vec<MetricPoint>, Self::Error>;
    async fn get_statistics(&self, query: MetricQuery) -> Result<MetricStatistics, Self::Error>;
}

// TODO(human): Implement the in-memory storage backend
pub struct InMemoryStorage {
    // Storage implementation will go here
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            // Initialize storage
        }
    }
}