// src/price_path.rs

use std::{collections::HashSet, fmt,fs};

use anyhow::Result;
use serde::Deserialize;


/// Loads exchange metadata and constructs all valid triangular pricing paths.
///
/// # Arguments
/// - `home_asset`: The asset to start and end each path with (e.g. "USDT").
/// - `targets`: A whitelist of intermediate assets to consider (e.g. ["BTC", "ETH"]).
///
/// # Returns
/// A list of fully directional `PricingPath` objects, each containing 3 legs.
///
/// This is the main entry point for generating pricing paths for arbitrage evaluation.
pub fn find_and_build_price_paths<'a>(
    home_asset: &'a str,
    targets: &[&'a str],
) -> Result<Vec<PricingPath>> {
    let exchange_info = load_exchange_info_fixture()?;
    let triplets = find_path_symbols(&exchange_info, home_asset, targets);
    Ok(build_paths(home_asset, triplets))
}


/// Root structure for deserializing Binance exchangeInfo JSON.
#[derive(Debug, Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Vec<SymbolInfo>,
}


/// Describes a tradable symbol from Binance, including its base and quote assets.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SymbolInfo {
    pub symbol: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
    pub status: String,
}


/// Indicates the direction to evaluate the price for a trade leg:
/// - `Ask` means buy the base asset using the quote.
/// - `Bid` means sell the base asset to get the quote.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Bid,
    Ask
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (text, color) = match self {
            Self::Ask => ("BUY", "\x1b[32m"), // Green
            Self::Bid => ("SELL", "\x1b[31m"), // Red
        };
        write!(f, "{}{}{}", color, text, "\x1b[0m")
    }
}


/// A single leg of a pricing path: includes the trading pair and side of book
#[derive(Debug, Clone)]
pub struct PathLeg {
    pub symbol: SymbolInfo,
    pub side: Side,
}


/// A complete 3-leg pricing path forming a triangle that starts and ends in the home currency.
/// Each leg specifies the market symbol and trade direction.
#[derive(Debug, Clone)]
pub struct PricingPath {
    pub leg1: PathLeg,
    pub leg2: PathLeg,
    pub leg3: PathLeg,
}

impl<'a> fmt::Display for PricingPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn describe_leg(leg: &PathLeg) -> String {
            format!("{} {}", leg.side, leg.symbol.symbol)
        }
        write!(
            f,
            "{} → {} → {}",
            describe_leg(&self.leg1),
            describe_leg(&self.leg2),
            describe_leg(&self.leg3),
        )
    }
}

impl PricingPath {
    /// Returns all unique symbol names (e.g. "BTCUSDT") used in this path.
    pub fn symbols(&self) -> Vec<String> {
        let mut set = HashSet::new();
        set.insert(self.leg1.symbol.symbol.clone());
        set.insert(self.leg2.symbol.symbol.clone());
        set.insert(self.leg3.symbol.symbol.clone());
        set.into_iter().collect()
    }
}


/// Loads a local JSON fixture file containing Binance exchangeInfo data.
///
/// Used for offline development or testing.
pub fn load_exchange_info_fixture() -> Result<ExchangeInfo> {
    let path = "fixtures/exchangeInfoSpot.json";
    let raw = fs::read_to_string(path)?;
    let parsed: ExchangeInfo = serde_json::from_str(&raw)?;
    Ok(parsed)
}


/// Finds all valid symbol triplets forming a triangular trading loop starting and ending in `home`.
///
/// # Arguments
/// - `exchange_info`: The full list of symbols from Binance.
/// - `home`: The asset to start and end in (e.g. "USDT").
/// - `targets`: Intermediate currencies to consider passing through (e.g. "BTC", "ETH").
///
/// # Returns
/// A list of 3-tuples (symbol1, symbol2, symbol3) that represent candidate triangle paths.
///
/// This function does not assign directional price logic; that happens in `build_paths()`.
pub fn find_path_symbols<'a>(
    exchange_info: &'a ExchangeInfo,
    home: &str,
    targets: &[&str],
) -> Vec<(&'a SymbolInfo, &'a SymbolInfo, &'a SymbolInfo)> {
    let symbols: Vec<&SymbolInfo> = exchange_info
        .symbols
        .iter()
        .filter(|s| s.status == "TRADING")
        .collect();

    let mut result = Vec::new();

    for &leg1 in &symbols {
        if leg1.quote_asset != home { continue; }
        if !targets.contains(&leg1.base_asset.as_str()) { continue; }

        let mid1 = &leg1.base_asset;

        for &leg2 in &symbols {
            if leg2 == leg1 { continue; }

            let connects_to_mid1 = leg2.base_asset == *mid1 || leg2.quote_asset == *mid1;
            if !connects_to_mid1 { continue; }

            if !(targets.contains(&leg2.base_asset.as_str()) && targets.contains(&leg2.quote_asset.as_str())) {
                continue;
            }

            for &leg3 in &symbols {
                if leg3 == leg1 || leg3 == leg2 { continue; }
                if leg3.quote_asset != home { continue; }

                if leg3.base_asset == leg2.base_asset || leg3.base_asset == leg2.quote_asset {
                    result.push((leg1, leg2, leg3));
                }
            }
        }
    }
    result
}


/// Converts symbol triplets into fully directional `PricingPath` structs with side-of-book info.
///
/// # Arguments
/// - `home`: The home currency used to infer direction of trade.
/// - `triplets`: The raw symbols making up each triangular candidate.
///
/// # Returns
/// A vector of `PricingPath` with correct direction and book side assignment.
pub fn build_paths<'a>(
    home: &str,
    triplets: Vec<(&'a SymbolInfo, &'a SymbolInfo, &'a SymbolInfo)>
) -> Vec<PricingPath> {
    let mut result = Vec::new();
    println!("Constructing pricing paths");
    for (s1, s2, s3) in triplets {
        // leg1: home → mid1
        let to1 = if s1.base_asset == home { &s1.quote_asset } else { &s1.base_asset };
        let side1 = side_for_trade(home, s1);

        // leg2: mid1 → mid2
        let to2 = if s2.base_asset == *to1 { &s2.quote_asset } else { &s2.base_asset };
        let side2 = side_for_trade(to1, s2);

        // leg3: mid2 → home
        let side3 = side_for_trade(to2, s3);
        let path = PricingPath {
            leg1: PathLeg { symbol: s1.clone(), side: side1 },
            leg2: PathLeg { symbol: s2.clone(), side: side2 },
            leg3: PathLeg { symbol: s3.clone(), side: side3 },
        };
        println!("Constructed: {}", path);
        result.push(path);
    }

    result
}


/// Determines the correct side of the order book to use given an input asset and symbol.
///
/// # Arguments
/// - `input_asset`: The asset you currently hold.
/// - `symbol`: The trading pair being evaluated.
///
/// # Panics
/// If the symbol does not include the input asset at all.
fn side_for_trade(input_asset: &str, symbol: &SymbolInfo) -> Side {
    if symbol.base_asset == input_asset {
        Side::Bid // You are selling base to get quote
    } else if symbol.quote_asset == input_asset {
        Side::Ask // You are buying base using quote
    } else {
        panic!("Invalid trade direction for {}: from {}", symbol.symbol, input_asset);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    static TARGETS: &[&str] = &["BTC", "ETH", "SOL"];
    static HOME: &str = "USDT";
    fn mock_exchange_info() -> ExchangeInfo {
        ExchangeInfo {
            symbols: vec![
                // ───── BTC/ETH Triangle ─────
                SymbolInfo {
                    symbol: "BTCUSDT".into(),
                    base_asset: "BTC".into(),
                    quote_asset: "USDT".into(),
                    status: "TRADING".into(),
                },
                SymbolInfo {
                    symbol: "ETHBTC".into(),
                    base_asset: "ETH".into(),
                    quote_asset: "BTC".into(),
                    status: "TRADING".into(),
                },
                SymbolInfo {
                    symbol: "ETHUSDT".into(),
                    base_asset: "ETH".into(),
                    quote_asset: "USDT".into(),
                    status: "TRADING".into(),
                },
    
                // ───── SOL/BTC Triangle ─────
                SymbolInfo {
                    symbol: "SOLBTC".into(),
                    base_asset: "SOL".into(),
                    quote_asset: "BTC".into(),
                    status: "TRADING".into(),
                },
                SymbolInfo {
                    symbol: "SOLUSDT".into(),
                    base_asset: "SOL".into(),
                    quote_asset: "USDT".into(),
                    status: "TRADING".into(),
                },
    
                // ───── Controls ─────
                SymbolInfo {
                    symbol: "LTCUSDT".into(),
                    base_asset: "LTC".into(),
                    quote_asset: "USDT".into(),
                    status: "TRADING".into(),
                },
                SymbolInfo {
                    symbol: "BADPAIR".into(),
                    base_asset: "BTC".into(),
                    quote_asset: "ETH".into(),
                    status: "BREAKING".into(), // should be ignored
                }
            ],
        }
    }
    
    #[test]
    fn test_find_path_symbols_triangle_with_btc_eth_sol() {
        let exchange_info = mock_exchange_info();
        let paths = find_path_symbols(&exchange_info, &HOME, &TARGETS);
        assert_eq!(paths.len(), 4, "Expected 4 valid triangle paths");

        let syms: Vec<_> = paths.iter().map(|(a, b, c)| {
            (a.symbol.as_str(), b.symbol.as_str(), c.symbol.as_str())
        }).collect();

        // ETH-BTC triangle (2 directions)
        assert!(syms.contains(&("BTCUSDT", "ETHBTC", "ETHUSDT")));
        assert!(syms.contains(&("ETHUSDT", "ETHBTC", "BTCUSDT")));

        // SOL-BTC triangle (2 directions)
        assert!(syms.contains(&("BTCUSDT", "SOLBTC", "SOLUSDT")));
        assert!(syms.contains(&("SOLUSDT", "SOLBTC", "BTCUSDT")));
    }

    #[test]
    fn no_triangle_when_cross_missing() {
        let exchange_info = ExchangeInfo {
            symbols: vec![
                SymbolInfo {
                    symbol: "BTCUSDT".into(),
                    base_asset: "BTC".into(),
                    quote_asset: "USDT".into(),
                    status: "TRADING".into(),
                },
                SymbolInfo {
                    symbol: "BTCUSDC".into(),
                    base_asset: "BTC".into(),
                    quote_asset: "USDC".into(),
                    status: "TRADING".into(),
                },
                SymbolInfo {
                    symbol: "ETHUSDT".into(),
                    base_asset: "ETH".into(),
                    quote_asset: "USDT".into(),
                    status: "TRADING".into(),
                },
                // Control: not part of triangle
                SymbolInfo {
                    symbol: "ETHUSDC".into(),
                    base_asset: "ETH".into(),
                    quote_asset: "USDC".into(),
                    status: "TRADING".into(),
                },
            ],
        };
        let result = find_path_symbols(&exchange_info, "USDT", &["BTC", "ETH"]);
        assert_eq!(result.len(), 0, "Should not find a triangle without ETHBTC");
    }

    #[test]
    fn all_paths_have_three_distinct_assets() {
        let exchange_info = mock_exchange_info();
        let triplets = find_path_symbols(&exchange_info, &HOME, &TARGETS);
        let paths = build_paths(&HOME, triplets);

        for (i, path) in paths.iter().enumerate() {
            let mut assets = std::collections::HashSet::new();
            assets.insert(HOME.to_string());
            assets.insert(path.leg1.symbol.base_asset.clone());
            assets.insert(path.leg1.symbol.quote_asset.clone());
            assets.insert(path.leg2.symbol.base_asset.clone());
            assets.insert(path.leg2.symbol.quote_asset.clone());
            assets.insert(path.leg3.symbol.base_asset.clone());
            assets.insert(path.leg3.symbol.quote_asset.clone());

            assert!(
                assets.len() >= 3,
                "Path {} has fewer than 3 unique assets:\n{}",
                i,
                path
            );
        }
    }
    
    #[test]
    fn all_paths_start_and_end_with_home() {
        let exchange_info = mock_exchange_info();
        let triplets = find_path_symbols(&exchange_info, &HOME, &TARGETS);
        let paths = build_paths(&HOME, triplets);

        for (i, path) in paths.iter().enumerate() {
            let start_assets = [&path.leg1.symbol.base_asset, &path.leg1.symbol.quote_asset];
            let end_assets = [&path.leg3.symbol.base_asset, &path.leg3.symbol.quote_asset];

            assert!(
                start_assets.contains(&&HOME.to_string()),
                "Path {} does not start with home asset: {}", i, path
            );
            assert!(
                end_assets.contains(&&HOME.to_string()),
                "Path {} does not end with home asset: {}", i, path
            );
        }
    }
    
    #[test]
    fn no_duplicate_symbols_in_path() {
        let exchange_info = mock_exchange_info();
        let triplets = find_path_symbols(&exchange_info, &HOME, &TARGETS);
        let paths = build_paths(&HOME, triplets);

        for (i, path) in paths.iter().enumerate() {
            let symbols = [
                &path.leg1.symbol.symbol,
                &path.leg2.symbol.symbol,
                &path.leg3.symbol.symbol,
            ];
            let unique: std::collections::HashSet<_> = symbols.iter().collect();
            assert_eq!(
                unique.len(),
                3,
                "Path {} reuses a symbol:\n{}", i, path
            );
        }
    }
    
    #[test]
    fn all_legs_have_valid_side_assignment() {
        let exchange_info = mock_exchange_info();
        let triplets = find_path_symbols(&exchange_info, &HOME, &TARGETS);
        let paths = build_paths(&HOME, triplets);
        for path in paths.iter() {
            for leg in [&path.leg1, &path.leg2, &path.leg3] {
                match leg.side {
                    Side::Bid | Side::Ask => {},
                }
            }
        }
    }
}
