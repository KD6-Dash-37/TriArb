// src/price_path.rs

use std::{fmt,fs};

use anyhow::Result;
use serde::Deserialize;

pub fn find_and_build_price_paths<'a>(
    home_asset: &'a str,
    targets: &[&'a str],
) -> Result<Vec<PricingPath>> {
    let exchange_info = load_exchange_info_fixture()?;
    let triplets = find_path_symbols(&exchange_info, home_asset, targets);
    Ok(build_paths(home_asset, triplets))
}


/// Wrapper around exchangeInfo JSON response/fixture
#[derive(Debug, Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Vec<SymbolInfo>,
}


/// Raw symbol info from Binance exchangeInfo
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SymbolInfo {
    pub symbol: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
    pub status: String,
}


/// Indicates which side of the book to evaluate for the leg
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
#[derive(Debug)]
pub struct PathLeg {
    pub symbol: SymbolInfo,
    pub side: Side,
}


/// Represent a valid 3-leg pricing path (triangle)
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


/// Load static exchangeInfo from fixture
pub fn load_exchange_info_fixture() -> Result<ExchangeInfo> {
    let path = "fixtures/exchangeInfoSpot.json";
    let raw = fs::read_to_string(path)?;
    let parsed: ExchangeInfo = serde_json::from_str(&raw)?;
    Ok(parsed)
}


/// Finds all possible 3-leg triangular trading paths starting and ending at the home currency.
/// 
/// # Arguments
/// - `exchange_info`: Full Binance symbol list
/// - `home_asset`: Currency to start/end in (e.g., "USDT")
/// - `targets`: Currency assets we are looking to trade through
/// 
/// # Returns
/// Vector of tuples containing the 3 SymbolInfo entries that make up a valid triangular path.
pub fn find_path_symbols<'a>(
    exchange_info: &'a ExchangeInfo,
    home_asset: &str,
    targets: &[&str],
) -> Vec<(&'a SymbolInfo, &'a SymbolInfo, &'a SymbolInfo)> {
    // Filter only actively trading symbols from Binance reference data
    let active: Vec<&SymbolInfo> = exchange_info
        .symbols
        .iter()
        .filter(|s| s.status == "TRADING")
        .collect();

    let mut results = Vec::new();

    for &leg1 in &active {
        // ───── LEG 1 ─────
        // We start with a pair that converts from home → mid1
        // Example: BTCUSDT (base = BTC, quote = USDT)
        if leg1.quote_asset != home_asset { continue; }
        if !targets.contains(&leg1.base_asset.as_str()) { continue; }
        
        let mid1 = &leg1.base_asset;

        for &leg2 in &active {
            if leg2 == leg1 { continue; }
            // ───── LEG 2 ─────
            // This leg must trade from mid1 to some other currency (mid2)
            // So we check whether mid1 is either the base or quote in leg2
            let connects = leg2.base_asset == *mid1 || leg2.quote_asset == *mid1;
            if !connects { continue; }

            // Determine mid2 (the other side of the leg)
            let mid2 = if leg2.base_asset == *mid1 {
                &leg2.quote_asset
            } else {
                &leg2.base_asset
            };

            // Avoid degenerate triangles (mid2 shouldn't be home or mid1 again)
            if mid2 == home_asset || mid2 == mid1 { continue; }

            for &leg3 in &active {
                if leg3 == leg1 || leg3 == leg2 { continue; }

                // ───── LEG 3 ─────
                // This leg must bring us back from mid2 → home
                // Example: ETHUSDT or USDTETH (depending on direction)
                let valid_leg3 = 
                    (leg3.base_asset == *mid2 && leg3.quote_asset == home_asset)
                    || 
                    (leg3.quote_asset == *mid2 && leg3.base_asset == home_asset);

                if valid_leg3 {
                    results.push((leg1, leg2, leg3));
                }
            }
        }
    }
    results
}


/// Constructs directional `PricingPath` objects from symbol triplets
/// inferring the appropriate side of book for each leg.
///
/// # Panics
/// Panics only if `find_path_symbols` returned invalid data (should never happen).
pub fn build_paths<'a>(
    home: &str,
    symbol_triples: Vec<(&'a SymbolInfo, &'a SymbolInfo, &'a SymbolInfo)>
) -> Vec<PricingPath> {
    let mut result = Vec::new();
    println!("Constructing Pricing Paths...");
    for (sym1, sym2, sym3) in symbol_triples {
        // Step 1: USDT → mid1
        let to1 = &sym1.base_asset;
        let side1 = side_for_trade(sym1, home, to1);

        // Step 2: mid1 → mid2
        let from2 = to1;
        let to2 = if &sym2.base_asset == from2 {
            &sym2.base_asset
        } else if &sym2.quote_asset == from2 {
            &sym2.base_asset
        } else {
            unreachable!("find_path_symbols should guarantee sym2 connects to mid1");
        };
        let side2 = side_for_trade(sym2, from2, to2);

        // Step 3: mid2 → USDT
        let from3 = to2;
        let side3 = side_for_trade(sym3, from3, home);
        let path = PricingPath {
            leg1: PathLeg { symbol: sym1.clone(), side: side1 },
            leg2: PathLeg { symbol: sym2.clone(), side: side2 },
            leg3: PathLeg { symbol: sym3.clone(), side: side3 },
        };
        println!("{}", path);
        result.push(path)
    }
    println!("Pricing Path construction complete!");
    result
}


/// Infers whether the trade should use the bid or ask side of the book
fn side_for_trade(symbol: &SymbolInfo, input_asset: &str, output_asset: &str) -> Side {
    if symbol.base_asset == input_asset && symbol.quote_asset == output_asset {
        Side::Ask // You are buying 'base' with 'quote'
    } else if symbol.base_asset == output_asset && symbol.quote_asset == input_asset {
        Side::Bid // You are selling 'base' to get 'quote'
    } else {
        panic!("Invalid trade direction for {}: {} → {}", symbol.symbol, input_asset, output_asset);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn mock_exchange_info() -> ExchangeInfo {
        ExchangeInfo {
            symbols: vec![
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
                // Control: not part of triangle
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
    fn test_find_path_symbols_btc_eth_triangle() {
        let exchange_info = mock_exchange_info();
        let home = "USDT";
        let targets = ["BTC", "ETH"];

        let paths = find_path_symbols(&exchange_info, home, &targets);
        assert_eq!(paths.len(), 2, "Expected 2 valid triangle paths");

        let syms: Vec<_> = paths.iter().map(|(a, b, c)| {
            (a.symbol.as_str(), b.symbol.as_str(), c.symbol.as_str())
        }).collect();

        assert!(syms.contains(&("BTCUSDT", "ETHBTC", "ETHUSDT")));
        assert!(syms.contains(&("ETHUSDT", "ETHBTC", "BTCUSDT")));
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
    fn test_side_for_trade_bid_ask_logic() {
        let sym = SymbolInfo {
            symbol: "ETHBTC".into(),
            base_asset: "ETH".into(),
            quote_asset: "BTC".into(),
            status: "TRADING".into(),
        };

        // Scenario 1: Buying ETH using BTC → Ask
        let side = side_for_trade(&sym, "ETH", "BTC");
        assert_eq!(side, Side::Ask);

        // Scenario 2: Selling ETH for BTC → Bid
        let side = side_for_trade(&sym, "BTC", "ETH");
        assert_eq!(side, Side::Bid);

        // Scenario 3: Invalid direction → should panic
        let result = std::panic::catch_unwind(|| {
            side_for_trade(&sym, "USDT", "LTC");
        });
        assert!(result.is_err(), "Expected panic for invalid direction");
    }

}
