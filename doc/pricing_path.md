# 🧭 Triangle Discovery from Exchange Metadata

This module builds all valid 3-leg triangular arbitrage paths from Binance `exchangeInfo` metadata. It performs a one-time construction of **pricing paths**, each representing a sequence of tradable pairs that begins and ends in a designated home asset (e.g. `USDT`), traversing two intermediate currencies.

Each path includes:

* The trading pair (symbol)
* The correct price side (`Bid` or `Ask`) for evaluating the route directionally

The result is a clean, type-safe representation of executable arbitrage paths, ready to be consumed by evaluation engines or real-time pricing systems.

🔍 Focus areas:

* **Correctness-first design** — favors clarity and safety over speed during initialization.
* **Minimal lifetime complexity** — path data is fully owned, enabling ergonomic downstream use.
* **Easy integration** — `build_all_paths()` provides a single entry point to extract all valid opportunities.

Use this module to bootstrap your arbitrage engine with a consistent and trustworthy universe of opportunities.
