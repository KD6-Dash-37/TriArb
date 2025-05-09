// src/devtools/path_sampler.rs

use std::collections::HashSet;
use anyhow::Result;
use super::load_exchange_info;
use crate::price_path::{find_and_build_price_paths, PricingPath};


/// Sample up to `n` triangular arbitrage paths that start and end with the given `home_asset`.
///
/// This uses all unique base assets from the exchange info as potential targets,
/// allowing full discovery of 3-leg paths (including cross-quote opportunities).
///
/// Returns:
/// - A list of pricing paths (up to `n`)
/// - A flattened, deduplicated list of symbols used in those paths
pub fn sample_paths(home_asset: &str, path_count: usize) -> Result<(Vec<PricingPath>, Vec<String>)> {
    let info = load_exchange_info()?;

    // Collect all unique base assets from the exchange info
    let mut target_assets = HashSet::new();
    for symbol in &info.symbols {
        target_assets.insert(symbol.base_asset.clone());
    }
    let targets: Vec<&str> = target_assets.iter().map(String::as_str).collect();

    let all_paths = find_and_build_price_paths(home_asset, &targets)?;
    let sampled_paths = all_paths.into_iter().take(path_count).collect::<Vec<_>>();

    let mut symbol_set = HashSet::new();
    for path in &sampled_paths {
        symbol_set.extend(path.symbols());
    }

    let symbols: Vec<String> = symbol_set.into_iter().collect();

    Ok((sampled_paths, symbols))
}