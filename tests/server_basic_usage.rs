// src/tests/server_basic_usage.rs

// cargo test --test server_basic_usage -- --nocapture


#[tokio::test]
async fn test_ws_client_receives_dummy_data() {
    use std::collections::HashSet;
    use std::time::Duration;
    
    use bytes::Bytes;
    use serde_json::Value;
    use tokio::sync::mpsc;
    use tokio::time::timeout;
    
    use tri_arb::price_path::find_and_build_price_paths;
    use tri_arb::mock_feed::hot_cache::start_hot_cache_updater;
    use tri_arb::mock_feed::ws_server;
    use tri_arb::ws::start_ws_listener;
    
    // Set up pricing logic
    let home_asset = "USDT";
    let targets = ["BTC", "ETH", "SOL"];
    let price_paths = find_and_build_price_paths(home_asset, &targets)
        .unwrap_or_else(|e| panic!("Unable to build price paths: {e}"));
    
    // Flatten all pricing path symbols into a duplicated Vec<String>
    let mut unique_symbols: HashSet<String> = HashSet::new();
    for path in &price_paths {
        unique_symbols.extend(path.symbols());
    }
    let symbols: Vec<String> = unique_symbols.iter().cloned().collect();

    // Start the hot cache and dummy WebSocket server
    let cache = start_hot_cache_updater(symbols.clone(), 20);
    tokio::spawn(ws_server::run(cache));

    // Create channel to receive message from client
    // and start the websocket client which will automatically subscribe to the symbols
    let (tx, mut rx) = mpsc::channel::<Bytes>(100);
    // Start the websocket client
    tokio::spawn({
        let paths = price_paths.clone();
        async move {
            start_ws_listener(paths, tx, Some(true)).await.unwrap();
        }
    });

    // Receive messages and ensure we got at least one per symbol
    let mut received_symbols: HashSet<String> = HashSet::new();

    let success = timeout(Duration::from_secs(5), async {
        while received_symbols.len() < symbols.len() {
            if let Some(bytes) = rx.recv().await {
                let msg = String::from_utf8_lossy(&bytes);
                if let Ok(json) = serde_json::from_str::<Value>(&msg) {
                    if let Some(sym) = json.get("s").and_then(|s| s.as_str()) {
                        received_symbols.insert(sym.to_string());
                    }
                }
            }
        }
    })
    .await
    .is_ok();
    
    assert!(success, "Timeout: not all symbols received");
    assert_eq!(received_symbols.len(), symbols.len(), "Mismatch in symbol count");
    println!("✅ Received all expected symbols: {:?}", received_symbols);
}