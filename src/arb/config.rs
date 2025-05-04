// src/arb/config.rs

use serde::Deserialize;


/// Top-level arbitrage configuration loaded from `config/arb.toml`.
#[derive(Debug, Deserialize, Clone)]
pub struct ArbConfig {
    pub rayon_scan: Option<RayonScanConfig>
}

#[derive(Debug, Deserialize, Clone)]
pub struct RayonScanConfig {
    pub on_update_return: OnUpdateReturn
}

/// Strategy for returning arbitrage results on update.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OnUpdateReturn {
    /// Return the first profitable path found (fastest).
    First,
    /// Evaluate all paths and return the most profitable one.
    Best
}

impl Default for OnUpdateReturn {
    fn default() -> Self {
        Self::First
    }
}
