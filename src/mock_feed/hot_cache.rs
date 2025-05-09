// src/dummy/hot_cache.rs

use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration
};


use serde_json::json;
use tokio::sync::RwLock;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use rand::rngs::OsRng;

/// A shared, concurrent map of symbol → pre-serialized bookTicker messages.
pub type HotCache = Arc<RwLock<HashMap<String, String>>>;

/// Spawns the background task that updates the hot cache every `interval_ms`.
pub fn start_hot_cache_updater(symbols: Vec<String>, interval_ms: u64) -> HotCache {
    let cache: HotCache = Arc::new(RwLock::new(HashMap::new()));
    let cache_clone = Arc::clone(&cache);

    tokio::spawn(async move {
        let mut rng = ChaCha12Rng::from_rng(OsRng).unwrap();
        let interval = Duration::from_millis(interval_ms);
        let mut update_ids: HashMap<String, u64> = HashMap::new();
        
        loop {
            {
                let mut guard = cache_clone.write().await;

                for symbol in &symbols {
                    // Get and increment the update ID
                    let counter = update_ids.entry(symbol.clone()).or_insert(1);
                    let u = *counter;
                    *counter +=1;
                    let bid = rng.gen_range(10000.0..30000.0);
                    let ask = bid + rng.gen_range(0.01..0.05);
                    let tick = json!({
                        "u": u,
                        "s": symbol,
                        "b": format!("{:.8}", bid),
                        "B": format!("{:.8}", rng.gen_range(1.0..100.0)),
                        "a": format!("{:.8}", ask),
                        "A": format!("{:.8}", rng.gen_range(1.0..100.0))
                    });

                    guard.insert(symbol.clone(), tick.to_string());
                }
                tokio::time::sleep(interval).await;
            }
        }
    });
    cache
}