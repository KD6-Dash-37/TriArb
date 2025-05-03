// src/parse/srd_jsn.rs
use anyhow::Result;
use serde_json;
use serde::Deserialize;
use bytes::Bytes;

use super::{TopOfBookUpdate, BookTickerParser};

pub struct SerdeJsonParser;

/// Simple serde_json parser implementation
impl BookTickerParser for SerdeJsonParser {
    fn parse(&self, raw: &Bytes) -> Result<TopOfBookUpdate> {
        let parsed: BookTickerWs = serde_json::from_slice(raw)?;
        Ok(TopOfBookUpdate {
            symbol: parsed.s,
            bid_price: parsed.b.parse()?,
            ask_price: parsed.a.parse()?,
        })
    }
}

#[derive(Debug, Deserialize)]
struct BookTickerWs {
    pub s: String,
    pub b: String,
    pub a: String,
}