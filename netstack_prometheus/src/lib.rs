use netstack::monitoring::ServerMonitor;
use prometheus::{TextEncoder, Encoder};

lazy_static::lazy_static! {
    static ref TICKS: prometheus::IntCounter = prometheus::register_int_counter!("ticks", "Total ticks elapsed since the server was started").unwrap();
    static ref CONNECTED: prometheus::IntCounter = prometheus::register_int_counter!("connected", "total number of connected events").unwrap();
    static ref DISCONNECTED: prometheus::IntCounter = prometheus::register_int_counter!("disconnected", "total number of disconnected events").unwrap();
    static ref MESSAGES: prometheus::IntCounter = prometheus::register_int_counter!("messages", "total number of received messages").unwrap();
}

pub struct PrometheusMonitor;

impl PrometheusMonitor {
    pub fn new() -> Self {

        Self
    }

    pub fn render() -> Vec<u8> {
        let metric_families = prometheus::gather();

        let encoder = TextEncoder::new();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();

        buffer
    }
}

impl ServerMonitor for PrometheusMonitor {
    fn tick(&mut self) {
        TICKS.inc();
    }

    fn connected(&mut self) {
        CONNECTED.inc();
    }

    fn disconnected(&mut self) {
        DISCONNECTED.inc();
    }

    fn message(&mut self) {
        MESSAGES.inc();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
