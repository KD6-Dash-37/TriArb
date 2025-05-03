# 🧩 Parsing Architecture & Performance Exploration

This module processes incoming Binance `bookTicker` WebSocket messages into structured `TopOfBookUpdate` events. Parsing speed is critical, as every message must be processed before arbitrage evaluation can occur.

We're actively exploring the tradeoffs between simplicity, safety, and raw performance through multiple parsing strategies — from fully safe `serde_json` to hand-tuned byte scanning and eventually SIMD acceleration.

---

### 📂 Implemented & Planned Methods

* ✅ [`SerdeJsonParser`](./src/parse/srd_jsn.rs) — baseline using `serde_json`
* ✅ [`ManualScanParser`](./src/parse/man_scan.rs) — string scanning for speed
* 🛠️ [`ByteScanParser`](./src/parse/byte_scan.rs) *(planned)* — operate on raw `&[u8]`
* 🛠️ [`SIMDParser`](./src/parse/simd.rs) *(planned)* — use `memchr`/SIMD for fast searches
* 🛠️ [`ZeroCopyParser`](./src/parse/zero_copy.rs) *(planned)* — advanced, no allocations

---


## 📦 Current Parser Implementations

### 🔹 **1. SerdeJsonParser**

**Description:** Uses `serde_json` to deserialize into a strongly-typed Rust struct.

* ✅ Easiest to implement
* ✅ Very safe (type-checked)
* ❌ Slower due to full JSON parsing and validation
* ❌ Heap allocations per message

**File:** [`srd_jsn.rs`](./srd_jsn.rs)
**Bench ID:** `single_parse_serde_json`, `batch_parse_serde_json`

### 🔹 **2. ManualScanParser**

**Description:** Uses manual string scanning to extract key fields from the raw message string.

* ✅ Much faster than `serde_json`
* ✅ Lower allocations (just string slices and parse)
* ❌ Harder to maintain
* ❌ Less robust to unexpected message formats

**File:** [`man_scan.rs`](./man_scan.rs)
**Bench ID:** `single_parse_manual_scan`, `batch_parse_manual_scan`

---

## 🧪 Benchmark-Driven Comparison

See: [`benches/parser_bench.rs`](../../benches/parser_bench.rs)

```bash
cargo bench --bench parser_bench
```

---

## 🧠 Upcoming Parser Variants

These variants are planned for future performance exploration:

### ⚡ **3. ByteScanParser** (Planned)

* Search directly in `&[u8]` using precomputed byte patterns (e.g., `br#""s":""#`)
* Avoid UTF-8 decoding altogether
* May use `slice::windows()` or raw pointer indexing

> 🏁 Goal: \~2–3x faster than string scanning

---

### ⚡ **4. SIMDParser** (Planned)

* Use `memchr` and SIMD-enhanced byte search
* Potentially parse 16–32 bytes at once
* Combine with raw float parsing to skip `str::parse::<f64>()`

> 🏁 Goal: compete with top-tier event-driven parsers (e.g. `simd-json`, `nom`)

---

### ⚡ **5. Zero-CopyParser** (Aspirational)

* Avoid `String`/`Bytes::copy_from_slice()` entirely
* Keep payload lifetime tied to original buffer
* Use `&str` or `&[u8]` directly without cloning

> 🧠 Advanced lifetime gymnastics — might be too complex for now

---

## 🧰 Trait-Based Design

All parsers implement:

```rust
trait BookTickerParser {
    fn parse(&self, raw: &Bytes) -> Result<TopOfBookUpdate>;
}
```

> ✅ Allows swapping parsers at runtime
> ✅ Makes benchmarking and testing each parser easy
> ✅ Only instantiated once at startup

The `parser_loop()` handles channel input + parsing:

```rust
pub async fn parser_loop(
    rx: Receiver<Bytes>,
    price_store: Arc<DashMap<String, TopOfBookUpdate>>,
)
```
