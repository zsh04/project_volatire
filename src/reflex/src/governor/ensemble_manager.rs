use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EnsembleManager {
    pub current_adapter_id: String,
    regime_map: HashMap<String, String>,
}

impl EnsembleManager {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        // D-95: Regime-to-Adapter Mapping
        // "The Chameleon": Matching Cognitive Style to Market Physics
        map.insert("Laminar".to_string(), "adapter_trend_follower_v1".to_string());
        map.insert("Turbulent".to_string(), "adapter_mean_reversion_v2".to_string());
        map.insert("Violent".to_string(), "adapter_volatility_hawk_v1".to_string());
        map.insert("Unknown".to_string(), "adapter_generalist_base".to_string());

        Self {
            current_adapter_id: "adapter_generalist_base".to_string(),
            regime_map: map,
        }
    }

    /// Updates the active adapter based on the identified Regime.
    /// Returns TRUE if a swap occurred (so we can log it).
    pub fn update_regime(&mut self, regime: &str) -> bool {
        // Default to Generalist if regime unknown
        let target_adapter = self.regime_map.get(regime)
            .unwrap_or(&"adapter_generalist_base".to_string())
            .clone();

        if self.current_adapter_id != target_adapter {
            tracing::info!("🦎 CHAMELEON SWAP: {} -> {}", self.current_adapter_id, target_adapter);
            self.current_adapter_id = target_adapter;
            true
        } else {
            false
        }
    }

    pub fn get_active_adapter(&self) -> String {
        self.current_adapter_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regime_switch_logic() {
        let mut mgr = EnsembleManager::new();
        
        // Initial state
        assert_eq!(mgr.get_active_adapter(), "adapter_generalist_base");

        // 1. Laminar -> Trend Follower
        let switched = mgr.update_regime("Laminar");
        assert!(switched);
        assert_eq!(mgr.get_active_adapter(), "adapter_trend_follower_v1");

        // 2. Same Regime -> No Swap
        let switched_again = mgr.update_regime("Laminar");
        assert!(!switched_again);

        // 3. Violent -> Volatility Hawk
        mgr.update_regime("Violent");
        assert_eq!(mgr.get_active_adapter(), "adapter_volatility_hawk_v1");
        
        // 4. Unknown -> Generalist
        mgr.update_regime("Aliens");
        assert_eq!(mgr.get_active_adapter(), "adapter_generalist_base");
    }
}
