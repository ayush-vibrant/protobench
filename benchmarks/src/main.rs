use benchmarks::{generate_test_data, rest_client, grpc_client, capnp_client};
use shared::MetricQuery;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("ProtoBench - Protocol Performance Comparison");
    println!("===========================================");
    
    // Run basic functionality tests
    test_protocols().await?;
    
    println!("\nProtocols working correctly!");
    println!("Run 'cargo bench' to execute performance benchmarks.");
    
    Ok(())
}

async fn test_protocols() -> anyhow::Result<()> {
    let test_metric = generate_test_data(1)[0].clone();
    
    println!("Testing REST API...");
    match rest_client::submit_metric(test_metric.clone()).await {
        Ok(()) => println!("✅ REST API metric submitted successfully!"),
        Err(e) => println!("❌ REST API failed: {}", e),
    }
    
    println!("Testing gRPC...");  
    match grpc_client::submit_metric(test_metric.clone()).await {
        Ok(()) => println!("✅ gRPC metric submitted successfully!"),
        Err(e) => println!("❌ gRPC failed: {}", e),
    }
    
    println!("Testing Cap'n Proto...");
    match capnp_client::submit_metric(test_metric.clone()).await {
        Ok(()) => println!("✅ Cap'n Proto metric submitted successfully!"),
        Err(e) => println!("❌ Cap'n Proto failed: {}", e),
    }
    
    // Test query functionality
    let query = MetricQuery {
        start_time: test_metric.timestamp - 3600,
        end_time: test_metric.timestamp + 3600,
        hostname_filter: Some(test_metric.hostname.clone()),
    };
    
    println!("\nTesting query operations...");
    
    match capnp_client::query_metrics(query.clone()).await {
        Ok(metrics) => println!("✅ Cap'n Proto query: {} metrics retrieved", metrics.len()),
        Err(e) => println!("❌ Cap'n Proto query failed: {}", e),
    }
    
    match capnp_client::get_statistics(query).await {
        Ok(stats) => println!("✅ Cap'n Proto stats: count={}, avg_cpu={}%", stats.count, stats.avg_cpu_percent),
        Err(e) => println!("❌ Cap'n Proto statistics failed: {}", e),
    }
    
    Ok(())
}

