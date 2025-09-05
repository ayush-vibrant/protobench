use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use shared::{InMemoryStorage, MetricPoint, MetricQuery, MetricStatistics};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct QueryParams {
    start_time: i64,
    end_time: i64,
    hostname_filter: Option<String>,
}

// Application dependency container - equivalent to Spring's @Autowired beans.
// Axum injects this into handlers via State(state) extractor, enabling shared
// access to storage across concurrent requests without cloning the backend.
struct AppState {
    storage: Arc<InMemoryStorage>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let storage = Arc::new(InMemoryStorage::new());
    let app_state = Arc::new(AppState { storage });

    let app = Router::new()
        .route("/metrics", post(submit_metric).get(query_metrics))
        .route("/statistics", get(get_statistics))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("REST service listening on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn submit_metric(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(metric): Json<MetricPoint>,
) -> Result<StatusCode, StatusCode> {
    match state.storage.store_metric(metric) {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn query_metrics(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Vec<MetricPoint>>, StatusCode> {
    let query = MetricQuery {
        start_time: params.start_time,
        end_time: params.end_time,
        hostname_filter: params.hostname_filter,
    };

    match state.storage.query_metrics(&query) {
        Ok(metrics) => Ok(Json(metrics)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_statistics(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<Json<MetricStatistics>, StatusCode> {
    let query = MetricQuery {
        start_time: params.start_time,
        end_time: params.end_time,
        hostname_filter: params.hostname_filter,
    };

    match state.storage.calculate_statistics(&query) {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
