// benches/arb.rs

use criterion::{criterion_group, Criterion, black_box, criterion_main};
use tri_arb::arb::{ArbEvaluator, HashMapEdgeScanner, NaivePrecompiledScanner};
use tri_arb::parse::TopOfBookUpdate;
use tri_arb::price_path::{PricingPath, PathLeg, SymbolInfo, Side};


fn sample_path() -> PricingPath {
    let s1 = SymbolInfo {
        symbol: "BTCUSDT".into(),
        base_asset: "BTC".into(),
        quote_asset: "USDT".into(),
        status: "TRADING".into(),
    };
    let s2 = SymbolInfo {
        symbol: "ETHBTC".into(),
        base_asset: "ETH".into(),
        quote_asset: "BTC".into(),
        status: "TRADING".into(),
    };
    let s3 = SymbolInfo {
        symbol: "ETHUSDT".into(),
        base_asset: "ETH".into(),
        quote_asset: "USDT".into(),
        status: "TRADING".into(),
    };
    PricingPath {
        leg1: PathLeg { symbol: s1, side: Side::Ask },
        leg2: PathLeg { symbol: s2, side: Side::Ask },
        leg3: PathLeg { symbol: s3, side: Side::Bid },
    }
}


fn mock_updates(count: usize) -> Vec<TopOfBookUpdate> {
    let mut updates = Vec::with_capacity(count);
    for i in 0..count {
        let symbol = match i % 3 {
            0 => "BTCUSDT",
            1 => "ETHBTC",
            _ => "ETHUSDT",
        };
        updates.push(TopOfBookUpdate {
            symbol: symbol.to_string(),
            bid_price: 1.0 + (i as f64 % 100.0) * 0.0001,
            ask_price: 1.0 + (i as f64 % 100.0) * 0.00015,
        });
    }
    updates
}

pub fn bench_evaluators(c: &mut Criterion) {
    let updates = mock_updates(10);
    let paths = vec![sample_path()]; // 10 copies

    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());

    c.bench_function("arb/naive/process_update", |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = naive.process_update(u);
            }
        })
    });

    c.bench_function("arb/edge/process_update", |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = edge.process_update(u);
            }
        })
    }); 
}

fn bench_overload_scanners(c: &mut Criterion) {
    
    let updates = mock_updates(500_000);
    let paths = vec![sample_path()];
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());

    c.bench_function("arb/naive/heavy_update", |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = naive.process_update(u);
            }
        })
    });

    c.bench_function("arb/edge/heavy_update", |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = edge.process_update(u);
            }
        })
    });
}

criterion_group!(
    arb_benches,
    bench_evaluators,
    bench_overload_scanners
);

criterion_main!(arb_benches);