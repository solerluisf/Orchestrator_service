#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use orchestrator_service::core::application::health_aggregator::HealthAggregator;
    use orchestrator_service::core::domain::health::HealthSnapshot;
    use orchestrator_service::adapters::health::http_health_adapter::HttpHealthAdapter;
    use orchestrator_service::adapters::metrics::prometheus_adapter::PrometheusAdapter;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_health_aggregator_creation() {
        let health_port = Arc::new(HttpHealthAdapter::new(HashMap::new()));
        let snapshot = Arc::new(RwLock::new(HealthSnapshot::new()));
        let metrics = Arc::new(PrometheusAdapter::new());

        let aggregator = HealthAggregator::new(health_port, snapshot, metrics);
        let snap = aggregator.get_snapshot().await;
        assert_eq!(snap.services.len(), 0);
    }
}
