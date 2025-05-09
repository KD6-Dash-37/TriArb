// src/devtools/mod.rs

pub mod path_sampler;

use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

use crate::price_path::ExchangeInfo;


/// Load the exchangeInfo fixture from disk.
///
/// Panics if the file is missing or invalid.
pub fn load_exchange_info() -> Result<ExchangeInfo> {
    let path = Path::new("fixtures/exchangeInfoSpot.json");

    let contents = fs::read_to_string(&path).with_context(|| {
        format!(
            "❌ Failed to read '{}'.\n\
             Please ensure the fixture exists.\n\
             Tip: run CREATE DEV TOOL TO DOWNLOAD FIXTURE or copy it from the Binance API.",
            path.display()
        )
    })?;

    let parsed: ExchangeInfo = serde_json::from_str(&contents).with_context(|| {
        format!(
            "❌ Failed to parse '{}'.\n\
             Ensure it's valid JSON and matches the expected structure.",
            path.display()
        )
    })?;

    Ok(parsed)
}