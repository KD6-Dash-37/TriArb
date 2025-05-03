// src/main.rs

use std::sync::Arc;
use bytes::Bytes;
use anyhow::Result;
use tri_arb::parse::{parser_loop, TopOfBookUpdate};
use tri_arb::ws::start_ws_listener;
use tri_arb::arb::{naive::NaivePrecompiledScanner, arb_loop};
use tri_arb::price_path::find_and_build_price_paths;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Binance WS listener");
    let home_asset = "USDT";
    let targets = ["BTC", "ETH", "SOL"];
    let _price_paths = find_and_build_price_paths(home_asset, &targets);
    
    let symbols = ["BTCUSDT", "ETHBTC", "ETHUSDT"];

    let evaluator = Arc::new(NaivePrecompiledScanner::new(&symbols));
    
    let (ws_tx, ws_rx) = mpsc::channel::<Bytes>(4096);
    let (parser_tx, parser_rx) = mpsc::channel::<TopOfBookUpdate>(4096);
    
    tokio::spawn(arb_loop(parser_rx, evaluator));
    tokio::spawn(parser_loop(ws_rx, parser_tx));
    tokio::spawn(start_ws_listener(symbols.to_vec(), ws_tx));
    
    tokio::signal::ctrl_c().await?;
    println!("Shutdown signal received");
    Ok(())
}
