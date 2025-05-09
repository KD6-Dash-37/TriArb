// examples/mock_feed_to_ws.rs

//! Example: Connect to the mock WebSocket feed and print raw bookTicker messages.
//!
//! This example demonstrates how to:
//! - Build pricing paths for a set of target assets
//! - Launch the in-process mock Binance-style WebSocket server
//! - Use the production WebSocket client to subscribe to mock data
//! - Print raw `bookTicker` JSON messages received from the mock server
//!
//! Run with:
//! ```bash
//! cargo run --example mock_feed_to_ws
//! ```

use std::collections::HashSet;

use bytes::Bytes;
use tokio::sync::mpsc;

use tri_arb::price_path::find_and_build_price_paths;
use tri_arb::mock_feed::hot_cache::start_hot_cache_updater;
use tri_arb::mock_feed::ws_server;
use tri_arb::ws::start_ws_listener;


#[tokio::main]
async fn main() {
    // Home asset (e.g., USDT) used as the base for pricing paths.
    let home_asset = "USDT";

    // Target assets we want to trade against the home asset.
    let targets = ["BTC", "ETH", "SOL"];

    // Generate 3-leg arbitrage paths using your production logic.
    let price_paths = find_and_build_price_paths(home_asset, &targets)
        .unwrap_or_else(|e| panic!("Unable to build price paths: {e}"));

    // Extract all unique market symbols (e.g., BTCUSDT) from pricing paths.
    let mut unique_symbols: HashSet<String> = HashSet::new();
    for path in &price_paths {
        unique_symbols.extend(path.symbols());
    }
    let symbols: Vec<String> = unique_symbols.iter().cloned().collect();

    // Start a high-frequency market data generator (the "hot cache").
    // This acts as the simulated exchange backend.
    let cache = start_hot_cache_updater(symbols.clone(), 20);

    // Start a WebSocket server that streams from the hot cache.
    // Clients will connect and subscribe just like they would to Binance.
    tokio::spawn(ws_server::run(cache));

    // Create a channel to receive mock data frames from the client.
    let (tx, mut rx) = mpsc::channel::<Bytes>(100);
    
    // Start the production WebSocket client and have it subscribe to mock symbols.
    tokio::spawn({
        let paths = price_paths.clone();
        async move {
            start_ws_listener(paths, tx, Some(true)).await.unwrap();
        }
    });

    println!("📡 Listening to mock feed for {:?}", targets);

    // Print each received message (raw `bookTicker` JSON) to stdout.
    while let Some(msg) = rx.recv().await {
        println!("📥 {}", String::from_utf8_lossy(&msg));
    }
}