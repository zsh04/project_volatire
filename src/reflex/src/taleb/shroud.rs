use crate::client::brain::StrategyIntent;


#[derive(Debug, PartialEq)]
pub enum ShroudVerdict {
    Safe,
    NuclearExit(String), // Reason
}

pub struct RiskShroud;

impl RiskShroud {
    pub fn new() -> Self {
        Self
    }

    /// Calculates the Bayesian Expected Shortfall (BES) and checks if price breaches the shroud.
    /// 
    /// BES_Long = (P10 + P20) / 2
    /// BES_Short = (P80 + P90) / 2
    pub fn check_shroud(
        &self, 
        current_price: f64, 
        intent: &StrategyIntent, 
        _entropy: f64
    ) -> ShroudVerdict {
        // If no active forecast, we can't enforce the shroud (or fallback to generic stop).
        // For D-22, we assume intent carries the latest forecast.
        if intent.model_used == "warming_up" || intent.model_used == "error_fallback" {
             return ShroudVerdict::Safe;
        }

        // Determine direction from Intent (Boyd's current stance)
        // Note: Ideally, Shroud protects the CURRENT POSITION. 
        // But in this architecture, Intent reflects the target state.
        // If Intent is LONG, we protect against downside.
        let action = &intent.action;

        if action == "LONG" {
             let p10 = intent.forecast_p10;
             let p20 = intent.forecast_p20;
             
             // Bayesian Expected Shortfall (Downside)
             let bes_long = (p10 + p20) / 2.0;

             if current_price < bes_long {
                 return ShroudVerdict::NuclearExit(format!(
                     "Price ({:.2}) breached BES Shroud (Logic: Long | P10: {:.2}, P20: {:.2} | Limit: {:.2})",
                     current_price, p10, p20, bes_long
                 ));
             }

        } else if action == "SHORT" {
             let p80 = intent.forecast_p80;
             let p90 = intent.forecast_p90;
             
             // Bayesian Expected Shortfall (Upside risk for Short)
             let bes_short = (p80 + p90) / 2.0;

             if current_price > bes_short {
                 return ShroudVerdict::NuclearExit(format!(
                     "Price ({:.2}) breached BES Shroud (Logic: Short | P80: {:.2}, P90: {:.2} | Limit: {:.2})",
                     current_price, p80, p90, bes_short
                 ));
             }
        }

        ShroudVerdict::Safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::brain::StrategyIntent;

    #[test]
    fn test_bes_long_breach() {
        let shroud = RiskShroud::new();
        let intent = StrategyIntent {
            action: "LONG".to_string(),
            forecast_p10: 100.0,
            forecast_p20: 102.0, // BES = 101.0
            forecast_p50: 105.0,
            forecast_p80: 108.0,
            forecast_p90: 110.0,
            ..Default::default()
        };

        // Price 101.5 (Safe)
        assert_eq!(shroud.check_shroud(101.5, &intent, 0.0), ShroudVerdict::Safe);
        
        // Price 100.5 (Breach < 101.0)
        let verdict = shroud.check_shroud(100.5, &intent, 0.0);
        match verdict {
            ShroudVerdict::NuclearExit(r) => assert!(r.contains("breached BES Shroud")),
            _ => panic!("Should have panicked"),
        }
    }

    #[test]
    fn test_bes_short_breach() {
        let shroud = RiskShroud::new();
        let intent = StrategyIntent {
            action: "SHORT".to_string(),
            forecast_p10: 90.0,
            forecast_p20: 92.0,
            forecast_p50: 95.0,
            forecast_p80: 98.0,
            forecast_p90: 100.0, // BES = 99.0
            ..Default::default()
        };

        // Price 99.5 (Breach > 99.0)
        let verdict = shroud.check_shroud(99.5, &intent, 0.0);
         match verdict {
            ShroudVerdict::NuclearExit(r) => assert!(r.contains("breached BES Shroud")),
            _ => panic!("Should have panicked"),
        }
    }
}
