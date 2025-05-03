# 🔌 WebSocket Payload Handling: Copy vs Borrow

This document explores how to handle incoming WebSocket frames from Binance efficiently, specifically focusing on whether to **copy** or **borrow** the payload bytes.

---

## 🎯 Goal

Avoid unnecessary allocations by using the WebSocket’s existing byte buffer (`&[u8]`) **without copying** into a new `Bytes` object — for maximum throughput and minimal memory pressure.

---

## 🚀 Current Approach (Safe Default)

We currently:

* Receive a `fastwebsockets::Payload::Borrowed(&[u8])`
* Copy it into `Bytes` via `Bytes::copy_from_slice`
* Send it through an async channel to a parser task

✅ **Safe**
✅ **Fast enough for current scale**
❌ **Includes minor memory copy overhead (\~50–100ns/msg)**

---

## 🧠 The Optimized Option: Zero-Copy Parsing

### How it would work:

| Step                                                       | Action                  |
| :--------------------------------------------------------- | :---------------------- |
| ✅ Payload is already `Borrowed(&[u8])`                     | No need to allocate     |
| 🧠 Pass the slice `&[u8]` directly into the parser         | Avoids copying          |
| ❗ Must parse **immediately** before reading the next frame | Prevents use-after-free |
| ❌ Can’t safely send the `&[u8]` over a channel             | It will become invalid  |

---

## ⚠️ Why It's Tricky

* `fastwebsockets` reuses its internal read buffer
* Once you call `.read_frame()` again, the previous payload's memory may be overwritten
* This makes `&[u8]` **dangerous to hold across frames**

✅ The only safe time to parse borrowed bytes is **immediately** in the WebSocket loop
❌ Any delay (e.g., via channel or background thread) requires cloning the bytes

---

## 🛠️ What You’d Need to Change

### Option A – **Zero-Copy Inline Parse** (High-efficiency mode)

```rust
loop {
    let frame = ws.read_frame().await?;
    if let Payload::Borrowed(data) = frame.payload {
        parse(&data); // Parse immediately here
    }
}
```

### Option B – **Safe Channel Parse** (What we do now)

```rust
let frame = ws.read_frame().await?;
if let Payload::Borrowed(data) = frame.payload {
    tx.send(Bytes::copy_from_slice(data)).await?; // Copy, then parse in another task
}
```

---

## 🔄 Tradeoff Summary

| Strategy                    | Pros                                     | Cons                        |
| :-------------------------- | :--------------------------------------- | :-------------------------- |
| **Copy into `Bytes`**       | ✅ Safe, clean, easy to send across tasks | ❌ Minor overhead            |
| **Borrow and parse inline** | ✅ Max performance, zero alloc            | ❌ Risky, tightly coupled    |
| **Use `Arc<[u8]>` instead** | ✅ Shared ownership, still fast-ish       | ❌ Heap alloc still required |

---

## 📌 TL;DR

| When to Copy               | When to Avoid Copy                                      |
| :------------------------- | :------------------------------------------------------ |
| ✅ You’re using channels    | ❌ You parse immediately inline                          |
| ✅ You want safe lifetimes  | ❌ You’re micro-optimizing for 10k+ msgs/sec             |
| ✅ You’re still prototyping | ❌ You’ve benchmarked and proven copy is your bottleneck |

---

## 📚 Future Optimizations (When Needed)

* Parse directly on `&[u8]`
* Consider `memchr` + SIMD acceleration
* Explore zero-copy parsing lifetimes
* Push parse + eval into the same task for tight loops

---

## 🔥 Final Note

> *Copying a few hundred nanoseconds per message is not your bottleneck — until it is.*

Until benchmarks prove otherwise:
✅ Stick with `Bytes::copy_from_slice` — it’s safe, fast, and flexible.
