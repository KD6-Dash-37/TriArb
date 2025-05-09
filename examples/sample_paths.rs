// examples/sample_paths.rss
use tri_arb::devtools::path_sampler::sample_paths;

fn main() -> anyhow::Result<()> {
    let home_asset = "USDT";
    let path_count= 50;

    let (paths, symbols) = sample_paths(home_asset, path_count)?;

    println!("✅ Sampled {} pricing paths starting/ending in {}", paths.len(), home_asset);
    println!("🔢 Unique symbols involved: {}", symbols.len());

    println!("\n🧠 Example paths:");
    for (i, path) in paths.iter().take(5).enumerate() {
        println!("{}. {}", i + 1, path);
    }

    println!("\n🔠 All symbols:");
    for sym in &symbols {
        println!("  - {}", sym);
    }

    Ok(())
}