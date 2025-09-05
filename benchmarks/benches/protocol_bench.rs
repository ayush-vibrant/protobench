use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use shared::MetricQuery;
use tokio::runtime::Runtime;

// Include the client modules
use benchmarks::{rest_client, grpc_client, capnp_client, generate_test_data};

/// Benchmark submit_metric operation across all protocols with single metric
fn benchmark_submit_single(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let test_metric = generate_test_data(1)[0].clone();
    
    let mut group = c.benchmark_group("submit_single");
    group.sample_size(100);
    
    // REST API
    group.bench_function("REST", |b| {
        b.iter(|| {
            rt.block_on(async {
                rest_client::submit_metric(black_box(test_metric.clone())).await.unwrap()
            })
        });
    });
    
    // gRPC
    group.bench_function("gRPC", |b| {
        b.iter(|| {
            rt.block_on(async {
                grpc_client::submit_metric(black_box(test_metric.clone())).await.unwrap()
            })
        });
    });
    
    // Cap'n Proto
    group.bench_function("CapnProto", |b| {
        b.iter(|| {
            rt.block_on(async {
                capnp_client::submit_metric(black_box(test_metric.clone())).await.unwrap()
            })
        });
    });
    
    group.finish();
}

/// Benchmark query_metrics operation across all protocols with single query
fn benchmark_query_single(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Setup: Populate data in all services
    let setup_metrics = generate_test_data(20);
    rt.block_on(async {
        for metric in &setup_metrics {
            // Populate all services with the same data
            let _ = rest_client::submit_metric(metric.clone()).await;
            let _ = grpc_client::submit_metric(metric.clone()).await;
            let _ = capnp_client::submit_metric(metric.clone()).await;
        }
    });
    
    let query = MetricQuery {
        start_time: setup_metrics.first().unwrap().timestamp - 100,
        end_time: setup_metrics.last().unwrap().timestamp + 100,
        hostname_filter: None,
    };
    
    let mut group = c.benchmark_group("query_single");
    group.sample_size(50);
    
    // REST API
    group.bench_function("REST", |b| {
        b.iter(|| {
            rt.block_on(async {
                rest_client::query_metrics(black_box(query.clone())).await.unwrap()
            })
        });
    });
    
    // gRPC
    group.bench_function("gRPC", |b| {
        b.iter(|| {
            rt.block_on(async {
                grpc_client::query_metrics(black_box(query.clone())).await.unwrap()
            })
        });
    });
    
    // Cap'n Proto
    group.bench_function("CapnProto", |b| {
        b.iter(|| {
            rt.block_on(async {
                capnp_client::query_metrics(black_box(query.clone())).await.unwrap()
            })
        });
    });
    
    group.finish();
}

/// Benchmark get_statistics operation across all protocols with single query
fn benchmark_statistics_single(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Setup: Use the same data as query benchmark
    let setup_metrics = generate_test_data(20);
    rt.block_on(async {
        for metric in &setup_metrics {
            let _ = rest_client::submit_metric(metric.clone()).await;
            let _ = grpc_client::submit_metric(metric.clone()).await;
            let _ = capnp_client::submit_metric(metric.clone()).await;
        }
    });
    
    let query = MetricQuery {
        start_time: setup_metrics.first().unwrap().timestamp - 100,
        end_time: setup_metrics.last().unwrap().timestamp + 100,
        hostname_filter: None,
    };
    
    let mut group = c.benchmark_group("statistics_single");
    group.sample_size(50);
    
    // REST API
    group.bench_function("REST", |b| {
        b.iter(|| {
            rt.block_on(async {
                rest_client::get_statistics(black_box(query.clone())).await.unwrap()
            })
        });
    });
    
    // gRPC
    group.bench_function("gRPC", |b| {
        b.iter(|| {
            rt.block_on(async {
                grpc_client::get_statistics(black_box(query.clone())).await.unwrap()
            })
        });
    });
    
    // Cap'n Proto
    group.bench_function("CapnProto", |b| {
        b.iter(|| {
            rt.block_on(async {
                capnp_client::get_statistics(black_box(query.clone())).await.unwrap()
            })
        });
    });
    
    group.finish();
}

/// Benchmark submit_metric operation with variable payload sizes across all protocols
fn benchmark_submit_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("submit_scaling");
    group.sample_size(30); // Smaller sample for scaling tests
    
    // Test different payload sizes
    for size in [1, 5, 10, 50].iter() {
        let test_metrics = generate_test_data(*size);
        
        // REST API scaling
        group.bench_with_input(BenchmarkId::new("REST", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    for metric in &test_metrics {
                        rest_client::submit_metric(black_box(metric.clone())).await.unwrap();
                    }
                })
            });
        });
        
        // gRPC scaling
        group.bench_with_input(BenchmarkId::new("gRPC", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    for metric in &test_metrics {
                        grpc_client::submit_metric(black_box(metric.clone())).await.unwrap();
                    }
                })
            });
        });
        
        // Cap'n Proto scaling
        group.bench_with_input(BenchmarkId::new("CapnProto", size), size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    for metric in &test_metrics {
                        capnp_client::submit_metric(black_box(metric.clone())).await.unwrap();
                    }
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark query_metrics operation with variable dataset sizes across all protocols
fn benchmark_query_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("query_scaling");
    group.sample_size(20); // Smaller sample for scaling tests
    
    // Test different dataset sizes
    for dataset_size in [10, 50, 100, 500].iter() {
        let setup_metrics = generate_test_data(*dataset_size);
        
        // Setup data for this scale test
        rt.block_on(async {
            for metric in &setup_metrics {
                let _ = rest_client::submit_metric(metric.clone()).await;
                let _ = grpc_client::submit_metric(metric.clone()).await;
                let _ = capnp_client::submit_metric(metric.clone()).await;
            }
        });
        
        let query = MetricQuery {
            start_time: setup_metrics.first().unwrap().timestamp - 100,
            end_time: setup_metrics.last().unwrap().timestamp + 100,
            hostname_filter: None,
        };
        
        // REST API scaling
        group.bench_with_input(BenchmarkId::new("REST", dataset_size), dataset_size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    rest_client::query_metrics(black_box(query.clone())).await.unwrap()
                })
            });
        });
        
        // gRPC scaling
        group.bench_with_input(BenchmarkId::new("gRPC", dataset_size), dataset_size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    grpc_client::query_metrics(black_box(query.clone())).await.unwrap()
                })
            });
        });
        
        // Cap'n Proto scaling
        group.bench_with_input(BenchmarkId::new("CapnProto", dataset_size), dataset_size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    capnp_client::query_metrics(black_box(query.clone())).await.unwrap()
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark get_statistics operation with variable dataset sizes across all protocols
fn benchmark_statistics_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("statistics_scaling");
    group.sample_size(20); // Smaller sample for scaling tests
    
    // Test different dataset sizes
    for dataset_size in [10, 50, 100, 500].iter() {
        let setup_metrics = generate_test_data(*dataset_size);
        
        // Setup data for this scale test
        rt.block_on(async {
            for metric in &setup_metrics {
                let _ = rest_client::submit_metric(metric.clone()).await;
                let _ = grpc_client::submit_metric(metric.clone()).await;
                let _ = capnp_client::submit_metric(metric.clone()).await;
            }
        });
        
        let query = MetricQuery {
            start_time: setup_metrics.first().unwrap().timestamp - 100,
            end_time: setup_metrics.last().unwrap().timestamp + 100,
            hostname_filter: None,
        };
        
        // REST API scaling
        group.bench_with_input(BenchmarkId::new("REST", dataset_size), dataset_size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    rest_client::get_statistics(black_box(query.clone())).await.unwrap()
                })
            });
        });
        
        // gRPC scaling
        group.bench_with_input(BenchmarkId::new("gRPC", dataset_size), dataset_size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    grpc_client::get_statistics(black_box(query.clone())).await.unwrap()
                })
            });
        });
        
        // Cap'n Proto scaling
        group.bench_with_input(BenchmarkId::new("CapnProto", dataset_size), dataset_size, |b, _| {
            b.iter(|| {
                rt.block_on(async {
                    capnp_client::get_statistics(black_box(query.clone())).await.unwrap()
                })
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_submit_single,
    benchmark_query_single,
    benchmark_statistics_single,
    benchmark_submit_scaling,
    benchmark_query_scaling,
    benchmark_statistics_scaling
);
criterion_main!(benches);