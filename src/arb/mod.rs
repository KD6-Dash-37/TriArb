// src/arb/mod.rs
use std::{fs, path::Path};
use std::sync::Arc;

use anyhow::Result;
use config::OnUpdateReturn;
use tokio::sync::mpsc::Receiver;
use tracing::info;
use crate::{parse::TopOfBookUpdate, price_path::PricingPath};

pub mod naive;
pub mod edge;
pub mod rayon_scan;
pub mod config;

pub use config::{ArbConfig, RayonScanConfig};
pub use naive::NaivePrecompiledScanner;
pub use edge::HashMapEdgeScanner;
pub use rayon_scan::{RayonFirstMatchScanner, RayonBestMatchScanner};


const CONFIG_FILE_PATH: &str = "config/arb.toml";


fn load_arb_config<P: AsRef<Path>>(path: P) -> Option<ArbConfig> {
    let contents = fs::read_to_string(path).ok()?;
    toml::from_str(&contents).ok()
}

#[derive(Debug, Clone, Copy)]
pub enum ArbMode {
    Naive,
    EdgeMap,
    RayonScan,
}

pub fn create_arb_evaluator(
    mode: ArbMode,
    price_paths: Vec<PricingPath>
) -> Arc<dyn ArbEvaluator + Send + Sync> {
    let config = load_arb_config(CONFIG_FILE_PATH);
    info!(?mode, "Creating arbitrage evaluator");
    
    match mode {
        
        ArbMode::Naive => {
            info!("Using NaivePrecompiledScanner");
            Arc::new(NaivePrecompiledScanner::new(price_paths))
        },
        
        ArbMode::EdgeMap => {
            info!("Using HashMapEdgeScanner");
            Arc::new(HashMapEdgeScanner::new(price_paths))
        },
        
        ArbMode::RayonScan => {
            
            let on_update_return = config
                .as_ref()
                .and_then(|c| c.rayon_scan.as_ref())
                .map(|r| r.on_update_return.clone())
                .unwrap_or_default();

            info!(?on_update_return, "Using RayonPathScanner");
            
            match on_update_return {
                OnUpdateReturn::First => Arc::new(RayonFirstMatchScanner::new(price_paths)),
                OnUpdateReturn::Best => Arc::new(RayonBestMatchScanner::new(price_paths)),
            }
        }
    }
}

pub trait ArbEvaluator: Send + Sync {
    fn process_update(&self, update: &TopOfBookUpdate) -> Option<(PricingPath, f64)>;
}

pub async fn arb_loop(
    mut rx: Receiver<TopOfBookUpdate>,
    evaluator: Arc<dyn ArbEvaluator>,
) -> Result<()> {
    while let Some(update) = rx.recv().await {
        if let Some((_path, _result)) = evaluator.process_update(&update) {
            // println!(
            //     "✅ Arbitrage found: {} | Return: {:.6} | Profit: {:.4}%",
            //     _path,
            //     _result,
            //     (_result - 1.0) * 100.0
            // );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::price_path::{PathLeg, PricingPath, Side, SymbolInfo};

    fn mock_path() -> PricingPath {
        let s1 = SymbolInfo {
            symbol: "BTCUSDT".into(),
            base_asset: "BTC".into(),
            quote_asset: "USDT".into(),
            status: "TRADING".into(),
        };
        let s2 = SymbolInfo {
            symbol: "ETHBTC".into(),
            base_asset: "ETH".into(),
            quote_asset: "BTC".into(),
            status: "TRADING".into(),
        };
        let s3 = SymbolInfo {
            symbol: "ETHUSDT".into(),
            base_asset: "ETH".into(),
            quote_asset: "USDT".into(),
            status: "TRADING".into(),
        };

        PricingPath {
            leg1: PathLeg { symbol: s1, side: Side::Ask },
            leg2: PathLeg { symbol: s2, side: Side::Ask },
            leg3: PathLeg { symbol: s3, side: Side::Bid },
        }
    }

    fn mock_update(symbol: &str, bid: f64, ask: f64) -> TopOfBookUpdate {
        TopOfBookUpdate {
            symbol: symbol.to_string(),
            bid_price: bid,
            ask_price: ask,
        }
    }

    #[test]
    fn test_edge_scanner_accepts_update() {
        let path = mock_path();
        let scanner = HashMapEdgeScanner::new(vec![path]);

        scanner.process_update(&mock_update("BTCUSDT", 30000.0, 30010.0));
        scanner.process_update(&mock_update("ETHBTC", 0.065, 0.066));
        scanner.process_update(&mock_update("ETHUSDT", 1980.0, 1985.0));

        // There's no assertion here yet, since current logic just prints.
        // You can add a counter, hook, or event log in future versions to validate detection.
    }
}
