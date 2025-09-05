use rand::prelude::*;
use rand::rngs::StdRng;
use shared::MetricPoint;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use stats_alloc::{StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

// Use instrumented allocator for memory tracking
#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

// Generated Cap'n Proto code
#[allow(clippy::needless_lifetimes)]
pub mod metrics_capnp {
    include!(concat!(env!("OUT_DIR"), "/metrics_capnp.rs"));
}

pub mod rest_client;
pub mod grpc_client;
pub mod capnp_client;

/// Comprehensive performance metrics for benchmarking
#[derive(Debug, Clone)]
pub struct BenchmarkMetrics {
    pub latency: Duration,           // Time taken for operation
    pub payload_size: PayloadSizes,  // Bytes sent/received
    pub memory_allocated: usize,     // Heap allocations during operation
    pub cpu_cycles: u64,             // CPU cycles (approximated via timing)
}

/// Payload size breakdown for request and response
#[derive(Debug, Clone)]
pub struct PayloadSizes {
    pub request_bytes: usize,     // Serialized request size
    pub response_bytes: usize,    // Serialized response size
    pub total_bytes: usize,       // Total network traffic
}

impl PayloadSizes {
    pub fn new(request_bytes: usize, response_bytes: usize) -> Self {
        Self {
            request_bytes,
            response_bytes,
            total_bytes: request_bytes + response_bytes,
        }
    }
}

/// Measure memory allocations during a closure execution
pub fn measure_memory<T, F>(f: F) -> (T, usize)
where
    F: FnOnce() -> T,
{
    let start_stats = GLOBAL.stats();
    let result = f();
    let end_stats = GLOBAL.stats();
    
    let bytes_allocated = end_stats.bytes_allocated - start_stats.bytes_allocated;
    (result, bytes_allocated)
}

/// Estimate CPU cycles based on high-resolution timing
/// Note: This is an approximation since we can't directly count CPU cycles
pub fn estimate_cpu_cycles(duration: Duration) -> u64 {
    // Rough approximation: assume 3 GHz CPU, convert nanoseconds to cycles
    const APPROXIMATE_CPU_HZ: u64 = 3_000_000_000;
    (duration.as_nanos() as u64 * APPROXIMATE_CPU_HZ) / 1_000_000_000
}

/// Measure payload sizes for different serialization formats
pub trait PayloadMeasurement {
    fn measure_payload_size(&self) -> usize;
}

impl PayloadMeasurement for shared::MetricPoint {
    fn measure_payload_size(&self) -> usize {
        // JSON size (what REST uses)
        serde_json::to_vec(self).map(|v| v.len()).unwrap_or(0)
    }
}

impl PayloadMeasurement for shared::MetricQuery {
    fn measure_payload_size(&self) -> usize {
        serde_json::to_vec(self).map(|v| v.len()).unwrap_or(0)
    }
}

impl PayloadMeasurement for Vec<shared::MetricPoint> {
    fn measure_payload_size(&self) -> usize {
        serde_json::to_vec(self).map(|v| v.len()).unwrap_or(0)
    }
}

impl PayloadMeasurement for shared::MetricStatistics {
    fn measure_payload_size(&self) -> usize {
        serde_json::to_vec(self).map(|v| v.len()).unwrap_or(0)
    }
}

/// Helper functions for measuring protocol-specific payload sizes
pub mod payload_measurement {
    use prost::Message;

    /// Measure gRPC protobuf payload size
    pub fn measure_grpc_metric_size(metric: &shared::MetricPoint) -> usize {
        let proto_metric = crate::grpc_client::metrics::MetricPoint {
            timestamp: metric.timestamp,
            hostname: metric.hostname.clone(),
            cpu_percent: metric.cpu_percent,
            memory_bytes: metric.memory_bytes,
            disk_io_ops: metric.disk_io_ops,
            tags: metric.tags.clone(),
        };
        proto_metric.encoded_len()
    }

    /// Measure gRPC protobuf query size
    pub fn measure_grpc_query_size(query: &shared::MetricQuery) -> usize {
        let proto_query = crate::grpc_client::metrics::MetricQuery {
            start_time: query.start_time,
            end_time: query.end_time,
            hostname_filter: query.hostname_filter.clone(),
        };
        proto_query.encoded_len()
    }

    /// Measure Cap'n Proto payload size (estimated based on schema)
    pub fn measure_capnp_metric_size(metric: &shared::MetricPoint) -> usize {
        // Cap'n Proto has fixed overhead + variable string lengths
        // Fixed: 8+4+8+4 = 24 bytes for primitives
        // Variable: strings + tags
        let hostname_len = metric.hostname.len();
        let tags_len: usize = metric.tags.iter()
            .map(|(k, v)| k.len() + v.len() + 8) // 8 bytes overhead per tag
            .sum();
        24 + hostname_len + tags_len + 32 // 32 bytes Cap'n Proto overhead
    }

    /// Measure Cap'n Proto query size
    pub fn measure_capnp_query_size(query: &shared::MetricQuery) -> usize {
        let hostname_len = query.hostname_filter.as_ref().map(|s| s.len()).unwrap_or(0);
        16 + hostname_len + 16 // timestamps + optional hostname + overhead
    }
}

/// Comprehensive benchmark wrapper that measures all metrics
pub async fn benchmark_operation<T, F, Fut>(
    _operation_name: &str,
    request_payload_size: usize,
    f: F,
) -> (T, BenchmarkMetrics)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
    T: PayloadMeasurement,
{
    let start_time = Instant::now();
    
    let (result, memory_allocated) = measure_memory(|| {
        tokio::runtime::Handle::current().block_on(f())
    });
    
    let latency = start_time.elapsed();
    let cpu_cycles = estimate_cpu_cycles(latency);
    
    let response_payload_size = result.measure_payload_size();
    let payload_size = PayloadSizes::new(request_payload_size, response_payload_size);
    
    let metrics = BenchmarkMetrics {
        latency,
        payload_size,
        memory_allocated,
        cpu_cycles,
    };
    
    (result, metrics)
}

pub fn generate_test_data(count: usize) -> Vec<MetricPoint> {
    let mut rng = StdRng::seed_from_u64(42); // Deterministic for consistent benchmarks
    let mut metrics = Vec::with_capacity(count);
    
    let hostnames = [
        "web-01", "web-02", "db-primary", "db-replica", "cache-01", 
        "api-gateway", "worker-01", "worker-02", "monitoring", "load-balancer"
    ];
    
    let environments = ["prod", "staging", "dev"];
    let regions = ["us-east", "us-west", "eu-central", "ap-southeast"];
    let services = ["frontend", "backend", "database", "cache", "queue"];
    
    let base_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    for i in 0..count {
        let mut tags = HashMap::new();
        tags.insert("env".to_string(), environments.choose(&mut rng).unwrap().to_string());
        tags.insert("region".to_string(), regions.choose(&mut rng).unwrap().to_string());
        tags.insert("service".to_string(), services.choose(&mut rng).unwrap().to_string());
        tags.insert("version".to_string(), format!("v{}.{}.{}", 
            rng.gen_range(1..3), rng.gen_range(0..10), rng.gen_range(0..5)));
        
        let metric = MetricPoint {
            timestamp: base_timestamp - rng.gen_range(0..3600) + (i as i64), // Spread over last hour
            hostname: hostnames.choose(&mut rng).unwrap().to_string(),
            cpu_percent: rng.gen_range(5.0..95.0), // Realistic CPU usage
            memory_bytes: rng.gen_range(1_000_000_000..16_000_000_000), // 1GB to 16GB
            disk_io_ops: rng.gen_range(100..10_000), // Reasonable I/O operations
            tags,
        };
        
        metrics.push(metric);
    }
    
    metrics
}