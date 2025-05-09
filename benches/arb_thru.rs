// benches/arb_thru.rs

// cargo bench --bench arb_thru -- --save-baseline current

use criterion::{
    criterion_group,
    criterion_main,
    BenchmarkGroup,
    Criterion,
    Throughput,
    black_box,
};
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


fn bench_scanner_throughput<B: ArbEvaluator>(
    group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    updates: &[TopOfBookUpdate],
    scanner: B,
) {
    group.bench_function(name, |b| {
        b.iter(|| {
            for u in black_box(updates) {
                let _ = scanner.process_update(u);
            }
        });
    });
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
    
    bench_scanner_throughput(&mut group, "naive", &updates, naive);
    bench_scanner_throughput(&mut group, "edge", &updates, edge);
    bench_scanner_throughput(&mut group, "rayon_best", &updates, rayon_best);
    bench_scanner_throughput(&mut group, "rayon_first", &updates, rayon_first);

    group.finish();
}

criterion_group!(arb_thru_benches, bench_arb_scanner_throughput);
criterion_main!(arb_thru_benches);