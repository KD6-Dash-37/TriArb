// benches/arb.rs

use criterion::{criterion_group, Criterion, black_box, criterion_main};
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

pub fn bench_scanners_small_universe_few_updates(c: &mut Criterion) {
    // Test params
    let path_count = 5;
    let n_updates = 10;
    
    // Test ID's
    let naive_test_id = format!("arb/naive/small_universe_few_updates/paths={path_count}/updates={n_updates}");
    let edge_test_id = format!("arb/edge/small_universe_few_updates/paths={path_count}/updates={n_updates}");
    let rayon_best_test_id = format!("arb/rayon_best/small_universe_few_updates/paths={path_count}/updates={n_updates}");
    let rayon_first_test_id = format!("arb/rayon_first/small_universe_few_updates/paths={path_count}/updates={n_updates}");
    
    // Test preparation & resources
    let (paths, symbols) = sample_paths("USDT", path_count).expect("path sampling failed");
    let updates = mock_updates(&symbols, n_updates);

    // Arb scanners
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());
    let rayon_best = RayonBestMatchScanner::new(paths.clone());
    let rayon_first = RayonFirstMatchScanner::new(paths.clone());

    c.bench_function(&naive_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = naive.process_update(u);
            }
        })
    });
    c.bench_function(&edge_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = edge.process_update(u);
            }
        })
    });
    c.bench_function(&rayon_best_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_best.process_update(u);
            }
        })
    });
    c.bench_function(&rayon_first_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_first.process_update(u);
            }
        })
    });  
}

fn bench_scanners_small_universe_many_updates(c: &mut Criterion) {
    // Test params
    let path_count = 5;
    let n_updates = 500_000;
    
    // Test ID's
    let naive_test_id = format!("arb/naive/small_universe_many_updates/paths={path_count}/updates={n_updates}");
    let edge_test_id = format!("arb/edge/small_universe_many_updates/paths={path_count}/updates={n_updates}");
    let rayon_best_test_id = format!("arb/rayon_best/small_universe_many_updates/paths={path_count}/updates={n_updates}");
    let rayon_first_test_id = format!("arb/rayon_first/small_universe_many_updates/paths={path_count}/updates={n_updates}");

    // Test preparation & resources
    let (paths, symbols) = sample_paths("USDT", path_count).expect("path sampling failed");
    let updates = mock_updates(&symbols, n_updates);

    // Arb scanners
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());
    let rayon_best = RayonBestMatchScanner::new(paths.clone());
    let rayon_first = RayonFirstMatchScanner::new(paths.clone());

    c.bench_function(&naive_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = naive.process_update(u);
            }
        })
    });
    c.bench_function(&edge_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = edge.process_update(u);
            }
        })
    });
    c.bench_function(&rayon_best_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_best.process_update(u);
            }
        })
    });
    c.bench_function(&rayon_first_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_first.process_update(u);
            }
        })
    });  
}


fn bench_scanners_large_universe_few_updates(c: &mut Criterion) {
    // Test params
    let path_count = 100;
    let n_updates = 10;
    
    // Test ID's
    let naive_test_id = format!("arb/naive/large_universe_few_updates/paths={path_count}/updates={n_updates}");
    let edge_test_id = format!("arb/edge/large_universe_few_updates/paths={path_count}/updates={n_updates}");
    let rayon_best_test_id = format!("arb/rayon_best/large_universe_few_updates/paths={path_count}/updates={n_updates}");
    let rayon_first_test_id = format!("arb/rayon_first/large_universe_few_updates/paths={path_count}/updates={n_updates}");

    // Test preparation & resources
    let (paths, symbols) = sample_paths("USDT", path_count).expect("path sampling failed");
    let updates = mock_updates(&symbols, n_updates);
    
    // Arb scanners
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());
    let rayon_best = RayonBestMatchScanner::new(paths.clone());
    let rayon_first = RayonFirstMatchScanner::new(paths.clone());

    c.bench_function(&naive_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = naive.process_update(u);
            }
        })
    });
    c.bench_function(&edge_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = edge.process_update(u);
            }
        })
    });
    c.bench_function(&rayon_best_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_best.process_update(u);
            }
        })
    });
    c.bench_function(&rayon_first_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_first.process_update(u);
            }
        })
    });  
}


fn bench_scanners_large_universe_many_updates(c: &mut Criterion) {
    // Test params
    let path_count = 100;
    let n_updates = 500_000;
    
    // Test ID's
    let naive_test_id = format!("arb/naive/large_universe_many_updates/paths={path_count}/updates={n_updates}");
    let edge_test_id = format!("arb/edge/large_universe_many_updates/paths={path_count}/updates={n_updates}");
    let rayon_best_test_id = format!("arb/rayon_best/large_universe_many_updates/paths={path_count}/updates={n_updates}");
    let rayon_first_test_id = format!("arb/rayon_first/large_universe_many_updates/paths={path_count}/updates={n_updates}");
    
    // Test preparation & resources
    let (paths, symbols) = sample_paths("USDT", path_count).expect("path sampling failed");
    let updates = mock_updates(&symbols, n_updates);

    // Arb scanners
    let naive = NaivePrecompiledScanner::new(paths.clone());
    let edge = HashMapEdgeScanner::new(paths.clone());
    let rayon_best = RayonBestMatchScanner::new(paths.clone());
    let rayon_first = RayonFirstMatchScanner::new(paths.clone());

    c.bench_function(&naive_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = naive.process_update(u);
            }
        })
    });
    c.bench_function(&edge_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = edge.process_update(u);
            }
        })
    });
    c.bench_function(&rayon_best_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_best.process_update(u);
            }
        })
    });
    c.bench_function(&rayon_first_test_id, |b| {
        b.iter(|| {
            for u in black_box(&updates) {
                let _ = rayon_first.process_update(u);
            }
        })
    });  
}


criterion_group!(
    arb_benches,
    bench_scanners_small_universe_few_updates,
    bench_scanners_small_universe_many_updates,
    bench_scanners_large_universe_few_updates,
    bench_scanners_large_universe_many_updates,
);

criterion_main!(arb_benches);
