use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;

pub struct MetricsWrapper {
    pub registry: Registry,
    pub http_requests: Family<(String, String), Counter>,
}

impl MetricsWrapper {
    pub fn build() -> Self {
        let mut registry = <Registry>::with_prefix("csearch");
        let http_requests = Family::<(String, String), Counter>::default();
        registry.register(
            "http_requests",
            "Number of HTTP requests received",
            Box::new(http_requests.clone()),
        );

        Self { registry, http_requests }
    }
}
