// benches/arb.rs

// cargo bench --bench arb -- --save-baseline current
// critcmp current

use criterion::{
    criterion_group,
    criterion_main,
    BenchmarkGroup,
    Criterion,
    black_box,
};
use rand::seq::SliceRandom;
use rand::thread_rng;

use tri_arb::arb::{ArbEvaluator, HashMapEdgeScanner, NaivePrecompiledScanner, RayonBestMatchScanner, RayonFirstMatchScanner};
use tri_arb::parse::TopOfBookUpdate;

use tri_arb::devtools::path_sampler::sample_paths;


fn mock_updates(symbols: &[String], count: usize) -> Vec<TopOfBookUpdate> {
    let mut updates = Vec::with_capacity(count);
    for i in 0..count {
        let symbol = &symbols[i % symbols.len()];
        updates.push(TopOfBookUpdate {
            symbol: symbol.clone(),
            bid_price: 1.0 + (i as f64 % 100.0) * 0.0001,
            ask_price: 1.0 + (i as f64 % 100.0) * 0.00015,
        })
    }
    updates.shuffle(&mut thread_rng());
    updates
}


fn bench_scanner<B: ArbEvaluator + 'static>(
    group: &mut BenchmarkGroup<criterion::measurement::WallTime>,
    label: &str,
    updates: &[TopOfBookUpdate],
    scanner: B,
) {
    group.bench_function(label, |b| {
        b.iter(|| {
            for u in black_box(updates) {
                let _ = scanner.process_update(u);
            }
        })
    });
}


pub fn bench_scanners_small_universe_few_updates(c: &mut Criterion) {
    // Test params
    let path_count = 5;
    let n_updates = 10;
        
    // Test preparation & resources
    let (paths, symbols) = sample_paths("USDT", path_count).expect("path sampling failed");
    let updates = mock_updates(&symbols, n_updates);

    // Arb scanners
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());
    let rayon_best = RayonBestMatchScanner::new(paths.clone());
    let rayon_first = RayonFirstMatchScanner::new(paths.clone());

    let group_name = format!("arb_timed/small_universe_few_updates/paths={path_count}/updates={n_updates}");
    let mut group = c.benchmark_group(group_name);
    
    bench_scanner(&mut group, "naive", &updates, naive);
    bench_scanner(&mut group, "edge", &updates, edge);
    bench_scanner(&mut group, "rayon_best", &updates, rayon_best);
    bench_scanner(&mut group, "rayon_first", &updates, rayon_first);

    group.finish();
}

fn bench_scanners_small_universe_many_updates(c: &mut Criterion) {
    // Test params
    let path_count = 5;
    let n_updates = 500_000;
    
    // Test preparation & resources
    let (paths, symbols) = sample_paths("USDT", path_count).expect("path sampling failed");
    let updates = mock_updates(&symbols, n_updates);

    // Arb scanners
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());
    let rayon_best = RayonBestMatchScanner::new(paths.clone());
    let rayon_first = RayonFirstMatchScanner::new(paths.clone());

    let group_name = format!("arb_timed/small_universe_many_updates/paths={path_count}/updates={n_updates}");
    let mut group = c.benchmark_group(group_name);

    bench_scanner(&mut group, "naive", &updates, naive);
    bench_scanner(&mut group, "edge", &updates, edge);
    bench_scanner(&mut group, "rayon_best", &updates, rayon_best);
    bench_scanner(&mut group, "rayon_first", &updates, rayon_first);

    group.finish();
}


fn bench_scanners_large_universe_few_updates(c: &mut Criterion) {
    // Test params
    let path_count = 100;
    let n_updates = 10;
    
    // Test preparation & resources
    let (paths, symbols) = sample_paths("USDT", path_count).expect("path sampling failed");
    let updates = mock_updates(&symbols, n_updates);
    
    // Arb scanners
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());
    let rayon_best = RayonBestMatchScanner::new(paths.clone());
    let rayon_first = RayonFirstMatchScanner::new(paths.clone());

    let group_name = format!("arb_timed/large_universe_few_updates/paths={path_count}/updates={n_updates}");
    let mut group = c.benchmark_group(group_name);

    bench_scanner(&mut group, "naive", &updates, naive);
    bench_scanner(&mut group, "edge", &updates, edge);
    bench_scanner(&mut group, "rayon_best", &updates, rayon_best);
    bench_scanner(&mut group, "rayon_first", &updates, rayon_first);

    group.finish();
}


fn bench_scanners_large_universe_many_updates(c: &mut Criterion) {
    // Test params
    let path_count = 100;
    let n_updates = 500_000;
        
    // Test preparation & resources
    let (paths, symbols) = sample_paths("USDT", path_count).expect("path sampling failed");
    let updates = mock_updates(&symbols, n_updates);

    // Arb scanners
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());
    let rayon_best = RayonBestMatchScanner::new(paths.clone());
    let rayon_first = RayonFirstMatchScanner::new(paths.clone());
    
    let group_name = format!("arb_timed/large_universe_many_updates/paths={path_count}/updates={n_updates}");
    let mut group = c.benchmark_group(group_name);

    bench_scanner(&mut group, "naive", &updates, naive);
    bench_scanner(&mut group, "edge", &updates, edge);
    bench_scanner(&mut group, "rayon_best", &updates, rayon_best);
    bench_scanner(&mut group, "rayon_first", &updates, rayon_first);

    group.finish();
}


criterion_group!(
    arb_benches,
    bench_scanners_small_universe_few_updates,
    bench_scanners_small_universe_many_updates,
    bench_scanners_large_universe_few_updates,
    bench_scanners_large_universe_many_updates,
);

criterion_main!(arb_benches);
