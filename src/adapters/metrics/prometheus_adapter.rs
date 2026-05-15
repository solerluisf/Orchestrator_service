use prometheus::{Encoder, Gauge, Histogram, HistogramOpts, IntCounter, Registry, TextEncoder};
use std::sync::Arc;

use crate::core::ports::metrics_port::IMetricsPort;

pub struct PrometheusAdapter {
    registry: Registry,
    counters: Arc<dashmap::DashMap<String, IntCounter>>,
    histograms: Arc<dashmap::DashMap<String, Histogram>>,
    gauges: Arc<dashmap::DashMap<String, Gauge>>,
}

impl PrometheusAdapter {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            counters: Arc::new(dashmap::DashMap::new()),
            histograms: Arc::new(dashmap::DashMap::new()),
            gauges: Arc::new(dashmap::DashMap::new()),
        }
    }

    fn make_key(name: &str, labels: &[(&str, &str)]) -> String {
        let label_str: Vec<String> = labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        format!("{}[{}]", name, label_str.join(","))
    }

    pub fn export_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl IMetricsPort for PrometheusAdapter {
    fn record_counter(&self, name: &str, labels: &[(&str, &str)], value: u64) {
        let key = Self::make_key(name, labels);
        let counter = self
            .counters
            .entry(key)
            .or_insert_with(|| {
                let _label_names: Vec<String> = labels.iter().map(|(k, _)| k.to_string()).collect();
                IntCounter::new(name, name).unwrap()
            });
        counter.inc_by(value);
    }

    fn record_histogram(&self, name: &str, labels: &[(&str, &str)], value_ms: f64) {
        let key = Self::make_key(name, labels);
        let histogram = self
            .histograms
            .entry(key)
            .or_insert_with(|| {
                Histogram::with_opts(HistogramOpts::new(name, name)).unwrap()
            });
        histogram.observe(value_ms);
    }

    fn record_gauge(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        let key = Self::make_key(name, labels);
        let gauge = self
            .gauges
            .entry(key)
            .or_insert_with(|| {
                Gauge::new(name, name).unwrap()
            });
        gauge.set(value);
    }
}
