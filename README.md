# Fast Count-Min Sketch (CMS) in Rust

A high-performance, memory-efficient, and zero-allocation implementation of the **Count-Min Sketch** algorithm in Rust. Optimized for CPU cache hierarchy and designed for massive data stream processing.

---

## TOC
1. [Introduction](#introduction)
2. [How It Works](#how-it-works)
3. [Mathematical Principles](#mathematical-principles)
4. [Technical Features](#technical-features)
5. [Installation & Usage](#installation--usage)
6. [Complexity Analysis](#complexity-analysis)
7. [Memory Profiling](#memory-profiling)
8. [Testing & Benchmarking](#testing--benchmarking)

---

## Introduction

The **Count-Min Sketch** is a probabilistic data structure (a "sketch") used to estimate the frequency of events in a data stream. Unlike a standard hash table, CMS uses fixed **sub-linear** space at the cost of a small, controlled over-estimation (false positives). It **never** under-estimates the true count.

It is ideal for:
- Identifying "Heavy Hitters" (frequent items).
- Network bandwidth monitoring.
- Real-time fraud detection and DoS attack prevention.

---

## How It Works

The CMS consists of a matrix of counters with width $w$ and depth $d$.
1. Each row in the matrix is associated with an independent hash function.
2. **Update (`increment`):** An item is passed through $d$ hash functions. Each function maps the item to a specific column in its respective row. The counter at that position is incremented.
3. **Query (`estimate`):** The same $d$ positions are calculated. The estimated value is the **minimum** among the counters found. Since collisions can only increase counter values, the minimum is the closest possible estimate to the true frequency.



---

## Mathematical Principles

The user defines the desired accuracy via two parameters:

- **$\epsilon$ (Epsilon):** Defines the maximum acceptable error. The estimate $\hat{a}_i$ respects the condition $\hat{a}_i \leq a_i + \epsilon N$, where $N$ is the total number of increments.
- **$\delta$ (Delta):** Defines the probability that the error exceeds the $\epsilon$ threshold. The confidence is $1 - \delta$.

**Automatic Sizing:**
- **Width ($w$):** $\lceil e / \epsilon \rceil$ (automatically rounded to the next power of two for performance).
- **Depth ($d$):** $\lceil \ln(1 / \delta) \rceil$.

---

## Technical Features

### 1. Zero-Allocation updates
By using a static helper for index calculation, the `increment` and `estimate` functions perform **zero heap allocations** during execution. This drastically reduces execution time and eliminates system allocator overhead or fragmentation.

### 2. Hash Network & Double Hashing
To reduce the computational cost of hashing, we use a single high-quality 64-bit hash produced by `ahash`. From this, we derive $d$ independent indices using the **SplitMix64** mixer. This ensures uniform distribution at the cost of only one primary hash calculation.

### 3. Saturating Counters
Counters use `u64` with `saturating_add` logic. Under extreme data loads, the counter stops at the `u64::MAX` value instead of wrapping around to zero, preserving the statistical integrity of the sketch.

### 4. Bitwise Masking
Instead of using the modulo operator (`%`), which is CPU-expensive, we force the sketch width to be a power of two. This allows us to use the much faster bitwise `&` operator to map hashes into buckets.

---

## Installation & Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
ahash = "0.8"
```

## Quick Example
This example shows how to initialize the sketch using statistical error bounds and track several items.

```rust
fn main() {
    // error 0.001 * iteration with confidence of 99%
    let mut cms = CountMinSketch::with_params(0.001, 0.01);
    
    let stream = vec!["apple", "banana", "apple", "cherry", "apple", "banana"];
    for item in stream {
        cms.increment(item);
    }

    println!("Apple count (est):  {}", cms.estimate("apple"));  // Likely 3
    println!("Banana count (est): {}", cms.estimate("banana")); // Likely 2
    println!("Gianluca count (est): {}", cms.estimate("Gianluca")); // 0 (if no collisions)

    cms.clear();
    assert_eq!(cms.estimate("apple"), 0);
}
```
