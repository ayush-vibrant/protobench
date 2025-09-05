use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use shared::{InMemoryStorage, MetricPoint as SharedMetricPoint, MetricQuery as SharedMetricQuery};

pub mod metrics {
    tonic::include_proto!("protobench.metrics");
}

use metrics::{
    metrics_service_server::{MetricsService, MetricsServiceServer},
    Empty, MetricPoint, MetricQuery, MetricStatistics,
};

pub struct MetricsServiceImpl {
    storage: Arc<InMemoryStorage>,
}

impl MetricsServiceImpl {
    pub fn new(storage: Arc<InMemoryStorage>) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceImpl {
    async fn submit_metric(
        &self,
        request: Request<MetricPoint>,
    ) -> Result<Response<Empty>, Status> {
        let metric = request.into_inner();
        
        // Convert protobuf MetricPoint to shared MetricPoint
        let shared_metric = SharedMetricPoint {
            timestamp: metric.timestamp,
            hostname: metric.hostname,
            cpu_percent: metric.cpu_percent,
            memory_bytes: metric.memory_bytes,
            disk_io_ops: metric.disk_io_ops,
            tags: metric.tags,
        };

        match self.storage.store_metric(shared_metric) {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(_) => Err(Status::internal("Failed to store metric")),
        }
    }

    type QueryMetricsStream = 
        tokio_stream::wrappers::ReceiverStream<Result<MetricPoint, Status>>;

    async fn query_metrics(
        &self,
        request: Request<MetricQuery>,
    ) -> Result<Response<Self::QueryMetricsStream>, Status> {
        let query = request.into_inner();
        
        // Convert protobuf query to shared query
        let shared_query = SharedMetricQuery {
            start_time: query.start_time,
            end_time: query.end_time,
            hostname_filter: query.hostname_filter,
        };

        let metrics = self.storage.query_metrics(&shared_query)
            .map_err(|_| Status::internal("Failed to query metrics"))?;

        let (tx, rx) = tokio::sync::mpsc::channel(128);
        
        tokio::spawn(async move {
            for metric in metrics {
                // Convert shared MetricPoint to protobuf MetricPoint
                let proto_metric = MetricPoint {
                    timestamp: metric.timestamp,
                    hostname: metric.hostname,
                    cpu_percent: metric.cpu_percent,
                    memory_bytes: metric.memory_bytes,
                    disk_io_ops: metric.disk_io_ops,
                    tags: metric.tags,
                };
                
                if tx.send(Ok(proto_metric)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn get_statistics(
        &self,
        request: Request<MetricQuery>,
    ) -> Result<Response<MetricStatistics>, Status> {
        let query = request.into_inner();
        
        // Convert protobuf query to shared query
        let shared_query = SharedMetricQuery {
            start_time: query.start_time,
            end_time: query.end_time,
            hostname_filter: query.hostname_filter,
        };

        let stats = self.storage.calculate_statistics(&shared_query)
            .map_err(|_| Status::internal("Failed to calculate statistics"))?;

        // Convert shared statistics to protobuf statistics
        let proto_stats = MetricStatistics {
            count: stats.count,
            avg_cpu_percent: stats.avg_cpu_percent,
            avg_memory_bytes: stats.avg_memory_bytes,
            avg_disk_io_ops: stats.avg_disk_io_ops,
            time_range_seconds: stats.time_range_seconds,
        };

        Ok(Response::new(proto_stats))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let storage = Arc::new(InMemoryStorage::new());
    let service = MetricsServiceImpl::new(storage);

    let addr = "127.0.0.1:50051".parse()?;
    println!("gRPC service listening on {}", addr);

    Server::builder()
        .add_service(MetricsServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
