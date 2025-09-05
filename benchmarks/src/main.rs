use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rand::prelude::*;
use rand::rngs::StdRng;
use shared::{MetricPoint, MetricQuery, MetricStatistics};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;

mod rest_client;
mod grpc_client;
mod capnp_client;

fn generate_test_data(count: usize) -> Vec<MetricPoint> {
    let mut rng = StdRng::seed_from_u64(42); // Deterministic for consistent benchmarks
    let mut metrics = Vec::with_capacity(count);
    
    let hostnames = vec![
        "web-01", "web-02", "db-primary", "db-replica", "cache-01", 
        "api-gateway", "worker-01", "worker-02", "monitoring", "load-balancer"
    ];
    
    let environments = vec!["prod", "staging", "dev"];
    let regions = vec!["us-east", "us-west", "eu-central", "ap-southeast"];
    let services = vec!["frontend", "backend", "database", "cache", "queue"];
    
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

fn benchmark_submit_metrics(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let test_data = generate_test_data(100);
    
    let mut group = c.benchmark_group("submit_metrics");
    
    // Benchmark REST API
    group.bench_with_input(BenchmarkId::new("REST", 100), &test_data, |b, data| {
        b.to_async(&rt).iter(|| async {
            for metric in data {
                rest_client::submit_metric(black_box(metric.clone())).await.unwrap();
            }
        });
    });
    
    // Benchmark gRPC
    group.bench_with_input(BenchmarkId::new("gRPC", 100), &test_data, |b, data| {
        b.to_async(&rt).iter(|| async {
            for metric in data {
                grpc_client::submit_metric(black_box(metric.clone())).await.unwrap();
            }
        });
    });
    
    // Benchmark Cap'n Proto
    group.bench_with_input(BenchmarkId::new("CapnProto", 100), &test_data, |b, data| {
        b.to_async(&rt).iter(|| async {
            for metric in data {
                capnp_client::submit_metric(black_box(metric.clone())).await.unwrap();
            }
        });
    });
    
    group.finish();
}

fn benchmark_query_metrics(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let query = MetricQuery {
        start_time: 1000000000,
        end_time: 2000000000,
        hostname_filter: None,
    };
    
    let mut group = c.benchmark_group("query_metrics");
    
    group.bench_function("REST", |b| {
        b.to_async(&rt).iter(|| async {
            rest_client::query_metrics(black_box(query.clone())).await.unwrap()
        });
    });
    
    group.bench_function("gRPC", |b| {
        b.to_async(&rt).iter(|| async {
            grpc_client::query_metrics(black_box(query.clone())).await.unwrap()
        });
    });
    
    group.bench_function("CapnProto", |b| {
        b.to_async(&rt).iter(|| async {
            capnp_client::query_metrics(black_box(query.clone())).await.unwrap()
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_submit_metrics, benchmark_query_metrics);
criterion_main!(benches);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("ProtoBench - Protocol Performance Comparison");
    println!("===========================================");
    
    // Run basic functionality tests
    test_all_protocols().await?;
    
    println!("\nAll protocols working correctly!");
    println!("Run 'cargo bench' to execute performance benchmarks.");
    
    Ok(())
}

async fn test_all_protocols() -> anyhow::Result<()> {
    let test_metric = generate_test_data(1)[0].clone();
    
    println!("Testing REST API...");
    rest_client::submit_metric(test_metric.clone()).await?;
    
    println!("Testing gRPC...");  
    grpc_client::submit_metric(test_metric.clone()).await?;
    
    println!("Testing Cap'n Proto...");
    capnp_client::submit_metric(test_metric).await?;
    
    Ok(())
}
