# Memory Profiling and Optimization

## Goal
Provide a reproducible memory profiling workflow for the repository, identify hotspots in both contract and front-end code, and document improvements after optimization.

## Scope
- Smart contract memory usage in the Soroban/Rust stack
- Front-end and Node memory patterns for the interface
- Local profiling tools and measurements

## Setup

### Rust contract profiling

1. Install profiling tools:
```bash
cargo install cargo-llvm-cov
cargo install cargo-audit
cargo install cargo-geiger
sudo apt-get install -y valgrind lcov
```

2. Build and run the contract test profile:
```bash
cd contracts/crowdfund
cargo test -- --nocapture
```

3. Use a memory profiler to inspect test execution:
- `valgrind --tool=massif --massif-out-file=massif.out cargo test -- --nocapture`
- `massif-visualizer massif.out`

### JavaScript / TypeScript profiling

1. Use Node.js built-in profiler for backend or scripts:
```bash
node --inspect-brk ./path/to/script.js
```

2. Use browser devtools for interface memory profiling.

## Identify Memory Hotspots

### Contract-side patterns
- Frequent allocations in event payloads, strings, and vectors
- Repeated storage reads/writes for per-contributor state
- Large persistent structures stored inside instance storage

### Front-end patterns
- Long-lived arrays or large JSON payloads
- Unbounded caching in memory
- Inefficient DOM rendering loops

## Optimization Guidance

### Smart contract memory
- Batch instance writes when possible to reduce repeated storage overhead
- Minimize use of dynamic `Vec` and `String` values in hot code paths
- Prefer compact keys and remove entries when they are no longer required
- Use `Option`/`None` to avoid storing empty collections

### Persistent storage
- Store only required fields for each contributor
- Avoid duplicate state where one key can infer another
- Compress or shorten key namespaces when using structured storage keys

## Measure Improvements

### Before and after
- Record baseline memory usage metrics from a representative workload
- Re-run profiles after each optimization
- Compare results using the same inputs and environment

### Suggested metrics
- Resident Set Size (RSS)
- peak heap usage
- allocation count
- contract storage size and entry count

## Document Findings

Keep a running log of the following for each optimization:
- What was changed
- Which hotspot was targeted
- How the change affected memory usage
- Any tradeoffs or risks introduced

## Example Workflow

1. Baseline: profile the existing contract using `massif`
2. Identify: review the largest allocations and hot call paths
3. Optimize: remove or compress unneeded memory use
4. Validate: rerun tests and profiles
5. Document: add the findings to this file or a dedicated `docs/` note
