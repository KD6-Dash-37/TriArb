# 🔁 TriArb: Real-Time Triangular Arbitrage Engine

TriArb is a hobby project exploring real-time **triangular arbitrage** opportunities in cryptocurrency markets. The goal isn’t just to detect arbitrage — it’s to **discover the best way to do it** through iterative development and deep benchmarking.

This project is designed to help answer questions like:

* *What’s the fastest way to parse streaming data?*
* *How should we structure pricing paths to be both safe and efficient?*
* *Which arb evaluation strategy scales best: precompiled loops, hash maps, deltas, or SIMD scans?*

## 🚧 Disclaimer

**This project does *not* include trade execution, order placement, or connectivity to live accounts.**
It is a research and development effort meant for educational purposes only. Do not use it for trading real funds.

## ✨ What We’ve Built So Far

### 📡 WebSocket Ingestion

* **Powered by** [`fastwebsockets`](https://crates.io/crates/fastwebsockets) for ultra-low latency.
* Subscribes to Binance's `bookTicker` stream for a predefined set of symbols.
* Efficiently decodes raw frames and dispatches data to a parsing queue via `tokio::mpsc`.

### 🧩 Modular Parsers

* Abstracted via the `BookTickerParser` trait — switchable via Cargo feature flags.
* ✅ **Serde JSON Parser**: for correctness and reliability.
* ⚡ **Manual Byte Scanner**: handcrafted and 20–30% faster in benchmarks.
* 📈 Benchmarked with `criterion` across both single-message and batch parsing loads.

### 🔁 Pricing Paths & Universe Construction

* Parses Binance `exchangeInfo` fixture.
* Discovers all **valid 3-leg triangular paths** starting and ending in a "home" asset (e.g. USDT).
* Each path is assigned a direction (`Bid` or `Ask`) based on trade flow.

### 🧠 Arb Evaluators

Choose between multiple arbitrage evaluation strategies, each designed for different performance and complexity tradeoffs:

* ✅ [`Naive Precompiled Scanner`](./src/arb/naive.rs)  
* ✅ [`HashMap Edge Scanner`](./src/arb/edge.rs)  
* 🛠️ [`Multithreaded Scan with Rayon`](./src/arb/rayon_scan.rs) *(planned)*
* 🛠️ [`Delta-Based Scan`](./src/arb/delta.rs) *(planned)*  
* 🛠️ [`SIMD Vectorized Evaluation`](./src/arb/simd.rs) *(planned)*  

### 🚀 Benchmarking

* Integrated via `criterion`.
* Benchmarks both Serde and manual parsers for:

  * Single-message throughput
  * 100K+ message batch parsing
* Designed to help track performance gains over time and inform parser architecture decisions.

---

## 💡 Design Philosophy

* **Correctness-first**: especially in universe construction and trading logic.
* **Composability**: everything is modular and trait-based — plug-and-play architecture.
* **Benchmark-backed**: all parsing logic is testable, swappable, and measurable.
* **Async-native**: runs entirely on `tokio` without blocking.

---
