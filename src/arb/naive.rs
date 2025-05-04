// src/arb/naive.rs

use dashmap::DashMap;

use crate::parse::TopOfBookUpdate;
use crate::price_path::{PricingPath, Side};

use super::ArbEvaluator;

pub struct NaivePrecompiledScanner {
    paths: Vec<PricingPath>,
    price_store: DashMap<String, TopOfBookUpdate>,
}

impl ArbEvaluator for NaivePrecompiledScanner {
    fn process_update(&self, update: &TopOfBookUpdate) -> Option<(PricingPath, f64)> {
        self.price_store.insert(update.symbol.clone(), update.clone());

        for path in self.paths.iter() {
            let Some(p1) = self.price_store.get(&path.leg1.symbol.symbol) else { continue; };
            let Some(p2) = self.price_store.get(&path.leg2.symbol.symbol) else { continue; };
            let Some(p3) = self.price_store.get(&path.leg3.symbol.symbol) else { continue; };
            
            const START: f64 = 1.0;
            
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
                Side::Bid => step2 * p3.bid_price
            };

            if end > START {
                return Some((path.clone(), end));
            };
        }
        None
    }
}

impl NaivePrecompiledScanner {
    pub fn new(paths: Vec<PricingPath>) -> Self {
        let price_store = DashMap::new();
        Self {
            paths,
            price_store
        }
    }
}
