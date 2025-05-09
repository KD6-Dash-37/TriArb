// src/main.rs

use bytes::Bytes;
use anyhow::Result;
use tri_arb::parse::{parser_loop, TopOfBookUpdate};
use tri_arb::ws::start_ws_listener;
use tri_arb::arb::{create_arb_evaluator, arb_loop, ArbMode};
use tri_arb::price_path::find_and_build_price_paths;
use tokio::sync::mpsc;


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("Starting TriArb");
    
    // Config inputs
    let home_asset = "USDT";
    let targets = ["BTC", "ETH", "SOL"];
    let arb_eval_mode = ArbMode::RayonScan;
    println!("Home asset: {}", home_asset);
    println!("Target assets: {:?}", targets);
    
    // Create resources
    let price_paths = find_and_build_price_paths(home_asset, &targets)?;
    let evaluator = create_arb_evaluator(arb_eval_mode, price_paths.clone());
    let (ws_tx, ws_rx) = mpsc::channel::<Bytes>(4096);
    let (parser_tx, parser_rx) = mpsc::channel::<TopOfBookUpdate>(4096);
    
    // Start loops
    tokio::spawn(arb_loop(parser_rx, evaluator));
    tokio::spawn(parser_loop(ws_rx, parser_tx));
    tokio::spawn(start_ws_listener(price_paths.clone(), ws_tx, Some(true)));
    
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received");
    
    Ok(())
}
