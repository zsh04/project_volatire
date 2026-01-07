use crate::feynman::PhysicsEngine;
use crate::taleb::{RiskGuardian, TradeProposal, RiskVerdict};
use crate::ledger::AccountState;
use crate::sim::ticker::SimTicker;
use opentelemetry::{global, KeyValue};
use opentelemetry::metrics::{Counter, UpDownCounter};
use futures_util::StreamExt;
use std::time::Instant;
use rand::Rng; // Added for Jitter

// D-101: FIFO Queue State
struct OrderState {
    id: String,
    side: String,
    qty: f64,
    price: f64,
    queue_pos: f64, // Volume ahead of us
    placed_at_ts: f64, // When it enters the book (after latency)
}

pub struct SimulationEngine {
    physics: PhysicsEngine,
    guardian: RiskGuardian,
    ledger: AccountState,
    ticker: SimTicker,
    auditor: crate::audit::QuestBridge,
    // D-101: Sim Hardening Flags
    pub pessimistic: bool, 
    pending_orders: Vec<OrderState>,
    // Metrics
    signal_counter: Counter<u64>,
    trade_counter: Counter<u64>,
    nav_gauge: UpDownCounter<f64>,
}

impl SimulationEngine {
    pub async fn new(db_url: &str, auditor: crate::audit::QuestBridge) -> Result<Self, Box<dyn std::error::Error>> {
        let meter = global::meter("voltaire.reflex.sim");
        let signal_counter = meter.u64_counter("alpha.signal.count").init();
        let trade_counter = meter.u64_counter("alpha.trade.count").init();
        let nav_gauge = meter.f64_up_down_counter("portfolio.nav").init();

        let ticker = SimTicker::new(db_url).await?; 

        Ok(Self {
            physics: PhysicsEngine::new(2000), 
            guardian: RiskGuardian::new(),
            ledger: AccountState::new(100_000.0, 0.0), 
            ticker,
            auditor,
            pessimistic: false, // Default to Optimistic
            pending_orders: Vec::new(),
            signal_counter,
            trade_counter,
            nav_gauge,
        })

    }

    pub fn set_pessimistic(&mut self, enabled: bool) {
        self.pessimistic = enabled;
        println!("⚙️ Simulation Mode: {}", if self.pessimistic { "PESSIMISTIC (FIFO + Latency)" } else { "OPTIMISTIC (Instant Fill)" });
    }

    pub async fn run(mut self, start_ts: i64, end_ts: i64, speed: f64) -> Result<f64, Box<dyn std::error::Error>> {
        println!("🚀 Starting SHADOW SIMULATION: {} to {} (Speed: {:.1}x)", start_ts, end_ts, speed);
        
        // Sim State
        let mut sim_stream = self.ticker.stream_history("BTC-USDT", start_ts, end_ts).await?;
        let mut count = 0;
        let start_time = Instant::now();
        let mut _last_price = 0.0; 
        
        let sim_start_ms = start_ts as f64; 

        while let Some(tick_result) = sim_stream.next().await {
            match tick_result {
                Ok(tick) => {
                    count += 1;
                    _last_price = tick.price;
                    
                    // --- D-25B: Speed Control ---
                    if speed > 0.0 {
                        let sim_elapsed = tick.timestamp - sim_start_ms;
                        let real_elapsed_micros = start_time.elapsed().as_micros() as f64;
                        let target_real_elapsed = (sim_elapsed * 1000.0) / speed; // ms -> us
                        
                        if target_real_elapsed > real_elapsed_micros {
                             let wait = target_real_elapsed - real_elapsed_micros;
                             if wait > 1000.0 { // Sleep if > 1ms lag
                                 tokio::time::sleep(tokio::time::Duration::from_micros(wait as u64)).await;
                             }
                        }
                    }

                    // 1. Update Physics
                    let state = self.physics.update(tick.price, tick.timestamp, 0);

                    // --- D-101: Pessimistic Fill Logic (FIFO Queue) ---
                    // Process Pending Orders BEFORE generating new ones
                    // --- D-101: Pessimistic Fill Logic (FIFO Queue) ---
                    // Process Pending Orders BEFORE generating new ones
                    // In pessimistic mode, we only fill if we drained the queue.
                    if self.pessimistic {
                        let mut filled_indices = Vec::new();
                        let mut fills_to_log = Vec::new();
                        
                        for (i, order) in self.pending_orders.iter_mut().enumerate() {
                            // Check latency condition (has order reached the "exchange"?)
                            if tick.timestamp >= order.placed_at_ts {
                                // Check Price match
                                let price_match = if order.side == "LONG" { tick.price <= order.price } else { tick.price >= order.price };
                                
                                if price_match {
                                    // Decrement FIFO Queue
                                    order.queue_pos -= tick.quantity;
                                    
                                    if order.queue_pos <= 0.0 {
                                        // FILL!
                                        self.trade_counter.add(1, &[KeyValue::new("side", order.side.clone())]);
                                        self.nav_gauge.add(order.qty * tick.price, &[KeyValue::new("type", "exposure_add")]);
                                        self.ledger.update_fill(&order.side, tick.price, order.qty); 
                                        
                                        // Buffer Log
                                        use crate::audit::FrictionLog;
                                        let log = FrictionLog {
                                            ts: Some((tick.timestamp as i64) * 1_000_000), 
                                            symbol: "BTC-USDT".to_string(),
                                            order_id: order.id.clone(),
                                            side: order.side.clone(),
                                            intent_qty: order.qty,
                                            fill_price: tick.price, 
                                            slippage_bps: 5.0, 
                                            gas_usd: 0.0,
                                            realized_pnl: 0.0,
                                            fee_native: 0.0,
                                            tax_buffer: 0.0,
                                        };
                                        fills_to_log.push(log);
                                        filled_indices.push(i);
                                    }
                                }
                            }
                        }
                        
                        // Log Flush
                        for log in fills_to_log {
                             self.auditor.log(log);
                        }

                        // Remove filled
                        for i in filled_indices.into_iter().rev() {
                            self.pending_orders.remove(i);
                        }
                    }
                    
                    // 2. Mock Brain Intent
                    let action = if state.velocity > 0.0 { "LONG" } else { "HOLD" };
                    if action == "LONG" {
                         let intent = TradeProposal {
                             side: "LONG".to_string(),
                             price: tick.price,
                             qty: 0.1, 
                         };
                         
                         // Metric: Signal Generated
                         self.signal_counter.add(1, &[KeyValue::new("side", "LONG")]);

                         let p50 = tick.price * 1.001; 
                         let p90 = tick.price * 1.01;
                         let p10 = tick.price * 0.99;
                         let now = tick.timestamp as i64; // ms

                         let verdict = self.guardian.check(
                             &state, 
                             &self.ledger, 
                             &intent, 
                             p10, p50, p90, now, 0.05
                        );
                        
                        match verdict {
                            RiskVerdict::Allowed => {
                                if self.pessimistic {
                                    // D-101: Queue It Up (Don't Fill Yet)
                                    // 1. Calculate Network Latency (20-150ms)
                                    let mut rng = rand::thread_rng();
                                    let latency_ms: f64 = rng.gen_range(20.0..150.0);
                                    
                                    // 2. Queue Position
                                    let queue_pos = tick.quantity;

                                    let order = OrderState {
                                        id: format!("SIM-{}", count),
                                        side: intent.side.clone(),
                                        qty: intent.qty,
                                        price: tick.price, 
                                        queue_pos,
                                        placed_at_ts: tick.timestamp + latency_ms,
                                    };
                                    self.pending_orders.push(order);

                                } else {
                                    // D-101 OPTIMISTIC: Instant Fill
                                    self.trade_counter.add(1, &[KeyValue::new("side", "LONG")]);
                                    self.nav_gauge.add(intent.qty * tick.price, &[KeyValue::new("type", "exposure_add")]);
                                    self.ledger.update_fill(&intent.side, tick.price, intent.qty);
                                    
                                    // Inline Log (Optimistic)
                                    use crate::audit::FrictionLog;
                                    let log = FrictionLog {
                                        ts: Some(now * 1_000_000), 
                                        symbol: "BTC-USDT".to_string(),
                                        order_id: format!("SIM-{}", count),
                                        side: intent.side.clone(),
                                        intent_qty: intent.qty,
                                        fill_price: tick.price, 
                                        slippage_bps: 0.0, // Optimistic = 0 slippage
                                        gas_usd: 0.0,
                                        realized_pnl: 0.0,
                                        fee_native: 0.0,
                                        tax_buffer: 0.0,
                                    };
                                    self.auditor.log(log);
                                }
                            },
                            RiskVerdict::Veto(_reason) => {},
                            _ => {}
                        }
                    }
                },
                Err(e) => eprintln!("❌ Sim Stream Error: {}", e),
            }
            
            if count % 10_000 == 0 {
                print!(".");
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
        }
        
        let duration = start_time.elapsed();
        println!("\n🏁 Simulation Complete.");
        let final_equity = self.ledger.total_equity(_last_price);
        println!("📊 Stats: {} ticks processed in {:.2}s. Final NAV: ${:.2}", count, duration.as_secs_f64(), final_equity);
        Ok(final_equity)
    }
}

