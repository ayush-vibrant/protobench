use reqwest::Client;
use shared::{MetricPoint, MetricQuery, MetricStatistics};
use std::sync::OnceLock;

static CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    CLIENT.get_or_init(|| Client::new())
}

pub async fn submit_metric(metric: MetricPoint) -> anyhow::Result<()> {
    let client = get_client();
    let response = client
        .post("http://127.0.0.1:3000/metrics")
        .json(&metric)
        .send()
        .await?;
    
    if !response.status().is_success() {
        anyhow::bail!("REST submit failed: {}", response.status());
    }
    
    Ok(())
}

pub async fn query_metrics(query: MetricQuery) -> anyhow::Result<Vec<MetricPoint>> {
    let client = get_client();
    let mut url = "http://127.0.0.1:3000/metrics".to_string();
    url.push_str(&format!("?start_time={}&end_time={}", query.start_time, query.end_time));
    
    if let Some(hostname) = query.hostname_filter {
        url.push_str(&format!("&hostname_filter={}", hostname));
    }
    
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("REST query failed: {}", response.status());
    }
    
    let metrics: Vec<MetricPoint> = response.json().await?;
    Ok(metrics)
}

pub async fn get_statistics(query: MetricQuery) -> anyhow::Result<MetricStatistics> {
    let client = get_client();
    let mut url = "http://127.0.0.1:3000/statistics".to_string();
    url.push_str(&format!("?start_time={}&end_time={}", query.start_time, query.end_time));
    
    if let Some(hostname) = query.hostname_filter {
        url.push_str(&format!("&hostname_filter={}", hostname));
    }
    
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("REST statistics failed: {}", response.status());
    }
    
    let stats: MetricStatistics = response.json().await?;
    Ok(stats)
}