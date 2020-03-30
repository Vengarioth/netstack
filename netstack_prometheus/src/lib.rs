use netstack::monitoring::Monitor;
use prometheus::{Opts, Registry, Counter, TextEncoder, Encoder, IntCounter, register_int_counter};

lazy_static::lazy_static! {
    static ref A_INT_COUNTER: IntCounter =
        register_int_counter!("A_int_counter", "foobar").unwrap();
}

pub struct PrometheusMonitor {
    registry: Registry,
    ticks: Counter,
}

impl PrometheusMonitor {
    pub fn new() -> Self {
        let registry = Registry::new();
        let ticks = Counter::with_opts(Opts::new("ticks", "Number of ticks since start")).unwrap();

        registry.register(Box::new(ticks.clone())).unwrap();

        Self {
            registry,
            ticks,
        }
    }

    pub fn render(&self) {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();

        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();

        println!("{}", String::from_utf8(buffer).unwrap());
    }
}

impl Monitor for PrometheusMonitor {
    fn tick(&mut self) {
        self.ticks.inc();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
