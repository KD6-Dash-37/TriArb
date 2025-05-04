# 🔌 WebSocket Client Overview (Binance + Mock Feed)

This module provides a flexible WebSocket client that can connect to either:

* ✅ **Binance** (`wss://data-stream.binance.com`)
* 🧪 **Mock local feed** (`ws://localhost:9001`)

It handles **subscription**, **frame decoding**, and **frame forwarding** via an async channel — supporting both real-time trading and full-system testing.

---

## 🔧 Features

| Feature                           | Status | Description                                                     |
| --------------------------------- | ------ | --------------------------------------------------------------- |
| Real Binance connectivity         | ✅      | Secure TLS over `wss://`, with handshake                        |
| Mock server integration           | ✅      | Local testing over plain TCP `ws://localhost:9001`              |
| Symbol auto-subscription          | ✅      | Based on pricing path analysis                                  |
| Configurable connection mode      | ✅      | `use_mock: bool` passed at runtime                              |
| Safe message forwarding via Bytes | ✅      | Converts incoming payloads into `Bytes` for safe cross-task use |

---

## 🚀 Usage

### Real Binance connection:

```rust
start_ws_listener(paths, tx, false).await?;
```

### Mock server for testing:

```rust
start_ws_listener(paths, tx, true).await?;
```

---

## 🧪 Mock Feed Compatibility

When using the mock server:

* It accepts Binance-style `SUBSCRIBE` messages
* Emits `bookTicker`-formatted JSON from a hot cache
* Fully compatible with the real client code

Ideal for:

* Integration tests
* Parser benchmarks
* Offline development

---

# 🧠 Payload Handling: Copy vs Borrow

This section explains the design choices around payload memory — specifically, **copying** vs **borrowing** WebSocket frame data.

---

## ✅ Current Strategy: Safe Copy

We use:

```rust
Bytes::copy_from_slice(&frame.payload)
```

This makes the data:

* Owned
* Safe to move between threads
* Compatible with channels

✔️ Zero risk of buffer reuse bugs
✔️ Works well with downstream async parsing
❌ Small memory copy cost (\~50–100ns/msg)

---

## ⚡️ Future Option: Zero-Copy Parsing

To remove the copy:

```rust
if let Payload::Borrowed(data) = frame.payload {
    parse_inline(data); // Must happen before next frame is read
}
```

⚠️ Cannot send `&[u8]` over channels — it's tied to a reused read buffer.

Use only if:

* You **parse synchronously**
* You are optimizing for **ultra-high throughput** (e.g. >50K msg/s)
* You avoid buffering or dispatching `&[u8]`

---

## 🔄 Strategy Comparison

| Method                   | Safe to share? | Copy-Free? | Thread-safe? | Fast? |
| ------------------------ | -------------- | ---------- | ------------ | ----- |
| `Bytes::copy_from_slice` | ✅              | ❌          | ✅            | ✅     |
| `&[u8]` borrowed slice   | ❌              | ✅          | ❌            | ✅✅    |
| `Arc<[u8]>`              | ✅              | ❓          | ✅            | ✅     |

---

## 📌 Recommendation

Stick with the safe-copy model (`Bytes`) unless benchmarks prove otherwise.

Use zero-copy **only when**:

* You parse inline
* You avoid forwarding the frame
* You've benchmarked for performance bottlenecks

---

## 🛠 Future Improvements

* Push parsing into the read loop for a zero-copy path
* Add a runtime toggle (copy vs borrow mode)
* Consider `Arc<[u8]>` for hybrid performance
* Benchmark across hardware and workloads

---

## 🧪 Diagnostic Logs

* 🧪 Connecting to local: `localhost:9001`
* 🌐 Connecting to Binance: `data-stream.binance.com:9443`
* 📨 Subscribed symbols printed at runtime
