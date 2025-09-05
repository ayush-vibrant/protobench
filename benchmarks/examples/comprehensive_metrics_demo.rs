/// Example demonstrating comprehensive benchmark metrics collection
/// This shows how to use the new BenchmarkMetrics to measure:
/// - Latency
/// - Payload sizes (request + response)  
/// - Memory allocations
/// - CPU cycles (estimated)

use benchmarks::{
    generate_test_data, 
    rest_client, grpc_client, capnp_client,
    BenchmarkMetrics, PayloadSizes, PayloadMeasurement,
    payload_measurement, measure_memory, estimate_cpu_cycles
};
// Imports handled through benchmarks crate
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Comprehensive Protocol Metrics Demo");
    println!("===================================\n");
    
    let test_metric = generate_test_data(1)[0].clone();
    
    // Demonstrate payload size measurement for each protocol
    println!("📊 Payload Size Comparison:");
    println!("REST/JSON:     {} bytes", test_metric.measure_payload_size());
    println!("gRPC/Protobuf: {} bytes", payload_measurement::measure_grpc_metric_size(&test_metric));
    println!("Cap'n Proto:   {} bytes (estimated)", payload_measurement::measure_capnp_metric_size(&test_metric));
    println!();
    
    // Demonstrate comprehensive metrics collection for submit_metric
    println!("🚀 Submit Metric - Comprehensive Analysis:");
    println!();
    
    // REST submit with full metrics
    let rest_metrics = measure_submit_metric_comprehensive(
        "REST", 
        test_metric.measure_payload_size(),
        || rest_client::submit_metric(test_metric.clone())
    ).await?;
    
    print_comprehensive_metrics("REST", &rest_metrics);
    
    // gRPC submit with full metrics  
    let grpc_request_size = payload_measurement::measure_grpc_metric_size(&test_metric);
    let grpc_metrics = measure_submit_metric_comprehensive(
        "gRPC",
        grpc_request_size,
        || grpc_client::submit_metric(test_metric.clone())
    ).await?;
    
    print_comprehensive_metrics("gRPC", &grpc_metrics);
    
    // Cap'n Proto submit with full metrics
    let capnp_request_size = payload_measurement::measure_capnp_metric_size(&test_metric);
    let capnp_metrics = measure_submit_metric_comprehensive(
        "Cap'n Proto",
        capnp_request_size,
        || capnp_client::submit_metric(test_metric.clone())
    ).await?;
    
    print_comprehensive_metrics("Cap'n Proto", &capnp_metrics);
    
    println!("\n🔍 Efficiency Analysis:");
    analyze_efficiency(&[
        ("REST", &rest_metrics),
        ("gRPC", &grpc_metrics), 
        ("Cap'n Proto", &capnp_metrics)
    ]);
    
    Ok(())
}

/// Measure submit_metric operation with comprehensive metrics
async fn measure_submit_metric_comprehensive<F, Fut>(
    protocol: &str,
    request_size: usize, 
    f: F
) -> anyhow::Result<BenchmarkMetrics>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    println!("Measuring {} submit_metric...", protocol);
    
    let start_time = Instant::now();
    
    let (result, memory_allocated) = measure_memory(|| {
        tokio::runtime::Handle::current().block_on(f())
    });
    
    result?; // Propagate any errors
    
    let latency = start_time.elapsed();
    let cpu_cycles = estimate_cpu_cycles(latency);
    
    // For submit_metric, response is empty (just HTTP status)
    let payload_size = PayloadSizes::new(request_size, 0);
    
    Ok(BenchmarkMetrics {
        latency,
        payload_size,
        memory_allocated,
        cpu_cycles,
    })
}

/// Pretty print comprehensive metrics
fn print_comprehensive_metrics(protocol: &str, metrics: &BenchmarkMetrics) {
    println!("  {} Results:", protocol);
    println!("    ⏱️  Latency:        {:?}", metrics.latency);
    println!("    📦 Request Size:   {} bytes", metrics.payload_size.request_bytes);
    println!("    📥 Response Size:  {} bytes", metrics.payload_size.response_bytes);  
    println!("    📊 Total Traffic:  {} bytes", metrics.payload_size.total_bytes);
    println!("    🧠 Memory Used:    {} bytes", metrics.memory_allocated);
    println!("    ⚡ CPU Cycles:     {} (estimated)", metrics.cpu_cycles);
    println!("    💰 Cost Score:     {:.2} (lower is better)", calculate_cost_score(metrics));
    println!();
}

/// Calculate a composite "cost score" combining all metrics
fn calculate_cost_score(metrics: &BenchmarkMetrics) -> f64 {
    let latency_ms = metrics.latency.as_nanos() as f64 / 1_000_000.0;
    let memory_kb = metrics.memory_allocated as f64 / 1024.0;
    let traffic_kb = metrics.payload_size.total_bytes as f64 / 1024.0;
    let cpu_score = metrics.cpu_cycles as f64 / 1_000_000.0; // Normalize to millions
    
    // Weighted composite score (adjust weights based on your priorities)
    latency_ms * 0.4 + memory_kb * 0.2 + traffic_kb * 0.2 + cpu_score * 0.2
}

/// Analyze relative efficiency across protocols
fn analyze_efficiency(results: &[(&str, &BenchmarkMetrics)]) {
    if results.is_empty() { return; }
    
    let best_latency = results.iter().min_by_key(|(_, m)| m.latency).unwrap();
    let best_memory = results.iter().min_by_key(|(_, m)| m.memory_allocated).unwrap();
    let best_traffic = results.iter().min_by_key(|(_, m)| m.payload_size.total_bytes).unwrap();
    let best_overall = results.iter().min_by(|(_, a), (_, b)| 
        calculate_cost_score(a).partial_cmp(&calculate_cost_score(b)).unwrap()
    ).unwrap();
    
    println!("🏆 Winners:");
    println!("  Fastest:       {} ({:?})", best_latency.0, best_latency.1.latency);
    println!("  Least Memory:  {} ({} bytes)", best_memory.0, best_memory.1.memory_allocated);
    println!("  Least Traffic: {} ({} bytes)", best_traffic.0, best_traffic.1.payload_size.total_bytes);
    println!("  Best Overall:  {} (cost: {:.2})", best_overall.0, calculate_cost_score(best_overall.1));
}