// benches/all.rs

mod parse;
mod arb;

use criterion::criterion_main;

use arb::arb_benches;
use parse::parse_benches;

criterion_main!(
    arb_benches,
    parse_benches,
);
