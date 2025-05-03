// src/parse/mod.rs

pub mod srd_jsn;
pub mod man_scan;

use std::sync::Arc;
use anyhow::Result;
use bytes::Bytes;
use tokio::sync::mpsc::{Receiver, Sender};


#[derive(Debug, Clone)]
pub struct TopOfBookUpdate {
    pub symbol: String,
    pub bid_price: f64,
    pub ask_price: f64,
}


pub async fn parser_loop(
    mut ws_rx: Receiver<Bytes>,
    parser_tx: Sender<TopOfBookUpdate>,
) -> Result<()> {
    
    let parser: Arc<dyn BookTickerParser + Send + Sync> = create_parser();

    while let Some(raw_msg) = ws_rx.recv().await {
        match parser.parse(&raw_msg) {
            Ok(update) => {
                #[cfg(feature = "print_parsed")]
                {
                    println!("{:?}", update);
                }
                if let Err(e) = parser_tx.try_send(update) {
                    eprintln!("Failed to send parsed update: {e}");
                }
            }
            Err(e) => {
                eprintln!("Failed to parse incoming message: {e}");
            }
        }
    }
    Ok(())
}

pub trait BookTickerParser {
    fn parse(&self, raw: &Bytes) -> Result<TopOfBookUpdate>;
}

fn create_parser() -> Arc<dyn BookTickerParser + Send + Sync> {
    #[cfg(all(feature = "serde_parser", not(feature = "manual_parser")))]
    {
        return Arc::new(srd_jsn::SerdeJsonParser);
    }

    #[cfg(all(feature = "manual_parser", not(feature = "serde_parser")))]
    {
        return Arc::new(man_scan::ManualScanParser);
    }

    #[cfg(not(any(feature = "serde_parser", feature = "manual_parser")))]
    compile_error!("At least one parser feature (`serde_parser` or `manual_parser`) must be enabled.");

    #[cfg(all(feature = "serde_parser", feature = "manual_parser"))]
    compile_error!("Cannot enable both `serde_parser` and `manual_parser` features at the same time.");
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    const SAMPLE_MSG: &str = r#"{"e":"bookTicker","u":123456,"s":"BTCUSDT","b":"30000.12","B":"1.0","a":"30001.45","A":"2.0"}"#;

    #[test]
    fn test_serde_json_parser() {
        let parser = srd_jsn::SerdeJsonParser;
        let input = Bytes::from(SAMPLE_MSG);
        let result = parser.parse(&input).expect("Serde parser failed");

        assert_eq!(result.symbol, "BTCUSDT");
        assert!((result.bid_price - 30000.12).abs() < 1e-6);
        assert!((result.ask_price - 30001.45).abs() < 1e-6);
    }

    #[test]
    fn test_manual_scan_parser() {
        let parser = man_scan::ManualScanParser;
        let input = Bytes::from(SAMPLE_MSG);
        let result = parser.parse(&input).expect("Manual parser failed");

        assert_eq!(result.symbol, "BTCUSDT");
        assert!((result.bid_price - 30000.12).abs() < 1e-6);
        assert!((result.ask_price - 30001.45).abs() < 1e-6);
    }

    #[test]
    fn test_parsers_consistency() {
        let input = Bytes::from(SAMPLE_MSG);

        let serde_parser = srd_jsn::SerdeJsonParser;
        let manual_parser = man_scan::ManualScanParser;

        let serde_result = serde_parser.parse(&input).expect("Serde parser failed");
        let manual_result = manual_parser.parse(&input).expect("Manual parser failed");

        assert_eq!(serde_result.symbol, manual_result.symbol, "Symbols do not match");
        assert!((serde_result.bid_price - manual_result.bid_price).abs() < 1e-6, "Bid prices do not match");
        assert!((serde_result.ask_price - manual_result.ask_price).abs() < 1e-6, "Ask prices do not match");
    }

}
