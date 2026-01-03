use opentelemetry::metrics::{Counter, Histogram, Meter};

pub struct EngineMetrics {
    pub heartbeat: Counter<u64>,
    pub loop_duration: Histogram<f64>,
    pub signal_processed: Counter<u64>,
    pub risk_vetos: Counter<u64>,
    pub market_price: Histogram<f64>, // Using Histogram for price distribution/logging
    pub market_velocity: Histogram<f64>,
}

impl EngineMetrics {
    pub fn new(meter: &Meter) -> Self {
        EngineMetrics {
            heartbeat: meter
                .u64_counter("reflex_heartbeat")
                .with_description("Engine Liveness Heartbeat")
                .init(),
            loop_duration: meter
                .f64_histogram("reflex_loop_duration_ms")
                .with_description("Duration of the main simulation loop")
                .init(),
            signal_processed: meter
                .u64_counter("reflex_signals_processed")
                .with_description("Number of signals processed from Brain")
                .init(),
            risk_vetos: meter
                .u64_counter("reflex_risk_vetos")
                .with_description("Number of trades vetoed by Risk Guardian")
                .init(),
            market_price: meter
                .f64_histogram("reflex_market_price")
                .with_description("Current Market Price (Simulated)")
                .init(),
            market_velocity: meter
                .f64_histogram("reflex_market_velocity")
                .with_description("Current Market Velocity")
                .init(),
        }
    }
}
