// src/arb/edge.rs

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;

use crate::arb::ArbEvaluator;
use crate::parse::TopOfBookUpdate;
use crate::price_path::{PricingPath, Side};

/// A fast arbitrage evaluator that indexes triangular paths by symbol (edge)
/// so only relevant paths are re-evaluated on each update.
pub struct HashMapEdgeScanner {
    price_store: DashMap<String, TopOfBookUpdate>,
    path_index: HashMap<String, Vec<Arc<PricingPath>>>
}

impl HashMapEdgeScanner {
    /// Constructs a new HashMapEdgeScanner by indexing all paths by the symbols they reference.
    pub fn new(price_paths: Vec<PricingPath>) -> Self {
        let wrapped_paths: Vec<Arc<PricingPath>> = price_paths.into_iter().map(Arc::new).collect();
        let mut index: HashMap<String, Vec<Arc<PricingPath>>> = HashMap::with_capacity(wrapped_paths.len() * 3);

        for path in &wrapped_paths {
            for symbol in path.symbols() {
                index.entry(symbol).or_default().push(Arc::clone(path));
            }
        }
        
        Self {
            price_store: DashMap::new(),
            path_index: index,    
        }
    }
}

impl ArbEvaluator for HashMapEdgeScanner {
    /// Processes a top-of-book update and checks for arbitrage opportunities
    /// using only paths involving the updated symbol.
    fn process_update(&self, update: &TopOfBookUpdate) -> Option<(PricingPath, f64)> {
        self.price_store.insert(update.symbol.clone(), update.clone());
        const START: f64 = 1.0;
        if let Some(paths) = self.path_index.get(&update.symbol) {
            for path in paths {
                
                let s1 = &path.leg1.symbol.symbol;
                let s2 = &path.leg2.symbol.symbol;
                let s3 = &path.leg3.symbol.symbol;

                // Early filter: skip path if not all 3 symbols are present
                    if !(self.price_store.contains_key(s1)
                    && self.price_store.contains_key(s2)
                    && self.price_store.contains_key(s3))
                {
                    continue;
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
                    Side::Ask => step1 /  p2.ask_price,
                    Side::Bid => step1 * p2.bid_price 
                };

                let end = match path.leg3.side {
                    Side::Ask => step2 / p3.ask_price,
                    Side::Bid => step2 * p3.bid_price,
                };

                if end > START {
                    return Some((path.as_ref().clone(), end));
                };
            }
        }
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::price_path::{PathLeg, SymbolInfo};

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

    #[test]
    fn test_indexing_symbols_from_paths() {
        let path = mock_path();
        let scanner = HashMapEdgeScanner::new(vec![path]);

        assert!(scanner.path_index.contains_key("BTCUSDT"));
        assert!(scanner.path_index.contains_key("ETHBTC"));
        assert!(scanner.path_index.contains_key("ETHUSDT"));
    }

    #[test]
    fn test_no_false_positive_paths() {
        let path = mock_path();
        let scanner = HashMapEdgeScanner::new(vec![path]);

        assert!(!scanner.path_index.contains_key("FOOBAR"));
    }
}