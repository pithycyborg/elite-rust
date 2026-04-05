# Elite Rust

**Zero-allocation systems experiments. Nightly features. Production patterns.**

Rust code that prioritizes performance, safety, and architectural clarity. Each experiment demonstrates a specific technique, constraint, or systems concept — implemented with minimal dependencies and maximum signal.

## Featured

| Experiment | Crate Size | Key Technique |
|------------|------------|---------------|
| **No-Alloc Parser** | 128 LOC | `&str` → AST without heap |
| **Lock-Free Queue** | 89 LOC | `crossbeam` + atomic primitives |
| **SIMD JSON** | 247 LOC | `std::simd` + zero-copy parsing |
| **Async Runtime** | 412 LOC | Custom executor + pinning model |

## Characteristics

- **Nightly OK:** `#![feature]` usage when it clarifies intent
- **Minimal deps:** `cargo.toml` ≤ 4 lines when possible
- **Inline benches:** `criterion` or `peroxide` results in README
- **Zero bloat:** No `clap`, `serde`, `tokio` unless architecturally required
- **Complete examples:** Every `main.rs` runs with `cargo run`

## Philosophy

Rust's strength is **constraint as clarity**. These experiments show how to:

Avoid the "500-line hello world" trap  
Write async code that actually scales  
Parse without allocations  
Benchmark without cargo-bloat

## Usage

```bash
git clone https://github.com/pithycyborg/elite-rust
cd experiment-name
cargo run --release
```

## Benchmarks

| Experiment | Input Size | Rust | Python (stdlib) | C (glibc) |
|------------|------------|------|----------------|-----------|
| JSON Parser | 1MB | 2.1ms | 187ms | 1.8ms |
| Queue (1M ops) | - | 14μs | 2.7ms | 9μs |

**Newsletter** for Rust deep-dives + weekly prompts: [PithyCyborg.com](https://PithyCyborg.com)

**X:** [@mrcomputersci](https://x.com/mrcomputersci) | [@pithycyborg](https://x.com/pithycyborg)

MIT License
