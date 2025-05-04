// src/dummy/ws_server.rs

use std::sync::Arc;

use tokio::{net::TcpListener, time::{sleep, Duration}};
use tokio_tungstenite::{accept_async, tungstenite::{Message, Utf8Bytes}};
use futures_util::{StreamExt, SinkExt};


use super::hot_cache::HotCache;


pub async fn run(cache: HotCache) {
    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    println!("🟢 Dummy WebSocket server on ws://127.0.0.1:9001");
    while let Ok((stream, _)) = listener.accept().await {
        let cache = Arc::clone(&cache);
        tokio::spawn(handle_connection(stream, cache));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, cache: HotCache) {
    let mut ws_stream = accept_async(stream).await.unwrap();
    println!("New connection!");
    
    let msg = match ws_stream.next().await {
        Some(Ok(Message::Text(txt))) => txt,
        _ => {
            eprintln!("No valid subscribe message received");
            return;
        }
    };

    let parsed: serde_json::Value = serde_json::from_str(&msg).expect("Invalid JSON");
    let symbols: Vec<String>  = parsed["params"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.trim_end_matches("@bookTicker").to_uppercase())
        .collect();

    println!("Client subscribed to: {:?}", symbols);

    loop {
        let guard = cache.read().await;

        for symbol in &symbols {
            if let Some(msg) = guard.get(symbol) {
                if ws_stream.send(Message::Text(Utf8Bytes::from(msg))).await.is_err() {
                    eprintln!("Client disconnected");
                    return;
                }
            }
        }

        sleep(Duration::from_millis(100)).await;
    }
}