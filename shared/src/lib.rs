use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
    metrics: Arc<RwLock<Vec<MetricPoint>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn store_metric(&self, metric: MetricPoint) -> Result<(), anyhow::Error> {
        let mut metrics = self.metrics.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        metrics.push(metric);
        Ok(())
    }
    
    pub fn query_metrics(&self, query: &MetricQuery) -> Result<Vec<MetricPoint>, anyhow::Error> {
        let metrics = self.metrics.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let filtered: Vec<MetricPoint> = metrics
            .iter()
            .filter(|metric| {
                metric.timestamp >= query.start_time && metric.timestamp <= query.end_time
            })
            .filter(|metric| {
                query.hostname_filter.as_ref()
                    .map_or(true, |filter| &metric.hostname == filter)
            })
            .cloned()
            .collect();
            
        Ok(filtered)
    }
    
    pub fn calculate_statistics(&self, query: &MetricQuery) -> Result<MetricStatistics, anyhow::Error> {
        let metrics = self.query_metrics(query)?;
        
        if metrics.is_empty() {
            return Ok(MetricStatistics {
                count: 0,
                avg_cpu_percent: 0.0,
                avg_memory_bytes: 0,
                avg_disk_io_ops: 0.0,
                time_range_seconds: query.end_time - query.start_time,
            });
        }
        
        let count = metrics.len() as u64;
        let avg_cpu = metrics.iter().map(|m| m.cpu_percent).sum::<f32>() / count as f32;
        let avg_memory = metrics.iter().map(|m| m.memory_bytes).sum::<u64>() / count;
        let avg_disk_io = metrics.iter().map(|m| m.disk_io_ops as f32).sum::<f32>() / count as f32;
        
        Ok(MetricStatistics {
            count,
            avg_cpu_percent: avg_cpu,
            avg_memory_bytes: avg_memory,
            avg_disk_io_ops: avg_disk_io,
            time_range_seconds: query.end_time - query.start_time,
        })
    }
}