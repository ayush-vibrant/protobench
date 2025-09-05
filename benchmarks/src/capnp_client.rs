use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures_util::io::AsyncReadExt;
use shared::{MetricPoint as SharedMetricPoint, MetricQuery as SharedMetricQuery, MetricStatistics as SharedMetricStatistics};
use std::collections::HashMap;
use tokio::net::TcpStream;
use crate::metrics_capnp::metrics_service;

// Create a new client connection for each request
// This avoids the Send/Sync issues with static storage
async fn create_client() -> anyhow::Result<(metrics_service::Client, tokio::task::JoinHandle<()>)> {
    let stream = TcpStream::connect("127.0.0.1:55556").await?;
    let (reader, writer) = tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
    
    let rpc_network = Box::new(twoparty::VatNetwork::new(
        reader,
        writer,
        rpc_twoparty_capnp::Side::Client,
        Default::default(),
    ));
    
    let mut rpc_system = RpcSystem::new(rpc_network, None);
    let client: metrics_service::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
    
    // Spawn RPC system in background using LocalSet for !Send types
    let handle = tokio::task::spawn_local(async move {
        if let Err(e) = rpc_system.await {
            eprintln!("RPC system error: {}", e);
        }
    });
    
    Ok((client, handle))
}

pub async fn submit_metric(metric: SharedMetricPoint) -> anyhow::Result<()> {
    // Run in LocalSet since Cap'n Proto types are !Send
    tokio::task::LocalSet::new()
        .run_until(async {
            let (client, _handle) = create_client().await?;
            
            // Create a request builder
            let mut request = client.submit_metric_request();
            let mut metric_builder = request.get().init_metric();
            
            // Set basic fields
            metric_builder.set_timestamp(metric.timestamp);
            metric_builder.set_hostname((&metric.hostname[..]).into());
            metric_builder.set_cpu_percent(metric.cpu_percent);
            metric_builder.set_memory_bytes(metric.memory_bytes);
            metric_builder.set_disk_io_ops(metric.disk_io_ops);
            
            // Set tags
            let mut tags_builder = metric_builder.init_tags(metric.tags.len() as u32);
            for (i, (key, value)) in metric.tags.iter().enumerate() {
                let mut tag_builder = tags_builder.reborrow().get(i as u32);
                tag_builder.set_key((&key[..]).into());
                tag_builder.set_value((&value[..]).into());
            }
            
            let _response = request.send().promise.await?;
            Ok::<(), anyhow::Error>(())
        })
        .await
}

pub async fn query_metrics(query: SharedMetricQuery) -> anyhow::Result<Vec<SharedMetricPoint>> {
    // Run in LocalSet since Cap'n Proto types are !Send
    tokio::task::LocalSet::new()
        .run_until(async {
            let (client, _handle) = create_client().await?;
            
            // Create a query request
            let mut request = client.query_metrics_request();
            let mut query_builder = request.get().init_query();
            
            query_builder.set_start_time(query.start_time);
            query_builder.set_end_time(query.end_time);
            
            if let Some(hostname) = query.hostname_filter {
                query_builder.set_hostname_filter((&hostname[..]).into());
            }
            
            let response = request.send().promise.await?;
            let metrics_reader = response.get()?.get_metrics()?;
            
            let mut metrics = Vec::new();
            for metric_reader in metrics_reader.iter() {
                let tags_reader = metric_reader.get_tags()?;
                let mut tags = HashMap::new();
                
                for tag_reader in tags_reader.iter() {
                    let key = tag_reader.get_key()?.to_str()?.to_string();
                    let value = tag_reader.get_value()?.to_str()?.to_string();
                    tags.insert(key, value);
                }
                
                let shared_metric = SharedMetricPoint {
                    timestamp: metric_reader.get_timestamp(),
                    hostname: metric_reader.get_hostname()?.to_str()?.to_string(),
                    cpu_percent: metric_reader.get_cpu_percent(),
                    memory_bytes: metric_reader.get_memory_bytes(),
                    disk_io_ops: metric_reader.get_disk_io_ops(),
                    tags,
                };
                
                metrics.push(shared_metric);
            }
            
            Ok::<Vec<SharedMetricPoint>, anyhow::Error>(metrics)
        })
        .await
}

pub async fn get_statistics(query: SharedMetricQuery) -> anyhow::Result<SharedMetricStatistics> {
    // Run in LocalSet since Cap'n Proto types are !Send
    tokio::task::LocalSet::new()
        .run_until(async {
            let (client, _handle) = create_client().await?;
            
            // Create a statistics request
            let mut request = client.get_statistics_request();
            let mut query_builder = request.get().init_query();
            
            query_builder.set_start_time(query.start_time);
            query_builder.set_end_time(query.end_time);
            
            if let Some(hostname) = query.hostname_filter {
                query_builder.set_hostname_filter((&hostname[..]).into());
            }
            
            let response = request.send().promise.await?;
            let stats_reader = response.get()?.get_statistics()?;
            
            let shared_stats = SharedMetricStatistics {
                count: stats_reader.get_count(),
                avg_cpu_percent: stats_reader.get_avg_cpu_percent(),
                avg_memory_bytes: stats_reader.get_avg_memory_bytes(),
                avg_disk_io_ops: stats_reader.get_avg_disk_io_ops(),
                time_range_seconds: stats_reader.get_time_range_seconds(),
            };
            
            Ok::<SharedMetricStatistics, anyhow::Error>(shared_stats)
        })
        .await
}