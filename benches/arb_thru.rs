// benches/arb_thru.rs

use criterion::{criterion_group, criterion_main, Criterion, Throughput, black_box};
use tri_arb::{
    arb::{HashMapEdgeScanner, NaivePrecompiledScanner, RayonBestMatchScanner, ArbEvaluator, RayonFirstMatchScanner},
    devtools::path_sampler::sample_paths,
    parse::TopOfBookUpdate,
};
use rand::seq::SliceRandom;
use rand::thread_rng;

fn mock_updates(symbols: &[String], count: usize) -> Vec<TopOfBookUpdate> {
    let mut updates = Vec::with_capacity(count);
    for i in 0..count {
        let symbol = &symbols[i % symbols.len()];
        updates.push(TopOfBookUpdate {
            symbol: symbol.clone(),
            bid_price: 1.0 + (i as f64 % 100.0) * 0.0001,
            ask_price: 1.0 + (i as f64 % 100.0) * 0.00015,
        });
    }
    updates.shuffle(&mut thread_rng());
    updates
}

fn bench_arb_scanner_throughput(c: &mut Criterion) {
    let path_count = 50;
    let n_updates = 100_000;

    let (paths, symbols) = sample_paths("USDT", path_count).expect("Failed to sample paths");
    let updates = mock_updates(&symbols, n_updates);

    // Arb scanners
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());
    let rayon_best = RayonBestMatchScanner::new(paths.clone());
    let rayon_first = RayonFirstMatchScanner::new(paths.clone());

    let mut group = c.benchmark_group("arb_throughput");
    group.throughput(Throughput::Elements(n_updates as u64));

    group.bench_function("naive", |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = naive.process_update(u);
            }
        });
    });

    group.bench_function("edge", |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = edge.process_update(u);
            }
        });
    });

    group.bench_function("rayon_best", |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_best.process_update(u);
            }
        });
    });
    group.bench_function("rayon_first", |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_first.process_update(u);
            }
        });
    });
}

criterion_group!(arb_thru_benches, bench_arb_scanner_throughput);
criterion_main!(arb_thru_benches);