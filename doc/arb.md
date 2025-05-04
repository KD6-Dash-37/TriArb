# 🔁 Arb Evaluation Methods

*A benchmark-driven exploration of arbitrage detection strategies across triangular paths in crypto markets.*

This document outlines the core evaluation strategies used (or planned) within the **TriArb** system to detect profitable triangular arbitrage opportunities from real-time price updates. Each method is designed with a trade-off in mind: simplicity, scalability, speed, or architectural elegance.

---

### 📂 Implemented & Planned Methods


* ✅ [`Naive Precompiled Scanner`](./src/arb/naive.rs)  
  Iterates over all pricing paths every time — simple, slow, and great for correctness testing.

* ✅ [`HashMap Edge Scanner`](./src/arb/edge.rs)  
  Maintains a reverse index of symbols to relevant paths — only scans what changed. Designed for scalability and low-latency use.

* 🛠️ [`Multithreaded Scan with Rayon`](./src/arb/rayon_scan.rs) *(planned)*  
  Parallelizes path evaluation using Rayon — ideal for burst-heavy scenarios.

* 🛠️ [`Delta-Based Scan`](./src/arb/delta.rs) *(planned)*  
  Propagates changes through minimal deltas — avoids recomputation where possible.

* 🛠️ [`SIMD Vectorized Evaluation`](./src/arb/simd.rs) *(planned)*  
  Uses SIMD to batch-evaluate path profitability — targeting peak throughput on modern CPUs.


---

## ⚡ 1. **Naive Precompiled Triangle Scanner**

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


### 🛠️ How the HashMap Edge Scanner Works

The `HashMapEdgeScanner` is a real-time arbitrage evaluator designed for **efficient path filtering**. Rather than checking all possible triangular pricing paths on every update, it evaluates **only the paths relevant to the updated symbol**. This is a significant optimization over naive approaches.

---

### 🧩 Components and Flow

#### 1. **Path Indexing (Preprocessing Step)**

At initialization (`new`):

* The scanner receives a list of `PricingPath` objects.
* Each path contains 3 legs, and each leg uses a market symbol (e.g. `ETHBTC`, `BTCUSDT`, etc.).
* For each path, the scanner:

  * Wraps it in an `Arc` for cheap cloning across shared references.
  * Indexes it by each of its symbols in a `HashMap<String, Vec<Arc<PricingPath>>>`.

✅ Result: On every update to a symbol, we can instantly retrieve **only the relevant paths** via `path_index`.

#### 2. **State Management**

The scanner maintains an in-memory order book snapshot via a `DashMap<String, TopOfBookUpdate>`:

* When a new `TopOfBookUpdate` is received, it updates this map.
* The map is thread-safe and lock-free — designed for concurrent access.

#### 3. **Efficient Arb Evaluation**

During `process_update`:

* The scanner:

  * Finds all paths that depend on the updated symbol via `path_index`.
  * Skips early if any of the required symbols haven't yet been seen.
  * Executes a 3-leg arbitrage simulation (`START -> step1 -> step2 -> end`) based on the path’s side (bid/ask).
  * If a profitable arb is found (`end > START`), it returns `Some((path, end))`.

✅ This enables the engine to **react only to meaningful data**.

---

### 🧠 Design Advantages

* **Symbol-based filtering** via `HashMap` avoids unnecessary recomputation — critical for scaling to 1000s of pairs.
* **Arc wrapping** allows safe and lightweight sharing of paths.
* **DashMap** for live quote storage ensures thread safety with minimal locking overhead.
* **Side-aware simulation** handles both `BID`/`ASK` logic cleanly and consistently.

---

### 🔍 Opportunities for Improvement

---

#### ➡️  **Pre-warming Symbol Cache**

Until all 3 required symbols have been received, the arb evaluation is skipped.

**Improvement:** Allow pre-initialization of `DashMap` with known symbols to avoid missed arbs at startup.

---

#### ➡️ **Clone Reduction**

Currently, we clone the entire `PricingPath` when returning it.

**Improvement:** Return an `Arc<PricingPath>` to avoid unnecessary deep clones.

---

#### ➡️ **Multithreaded Path Evaluation**

If symbol volumes grow, the `for path in paths` loop can be parallelized.

**Improvement:** Consider using `rayon::par_iter()` over `paths` for parallel arb detection.


---

## ⚡ 3. **Multithreaded Scan with Rayon**

* Same triangle-based scan, but parallelized:

  * `triangles.par_iter().for_each(...)`
* ✅ *Linear performance scaling across CPU cores*
* ❌ *Slightly higher overhead; suboptimal for tiny symbol sets*

---

## ⚡ 4. **Delta-Based Scan**

* Track symbol-to-triangle **dependency map**:

  ```text
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
