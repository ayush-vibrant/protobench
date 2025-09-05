use shared::{MetricPoint as SharedMetricPoint, MetricQuery as SharedMetricQuery, MetricStatistics as SharedMetricStatistics};
use std::collections::HashMap;
use std::sync::OnceLock;
use tonic::transport::Channel;

pub mod metrics {
    tonic::include_proto!("protobench.metrics");
}

use metrics::{
    metrics_service_client::MetricsServiceClient, 
    Empty, MetricPoint, MetricQuery, MetricStatistics
};

static CLIENT: OnceLock<MetricsServiceClient<Channel>> = OnceLock::new();

async fn get_client() -> anyhow::Result<&'static MetricsServiceClient<Channel>> {
    if let Some(client) = CLIENT.get() {
        return Ok(client);
    }
    
    let channel = Channel::from_static("http://127.0.0.1:50051").connect().await?;
    let client = MetricsServiceClient::new(channel);
    
    CLIENT.set(client).map_err(|_| anyhow::anyhow!("Failed to set client"))?;
    Ok(CLIENT.get().unwrap())
}

pub async fn submit_metric(metric: SharedMetricPoint) -> anyhow::Result<()> {
    let mut client = get_client().await?.clone();
    
    // Convert shared metric to protobuf metric
    let proto_metric = MetricPoint {
        timestamp: metric.timestamp,
        hostname: metric.hostname,
        cpu_percent: metric.cpu_percent,
        memory_bytes: metric.memory_bytes,
        disk_io_ops: metric.disk_io_ops,
        tags: metric.tags,
    };
    
    let request = tonic::Request::new(proto_metric);
    client.submit_metric(request).await?;
    
    Ok(())
}

pub async fn query_metrics(query: SharedMetricQuery) -> anyhow::Result<Vec<SharedMetricPoint>> {
    let mut client = get_client().await?.clone();
    
    // Convert shared query to protobuf query
    let proto_query = MetricQuery {
        start_time: query.start_time,
        end_time: query.end_time,
        hostname_filter: query.hostname_filter,
    };
    
    let request = tonic::Request::new(proto_query);
    let mut stream = client.query_metrics(request).await?.into_inner();
    
    let mut metrics = Vec::new();
    while let Some(metric) = stream.message().await? {
        // Convert protobuf metric back to shared metric
        let shared_metric = SharedMetricPoint {
            timestamp: metric.timestamp,
            hostname: metric.hostname,
            cpu_percent: metric.cpu_percent,
            memory_bytes: metric.memory_bytes,
            disk_io_ops: metric.disk_io_ops,
            tags: metric.tags,
        };
        metrics.push(shared_metric);
    }
    
    Ok(metrics)
}

pub async fn get_statistics(query: SharedMetricQuery) -> anyhow::Result<SharedMetricStatistics> {
    let mut client = get_client().await?.clone();
    
    // Convert shared query to protobuf query
    let proto_query = MetricQuery {
        start_time: query.start_time,
        end_time: query.end_time,
        hostname_filter: query.hostname_filter,
    };
    
    let request = tonic::Request::new(proto_query);
    let response = client.get_statistics(request).await?;
    let stats = response.into_inner();
    
    // Convert protobuf statistics back to shared statistics
    let shared_stats = SharedMetricStatistics {
        count: stats.count,
        avg_cpu_percent: stats.avg_cpu_percent,
        avg_memory_bytes: stats.avg_memory_bytes,
        avg_disk_io_ops: stats.avg_disk_io_ops,
        time_range_seconds: stats.time_range_seconds,
    };
    
    Ok(shared_stats)
}