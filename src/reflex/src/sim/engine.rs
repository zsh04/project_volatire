use crate::feynman::PhysicsEngine;
use crate::taleb::{RiskGuardian, TradeProposal, RiskVerdict};
use crate::ledger::AccountState;
use crate::sim::ticker::SimTicker;
// use crate::brain_proto::StrategyIntent; // Unused
use std::time::Instant;
use futures_util::StreamExt;

pub struct SimulationEngine {
    physics: PhysicsEngine,
    guardian: RiskGuardian,
    ledger: AccountState,
    ticker: SimTicker,
    auditor: crate::audit::QuestBridge,
}

impl SimulationEngine {
    pub async fn new(db_url: &str, auditor: crate::audit::QuestBridge) -> Result<Self, Box<dyn std::error::Error>> {
        let ticker = SimTicker::new(db_url).await?;
        Ok(Self {
            physics: PhysicsEngine::new(2000), 
            guardian: RiskGuardian::new(),
            ledger: AccountState::new(100_000.0, 0.0), 
            ticker,
            auditor, 
        })
    }

    pub async fn run(mut self, start_ts: i64, end_ts: i64, speed: f64) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting SHADOW SIMULATION: {} to {} (Speed: {:.1}x)", start_ts, end_ts, speed);
        
        // Sim State
        let mut sim_stream = self.ticker.stream_history("BTC-USDT", start_ts, end_ts).await?;
        let mut count = 0;
        let start_time = Instant::now();
        let mut _last_price = 0.0; // Fixed warning
        
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
                    let state = self.physics.update(tick.price, tick.timestamp);
                    
                    // 2. Mock Brain Intent
                    let action = if state.velocity > 0.0 { "LONG" } else { "HOLD" };
                    if action == "LONG" {
                         let intent = TradeProposal {
                             side: "LONG".to_string(),
                             price: tick.price,
                             qty: 0.1, 
                         };
                         
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
                                // --- D-25B: Friction Logging ---
                                // Log to QuestDB via Auditor
                                use crate::audit::FrictionLog;
                                let log = FrictionLog {
                                    ts: Some(now * 1_000_000), // ms -> nanos
                                    symbol: "BTC-USDT".to_string(),
                                    order_id: format!("SIM-{}", count),
                                    side: intent.side.clone(),
                                    intent_qty: intent.qty,
                                    fill_price: tick.price, // Zero slippage logic
                                    slippage_bps: 0.0,
                                    gas_usd: 0.0,
                                    realized_pnl: 0.0,
                                    fee_native: 0.0,
                                    tax_buffer: 0.0,
                                };
                                self.auditor.log(log);
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
        println!("📊 Stats: {} ticks processed in {:.2}s", count, duration.as_secs_f64());
        Ok(())
    }
}
