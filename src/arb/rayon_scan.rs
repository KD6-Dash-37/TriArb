// src/arb/rayon_scan.rs

use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use rayon::prelude::*;

use crate::{parse::TopOfBookUpdate, price_path::{PricingPath, Side}};

use super::ArbEvaluator;

const START: f64 = 1.0;

/// `RayonPathScanner` evaluates arbitrage opportunities across all known pricing paths
/// using data-parallelism via the Rayon library.
///
/// This scanner is optimized for environments with:
/// - High volumes of pricing paths (e.g. 1000+ paths)
/// - Multicore CPUs that benefit from concurrent path evaluation
///
/// Unlike `HashMapEdgeScanner`, this implementation **does not filter paths by symbol**.
/// Instead, it re-evaluates *all* paths on every update, distributing the work across threads.
///
/// Internally uses a `DashMap` for concurrent price storage and `Arc<PricingPath>` for safe parallel access.
pub struct RayonFirstMatchScanner {
    price_store: DashMap<String, TopOfBookUpdate>,
    symbol_to_paths: HashMap<String, Vec<Arc<PricingPath>>>,
}

impl RayonFirstMatchScanner {
    /// Constructs a new `RayonFirstMatchScanner`, wrapping the provided paths in `Arc`
    /// for safe access across threads.
    pub fn new(price_paths: Vec<PricingPath>) -> Self {
        let paths: Vec<Arc<PricingPath>> = price_paths.into_iter().map(Arc::new).collect();

        // Preallocate with 3x paths since each path maps to 3 symbols
        let mut symbol_to_paths: HashMap<String, Vec<Arc<PricingPath>>> = HashMap::with_capacity(paths.len() * 3);
        
        for path in &paths {
            for symbol in path.symbols() {
                symbol_to_paths.entry(symbol).or_default().push(Arc::clone(path));
            }
        }
        Self {
            price_store: DashMap::new(),
            symbol_to_paths,
        }
    }
}


impl ArbEvaluator for RayonFirstMatchScanner {
    /// Evaluates only the pricing paths involving the updated symbol in parallel using Rayon.
    /// Returns the first profitable match, if any. 
    /// This is a fast, non-deterministic approach ideal for high-frequency updates.
    fn process_update(&self, update: &TopOfBookUpdate) -> Option<(PricingPath, f64)> {
        self.price_store.insert(update.symbol.clone(), update.clone());
        let Some(relevant_paths) = self.symbol_to_paths.get(&update.symbol) else {
            return None
        };

        relevant_paths
            .par_iter()
            .find_map_any(|path| {
                let s1 = &path.leg1.symbol.symbol;
                let s2 = &path.leg2.symbol.symbol;
                let s3 = &path.leg3.symbol.symbol;

                // Early filter: skip path if not all 3 symbols are present
                if !(self.price_store.contains_key(s1)
                    && self.price_store.contains_key(s2)
                    && self.price_store.contains_key(s3)) {
                    return None;
                }

                // Safe to unwrap now
                let p1 = self.price_store.get(s1).unwrap();
                let p2 = self.price_store.get(s2).unwrap();
                let p3 = self.price_store.get(s3).unwrap();

                let step1 = match path.leg1.side {
                    Side::Ask => START / p1.ask_price,
                    Side::Bid => START * p1.bid_price,
                };

                let step2 = match path.leg2.side {
                    Side::Ask => step1 / p2.ask_price,
                    Side::Bid => step1 * p2.bid_price,
                };

                let end = match path.leg3.side {
                    Side::Ask => step2 / p3.ask_price,
                    Side::Bid => step2 * p3.bid_price,
                };

                if end > START {
                    Some((path.as_ref().clone(), end))
                } else {
                    None
                }
        })
    }
}


/// `RayonBestMatchScanner` evaluates all known pricing paths in parallel and returns
/// the **most profitable** arbitrage opportunity, rather than the first one found.
///
/// This strategy incurs slightly more overhead per update than `RayonFirstMatchScanner`
/// but ensures the best available opportunity is returned.
pub struct RayonBestMatchScanner {
    price_store: DashMap<String, TopOfBookUpdate>,
    symbol_to_paths: HashMap<String, Vec<Arc<PricingPath>>>,
}


impl RayonBestMatchScanner {
    /// Constructs a new `RayonBestMatchScanner`, wrapping the provided paths in `Arc`
    /// for safe access across threads.
    pub fn new(price_paths: Vec<PricingPath>) -> Self {
        let paths: Vec<Arc<PricingPath>> = price_paths.into_iter().map(Arc::new).collect();

        // Preallocate with 3x paths since each path maps to 3 symbols
        let mut symbol_to_paths: HashMap<String, Vec<Arc<PricingPath>>> = HashMap::with_capacity(paths.len() * 3);
        
        for path in &paths {
            for symbol in path.symbols() {
                symbol_to_paths.entry(symbol).or_default().push(Arc::clone(path));
            }
        }
        Self {
            price_store: DashMap::new(),
            symbol_to_paths,
        }
    }
}


impl ArbEvaluator for RayonBestMatchScanner {
    /// Evaluates all relevant paths involving the updated symbol in parallel,
    /// returning the most profitable opportunity (if any).
    /// This ensures deterministic selection of the best opportunity but incurs slightly higher cost than early-exit scanning.
    fn process_update(&self, update: &TopOfBookUpdate) -> Option<(PricingPath, f64)> {
        self.price_store.insert(update.symbol.clone(), update.clone());
        let Some(relevant_paths) = self.symbol_to_paths.get(&update.symbol) else {
            return None;
        };
        relevant_paths
            .par_iter()
            .filter_map(|path| {
                let s1 = &path.leg1.symbol.symbol;
                let s2 = &path.leg2.symbol.symbol;
                let s3 = &path.leg3.symbol.symbol;

                // Early filter: skip path if not all 3 symbols are present
                if !(self.price_store.contains_key(s1)
                    && self.price_store.contains_key(s2)
                    && self.price_store.contains_key(s3)) {
                    return None;
                }

                // Safe to unwrap now
                let p1 = self.price_store.get(s1).unwrap();
                let p2 = self.price_store.get(s2).unwrap();
                let p3 = self.price_store.get(s3).unwrap();

                let step1 = match path.leg1.side {
                    Side::Ask => START / p1.ask_price,
                    Side::Bid => START * p1.bid_price,
                };

                let step2 = match path.leg2.side {
                    Side::Ask => step1 / p2.ask_price,
                    Side::Bid => step1 * p2.bid_price,
                };

                let end = match path.leg3.side {
                    Side::Ask => step2 / p3.ask_price,
                    Side::Bid => step2 * p3.bid_price,
                };

                if end > START {
                    Some((path.as_ref().clone(), end))
                } else {
                    None
                }
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
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
    fn test_rayon_scanner_detects_arb() {
        let path = mock_path();
        let scanner = RayonFirstMatchScanner::new(vec![path]);

        scanner.process_update(&mock_update("BTCUSDT", 95460.0, 95461.0));
        scanner.process_update(&mock_update("ETHBTC", 0.01914, 0.01915));
        scanner.process_update(&mock_update("ETHUSDT",1827.6, 1827.7)); // bid = ask = 1985

        let result = scanner.process_update(&mock_update("ETHUSDT", 1980.0, 1985.0));
        assert!(result.is_some());
    }

    #[test]
    fn test_best_path_is_selected_from_multiple_profitable_paths() {
        use crate::price_path::{SymbolInfo, PathLeg, Side};

        fn make_symbol(symbol: &str, base: &str, quote: &str) -> SymbolInfo {
            SymbolInfo {
                symbol: symbol.to_string(),
                base_asset: base.to_string(),
                quote_asset: quote.to_string(),
                status: "TRADING".into(),
            }
        }

        // Path 1: BTC → ETH → USDT
        let path1 = PricingPath {
            leg1: PathLeg { symbol: make_symbol("BTCUSDT", "BTC", "USDT"), side: Side::Ask },
            leg2: PathLeg { symbol: make_symbol("ETHBTC", "ETH", "BTC"), side: Side::Ask },
            leg3: PathLeg { symbol: make_symbol("ETHUSDT", "ETH", "USDT"), side: Side::Bid },
        };

        // Path 2: BTC → SOL → USDT (intentionally better ROI)
        let path2 = PricingPath {
            leg1: PathLeg { symbol: make_symbol("BTCUSDT", "BTC", "USDT"), side: Side::Ask },
            leg2: PathLeg { symbol: make_symbol("SOLBTC", "SOL", "BTC"), side: Side::Ask },
            leg3: PathLeg { symbol: make_symbol("SOLUSDT", "SOL", "USDT"), side: Side::Bid },
        };

        let scanner = RayonBestMatchScanner::new(vec![path1.clone(), path2.clone()]);

        // Insert quotes for both paths, tweaking prices so Path 2 has better arbitrage
        scanner.process_update(&mock_update("BTCUSDT", 50000.0, 50010.0)); // ask = 50010
        scanner.process_update(&mock_update("ETHBTC", 0.06, 0.061));
        scanner.process_update(&mock_update("ETHUSDT", 3000.0, 3001.0)); // ETH ask = 3001, bid = 3000

        scanner.process_update(&mock_update("SOLBTC", 0.005, 0.0051));
        scanner.process_update(&mock_update("SOLUSDT", 260.0, 261.0)); // SOL bid = 260

        // Trigger update
        let result = scanner.process_update(&mock_update("BTCUSDT", 50000.0, 50010.0));
        assert!(result.is_some());

        let (best_path, return_val) = result.unwrap();

        // Assert that the selected path is path2 (the SOL one)
        assert_eq!(best_path.leg2.symbol.symbol, "SOLBTC");
        assert!(return_val > 1.0);
    }
}