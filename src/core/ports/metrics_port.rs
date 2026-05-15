pub trait IMetricsPort: Send + Sync {
    fn record_counter(&self, name: &str, labels: &[(&str, &str)], value: u64);
    fn record_histogram(&self, name: &str, labels: &[(&str, &str)], value_ms: f64);
    fn record_gauge(&self, name: &str, labels: &[(&str, &str)], value: f64);
}
