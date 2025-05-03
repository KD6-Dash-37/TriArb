// benches/parser_bench.rs

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use bytes::Bytes;
use tri_arb::parse::{srd_jsn::SerdeJsonParser, man_scan::ManualScanParser, BookTickerParser};

const SAMPLE_MSG: &str = r#"{"e":"bookTicker","u":123456,"s":"BTCUSDT","b":"30000.12","B":"1.0","a":"30001.45","A":"2.0"}"#;

pub fn benchmark_single_parse(c: &mut Criterion) {
    let input = Bytes::from_static(SAMPLE_MSG.as_bytes());

    let serde_parser = SerdeJsonParser;
    let manual_parser = ManualScanParser;

    c.bench_function("single_parse_serde_json", |b| {
        b.iter(|| {
            let _ = serde_parser.parse(black_box(&input)).unwrap();
        })
    });

    c.bench_function("single_parse_manual_scan", |b| {
        b.iter(|| {
            let _ = manual_parser.parse(black_box(&input)).unwrap();
        })
    });
}

pub fn benchmark_batch_parse(c: &mut Criterion) {
    let single_msg = Bytes::from_static(SAMPLE_MSG.as_bytes());
    let batch_size = 100_000; // TODO parametrize it with ParameterizedBenchmark (advanced Criterion usage)

    // Prepare a batch of 100_000 messages
    let batch: Vec<Bytes> = (0..batch_size)
        .map(|_| single_msg.clone())
        .collect();

    let serde_parser = SerdeJsonParser;
    let manual_parser = ManualScanParser;

    c.bench_function(&format!("batch_parse_serde_json_batch_{}", batch_size), |b| {
        b.iter(|| {
            for msg in black_box(&batch) {
                let _ = serde_parser.parse(msg).unwrap();
            }
        })
    });

    c.bench_function(&format!("batch_parse_manual_scan_batch_{}", batch_size), |b| {
        b.iter(|| {
            for msg in black_box(&batch) {
                let _ = manual_parser.parse(msg).unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_single_parse,
    benchmark_batch_parse
);
criterion_main!(benches);
