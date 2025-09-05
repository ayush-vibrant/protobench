use std::sync::Arc;
use capnp::capability::Promise;
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};
use shared::{InMemoryStorage, MetricPoint as SharedMetricPoint, MetricQuery as SharedMetricQuery};
use std::collections::HashMap;
use futures_util::io::AsyncReadExt;

pub mod metrics_capnp {
    include!(concat!(env!("OUT_DIR"), "/metrics_capnp.rs"));
}

use metrics_capnp::metrics_service;

struct MetricsServiceImpl {
    storage: Arc<InMemoryStorage>,
}

impl MetricsServiceImpl {
    fn new(storage: Arc<InMemoryStorage>) -> Self {
        Self { storage }
    }
}

impl metrics_service::Server for MetricsServiceImpl {
    fn submit_metric(
        &mut self,
        params: metrics_service::SubmitMetricParams,
        mut _results: metrics_service::SubmitMetricResults,
    ) -> Promise<(), capnp::Error> {
        let metric_reader = pry!(pry!(params.get()).get_metric());
        
        // Convert Cap'n Proto MetricPoint to shared MetricPoint  
        let tags_reader = pry!(metric_reader.get_tags());
        let mut tags = HashMap::new();
        
        for tag in tags_reader.iter() {
            let key = pry!(pry!(tag.get_key()).to_str()).to_string();
            let value = pry!(pry!(tag.get_value()).to_str()).to_string();
            tags.insert(key, value);
        }
        
        let shared_metric = SharedMetricPoint {
            timestamp: metric_reader.get_timestamp(),
            hostname: pry!(pry!(metric_reader.get_hostname()).to_str()).to_string(),
            cpu_percent: metric_reader.get_cpu_percent(),
            memory_bytes: metric_reader.get_memory_bytes(),
            disk_io_ops: metric_reader.get_disk_io_ops(),
            tags,
        };

        match self.storage.store_metric(shared_metric) {
            Ok(_) => Promise::ok(()),
            Err(_) => Promise::err(capnp::Error::failed("Failed to store metric".to_string())),
        }
    }

    fn query_metrics(
        &mut self,
        params: metrics_service::QueryMetricsParams,
        mut results: metrics_service::QueryMetricsResults,
    ) -> Promise<(), capnp::Error> {
        let query_reader = pry!(pry!(params.get()).get_query());
        
        let hostname_filter = if query_reader.has_hostname_filter() {
            Some(pry!(pry!(query_reader.get_hostname_filter()).to_str()).to_string())
        } else {
            None
        };
        
        let shared_query = SharedMetricQuery {
            start_time: query_reader.get_start_time(),
            end_time: query_reader.get_end_time(),
            hostname_filter,
        };

        let metrics = match self.storage.query_metrics(&shared_query) {
            Ok(metrics) => metrics,
            Err(_) => return Promise::err(capnp::Error::failed("Failed to query metrics".to_string())),
        };

        let mut results_builder = results.get().init_metrics(metrics.len() as u32);
        
        for (i, metric) in metrics.iter().enumerate() {
            let mut metric_builder = results_builder.reborrow().get(i as u32);
            metric_builder.set_timestamp(metric.timestamp);
            metric_builder.set_hostname((&metric.hostname[..]).into());
            metric_builder.set_cpu_percent(metric.cpu_percent);
            metric_builder.set_memory_bytes(metric.memory_bytes);
            metric_builder.set_disk_io_ops(metric.disk_io_ops);
            
            let mut tags_builder = metric_builder.init_tags(metric.tags.len() as u32);
            for (j, (key, value)) in metric.tags.iter().enumerate() {
                let mut tag_builder = tags_builder.reborrow().get(j as u32);
                tag_builder.set_key((&key[..]).into());
                tag_builder.set_value((&value[..]).into());
            }
        }

        Promise::ok(())
    }

    fn get_statistics(
        &mut self,
        params: metrics_service::GetStatisticsParams,
        mut results: metrics_service::GetStatisticsResults,
    ) -> Promise<(), capnp::Error> {
        let query_reader = pry!(pry!(params.get()).get_query());
        
        let hostname_filter = if query_reader.has_hostname_filter() {
            Some(pry!(pry!(query_reader.get_hostname_filter()).to_str()).to_string())
        } else {
            None
        };
        
        let shared_query = SharedMetricQuery {
            start_time: query_reader.get_start_time(),
            end_time: query_reader.get_end_time(),
            hostname_filter,
        };

        let stats = match self.storage.calculate_statistics(&shared_query) {
            Ok(stats) => stats,
            Err(_) => return Promise::err(capnp::Error::failed("Failed to calculate statistics".to_string())),
        };

        let mut stats_builder = results.get().init_statistics();
        stats_builder.set_count(stats.count);
        stats_builder.set_avg_cpu_percent(stats.avg_cpu_percent);
        stats_builder.set_avg_memory_bytes(stats.avg_memory_bytes);
        stats_builder.set_avg_disk_io_ops(stats.avg_disk_io_ops);
        stats_builder.set_time_range_seconds(stats.time_range_seconds);

        Promise::ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:55556";
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Cap'n Proto service listening on {}", addr);

    let storage = Arc::new(InMemoryStorage::new());

    // Use LocalSet for concurrent connections since RpcSystem is !Send
    tokio::task::LocalSet::new()
        .run_until(async move {
            loop {
                let (stream, client_addr) = listener.accept().await?;
                println!("Cap'n Proto client connected from {}", client_addr);
                
                let storage_clone = storage.clone();
                
                // Use spawn_local since RpcSystem doesn't implement Send
                tokio::task::spawn_local(async move {
                    let (reader, writer) = tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
                    let rpc_network = Box::new(twoparty::VatNetwork::new(
                        reader,
                        writer,
                        rpc_twoparty_capnp::Side::Server,
                        Default::default(),
                    ));

                    let service_impl = MetricsServiceImpl::new(storage_clone);
                    let metrics_service: metrics_service::Client = capnp_rpc::new_client(service_impl);
                    let rpc_system = RpcSystem::new(rpc_network, Some(metrics_service.clone().client));

                    if let Err(e) = rpc_system.await {
                        eprintln!("RPC system error: {}", e);
                    }
                });
            }
        })
        .await
}
