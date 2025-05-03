# 🔁 Arb Evaluation Methods

*A benchmark-driven exploration of arbitrage detection strategies across triangular paths in crypto markets.*

This document outlines the core evaluation strategies used (or planned) within the **TriArb** system to detect profitable triangular arbitrage opportunities from real-time price updates. Each method is designed with a trade-off in mind: simplicity, scalability, speed, or architectural elegance.

---

### 📂 Implemented & Planned Methods

* ✅ [`Naive Precompiled Triangle Scanner`](./src/arb/naive.rs)
* 🛠️ [`HashMap Edge Scan`](./src/arb/edge.rs) *(planned)*
* 🛠️ [`Multithreaded Scan with Rayon`](./src/arb/rayon_scan.rs) *(planned)*
* 🛠️ [`Delta-Based Scan`](./src/arb/delta.rs) *(planned)*
* 🛠️ [`SIMD Vectorized Evaluation`](./src/arb/simd.rs) *(planned)*

---

## ⚡ 1. **Naive Precompiled Triangle Scanner**

* **Precompute a fixed list** of triangle paths (e.g., `BTCUSDT → ETHBTC → ETHUSDT`)
* Every time a price update arrives, **evaluate all triangles** one-by-one
* Uses `DashMap` for internal state
* ✅ *Ideal for low-latency prototypes and small symbol sets*
* ❌ *Poor scalability as the number of triangles grows*

---

## ⚡ 2. **HashMap Edge Scan**

* **Model the market as a graph**, e.g. `HashMap<(Asset1, Asset2), TopOfBook>`
* On update:

  * Reconstruct paths dynamically based on connected edges
  * Example: scan outward from `USDT → BTC → ETH → USDT`
* ✅ *No need to precompile triangles*
* ❌ *Still O(n) traversal per update*

---

## ⚡ 3. **Multithreaded Scan with Rayon**

* Same triangle-based scan, but parallelized:

  * `triangles.par_iter().for_each(...)`
* ✅ *Linear performance scaling across CPU cores*
* ❌ *Slightly higher overhead; suboptimal for tiny symbol sets*

---

## ⚡ 4. **Delta-Based Scan**

* Track symbol-to-triangle **dependency map**:

  ```rust
  HashMap<Symbol, Vec<TriangleID>>
  ```
* On update, **only recompute triangles** affected by the changed symbol
* ✅ *Avoids redundant evaluations*
* ❌ *Requires careful graph indexing and setup*

---

## ⚡ 5. **SIMD Vectorized Evaluation**

* Pack price values into **CPU vector registers**
* Evaluate multiple triangles simultaneously using:

  * `std::simd`
  * `wide`
  * `packed_simd`
* ✅ *Max throughput for CPU-bound workloads*
* ❌ *Complex, brittle, hard to debug and align*

---

## 📊 Comparison Table

| Strategy              | Pros                               | Cons                                     | Best When                         |
| --------------------- | ---------------------------------- | ---------------------------------------- | --------------------------------- |
| **Naive Precompiled** | Simple, fast to prototype          | Scales poorly with more triangles        | Small universes                   |
| **HashMap Edge Scan** | Dynamic, path-flexible             | Still O(n), slightly more logic          | Medium universes                  |
| **Rayon Multithread** | CPU parallelism                    | Extra setup, best with high CPU count    | Large universes                   |
| **Delta-Based Scan**  | Evaluates only impacted paths      | Needs indexing logic                     | High-frequency updates            |
| **SIMD Evaluation**   | Ultimate performance (low latency) | Very hard to implement, debug, and align | Hardcore performance environments |

---

## ⚠️ Disclaimer

> This is a **hobby / research project**.
> It does **not execute trades**, connect to user accounts, or offer financial advice.
> Designed purely for learning and exploration.

---
