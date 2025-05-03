// src/parse/man_scan.rs

use anyhow::{Result, anyhow};
use bytes::Bytes;

use super::{TopOfBookUpdate, BookTickerParser};


#[allow(dead_code)]
pub struct ManualScanParser;

impl BookTickerParser for ManualScanParser {
    fn parse(&self, raw: &Bytes) -> Result<TopOfBookUpdate> {
        let text = std::str::from_utf8(raw)?;

        let symbol = extract_json_field(text, "\"s\":\"")?;
        let bid_str = extract_json_field(text, "\"b\":\"")?;
        let ask_str = extract_json_field(text, "\"a\":\"")?;

        let bid_price: f64 = bid_str.parse()?;
        let ask_price: f64 = ask_str.parse()?;

        Ok(TopOfBookUpdate {
            symbol,
            bid_price,
            ask_price
        })
    }
}

#[allow(dead_code)]
fn extract_json_field(
    text: &str,
    key: &str
) -> Result<String> {
    let start = text.find(key)
        .ok_or_else(|| anyhow!("Key not found: {}", key))? + key.len();
    let end = text[start..]
        .find('"')
        .ok_or_else(|| anyhow!("No ending quote after key: {}", key))? + start;

    Ok(text[start..end].to_string())
}