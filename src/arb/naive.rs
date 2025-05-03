// src/arb/naive.rs

use std::sync::Arc;
use dashmap::DashMap;

use super::ArbEvaluator;
use crate::parse::TopOfBookUpdate;


#[derive(Debug, Clone)]
pub struct Triangle {
    pub leg1: String,
    pub leg2: String,
    pub leg3: String,
}

pub struct NaivePrecompiledScanner {
    triangles: Arc<Vec<Triangle>>,
    price_store: dashmap::DashMap<String, TopOfBookUpdate>,
}

impl ArbEvaluator for NaivePrecompiledScanner {
    fn process_update(&self, update: &TopOfBookUpdate) {
        self.price_store.insert(update.symbol.clone(), update.clone());
        
        for tri in self.triangles.iter() {
            let btcusdt = match self.price_store.get("BTCUSDT") {
                Some(v) => v.clone(),
                None => continue,
            };
            let ethbtc = match self.price_store.get("ETHBTC") {
                Some(v) => v.clone(),
                None => continue,
            };
            let ethusdt = match self.price_store.get("ETHUSDT") {
                Some(v) => v.clone(),
                None => continue,
            };

            let start = 1.0;
            let step1 =  start / btcusdt.ask_price;
            let step2 = step1 / ethbtc.ask_price;
            let final_amount = step2 * ethusdt.bid_price;

            if final_amount > start * 1.0001 {
                println!(
                    "✅ Arbitrage! {} -> {} -> {} | Start: {:.6} End: {:.6} Profit: {:.6}",
                    tri.leg1, tri.leg2, tri.leg3,
                    start, final_amount,
                    final_amount - start
                );
            }
        }
    }
}

impl NaivePrecompiledScanner {
    pub fn new(symbols: &[&str; 3]) -> Self {
        let naive_triangle = create_naive_triangle(symbols);
        let triangles = Arc::new(vec![naive_triangle]);
        Self {
            triangles,
            price_store: DashMap::new(),
        }
    }
}

fn create_naive_triangle(symbols: &[&str; 3]) -> Triangle {
    let required = ["BTCUSDT", "ETHBTC", "ETHUSDT"];
    for &req in &required {
        if !symbols.contains(&req) {
            panic!("Missing required symbol {} to run NaivePrecompiledScanner. Provided: {:?}", req, symbols);
        }
    }
    Triangle {
        leg1: "BTCUSDT".to_string(),
        leg2: "ETHBTC".to_string(),
        leg3: "ETHUSDT".to_string(),
    }
}