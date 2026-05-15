use std::time::Instant;

use crate::core::domain::events::TraceId;
use crate::core::ports::metrics_port::IMetricsPort;

pub struct TelemetryDecorator<'a> {
    metrics: &'a dyn IMetricsPort,
}

impl<'a> TelemetryDecorator<'a> {
    pub fn new(metrics: &'a dyn IMetricsPort) -> Self {
        Self { metrics }
    }

    pub async fn wrap<F, T, E>(&self, operation_name: &str, labels: &[(&str, &str)], f: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let start = Instant::now();

        let result = f.await;

        let duration_ms = start.elapsed().as_millis() as f64;

        let status = if result.is_ok() { "success" } else { "failure" };
        let mut all_labels = labels.to_vec();
        all_labels.push(("status", status));

        self.metrics
            .record_histogram(operation_name, &all_labels, duration_ms);
        self.metrics
            .record_counter(&format!("{}_total", operation_name), &all_labels, 1);

        result
    }

    pub fn start_span(&self, operation_name: &str) -> SpanGuard<'_> {
        SpanGuard {
            operation_name: operation_name.to_string(),
            start: Instant::now(),
            metrics: self.metrics,
        }
    }
}

pub struct SpanGuard<'a> {
    operation_name: String,
    start: Instant,
    metrics: &'a dyn IMetricsPort,
}

impl<'a> SpanGuard<'a> {
    pub fn finish(self, labels: &[(&str, &str)]) {
        let duration_ms = self.start.elapsed().as_millis() as f64;
        self.metrics
            .record_histogram(&self.operation_name, labels, duration_ms);
    }
}

pub struct TraceContext {
    pub trace_id: TraceId,
    pub correlation_id: String,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: TraceId::new(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
