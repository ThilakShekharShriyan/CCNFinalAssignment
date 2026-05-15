# edge-cache-sim

Discrete-event style simulator for **collaborative edge chunk caching** with nearest-neighbour-then-origin routing. Policies: **LRU**, **LFU**, **Closer** (first-chunk + popularity biased eviction), **P99Aware** (sliding-window tail proxy for eviction).

## Test

```bash
cargo test
```

Uses [`configs/smoke.toml`](configs/smoke.toml) for fast integration tests (~1s). Full defaults are validated on load.

## Run

From this directory:

```bash
cargo run --release -- --config configs/default.toml
```

Quick run (smaller catalogue / fewer requests):

```bash
cargo run --release -- --config configs/smoke.toml
```

## Demo mode (recommended for presentation)

Pre-built figures for reviewers: [../artifacts/simulator-demo/](../artifacts/simulator-demo/).

Generate a fresh demo pack (CSVs + plots + markdown summary) in one command:

```bash
bash scripts/demo_run.sh
```

This creates `results/demo_pack_<timestamp>/` (gitignored) with:
- `cache_sweep.csv`, `burst_sweep.csv`, `hetero_sweep.csv`, `unit_interval_sweep.csv`
- `cache_mean.png`, `cache_hit.png`, `burst_mean.png`, `unit_interval_mean.png` (if pandas/matplotlib installed)
- `DEMO_SUMMARY.md` with talking points

Dependencies for plots:

```bash
pip install -r scripts/requirements-plot.txt
```

## Phase 2: multi-chunk workload + policy sweep

- Set **`chunks_per_object`** (>1) and **`chunk_non_head_prob`** in TOML so some requests target non-head chunks (see [`configs/phase2.toml`](configs/phase2.toml)).
- **`--sweep-policies`** runs **lru, lfu, closer, p99aware** with the same scenario and writes a fresh CSV (**requires `--csv-out`**):

```bash
mkdir -p results
cargo run --release -- --config configs/phase2.toml --sweep-policies --csv-out results/phase2_policies.csv
```

CSV columns include `chunks_per_object`, `chunk_non_head_prob`, `cache_capacity_chunks`, `heterogeneous_backhaul`, seeds, and latency/hit/Jain metrics for plotting in Python or Numbers.

## Phase 4: prediction-noise sweep (`--sweep-noise-sigmas`)

Adds another dimension to batch runs (noise only changes **P99Aware** eviction scores; other policies repeat for a controlled comparison).

```bash
cargo run --release -- --config configs/phase3_base.toml \
  --sweep-policies --sweep-noise-sigmas 0,0.05,0.1,0.2 \
  --csv-out results/phase4_noise_sweep.csv

python3 scripts/plot_sweep.py results/phase4_noise_sweep.csv --x prediction_noise_sigma --metric mean_ms --het any --out results/mean_vs_noise.png
```

CSV includes column `prediction_noise_sigma`.

## Phase 5: burstiness sweep (`--sweep-burst-high-mults`)

Sweeps request burst intensity while keeping other config values fixed. This targets the burstiness IV directly.

```bash
cargo run --release -- --config configs/phase3_base.toml \
  --sweep-policies --sweep-burst-high-mults 1,2,4,6 \
  --csv-out results/phase5_burst_sweep.csv

python3 scripts/plot_sweep.py results/phase5_burst_sweep.csv --x burst_high_rate_mult --metric mean_ms --het any --out results/mean_vs_burst.png
```

CSV includes columns `burst_high_rate_mult`, `burst_low_rate_mult`, and `burst_toggle_mean_requests`.

## Phase 6: load calibration (`--sweep-unit-intervals`)

Calibrates offered load by sweeping base inter-arrival interval. This keeps burst/topology fixed while controlling queue pressure.

```bash
cargo run --release -- --config configs/phase3_base.toml \
  --sweep-policies --sweep-unit-intervals 60,120,180 \
  --csv-out results/phase6_unit_interval_sweep.csv

python3 scripts/plot_sweep.py results/phase6_unit_interval_sweep.csv --x request_unit_interval --metric mean_ms --het any --out results/phase6_mean_vs_unit_interval.png
```

Batch mode supports combining this with other sweeps, e.g. burst × unit interval.

## Phase 3: cache-size and backhaul sweeps + plots

**Independent variables** (aligned with the research scope doc): cache budget, heterogeneous backhaul; combine with policy sweep.

- **`--sweep-cache-sizes N,N,N`** — repeat the run for each capacity (comma-separated).
- **`--sweep-heterogeneous`** — run once with `heterogeneous_backhaul = false`, then `true`.
- Batch mode (**any** of `--sweep-policies`, `--sweep-cache-sizes`, `--sweep-heterogeneous`, `--sweep-noise-sigmas`, `--sweep-burst-high-mults`, `--sweep-unit-intervals`) **requires** `--csv-out` and overwrites that file.

Examples:

```bash
# 4 policies × 4 cache sizes = 16 rows (homogeneous only)
cargo run --release -- --config configs/phase3_base.toml \
  --sweep-policies --sweep-cache-sizes 36,54,72,90 \
  --csv-out results/p3_cache_sweep.csv

# Add heterogeneous dimension: 2 × 4 × 4 = 32 rows
cargo run --release -- --config configs/phase3_base.toml \
  --sweep-policies --sweep-cache-sizes 36,54,72,90 --sweep-heterogeneous \
  --csv-out results/p3_full_sweep.csv
```

**Figure from CSV** (optional Python):

```bash
pip install -r scripts/requirements-plot.txt
python3 scripts/plot_sweep.py results/p3_cache_sweep.csv --metric p99_ms --het any --out results/p99_vs_cache.png
```

Override policy in a copy of the config or edit `policy` (`lru`, `lfu`, `closer`, `p99aware`).

Append one CSV summary row per run:

```bash
cargo run --release -- --config configs/default.toml --csv-out results/runs.csv
```

## What is simulated

- **Topology:** full mesh among `num_edge_nodes` edge caches + symmetric **origin** links (configurable heterogeneity on half the nodes).
- **Demand:** Zipf object popularity, users pinned to a home edge, **on/off burst** scaling of request rate (abstract; does not change ordering of the stream for a fixed seed—burst state advances each request).
- **Metric:** time to fetch the **first chunk** (here one chunk per object) including propagation + serialization at path bottleneck.
- **Warmup:** first `warmup_requests` fill caches but are excluded from printed metrics.

See [../docs/SCOPE_LOCK.md](../docs/SCOPE_LOCK.md) for the frozen experiment contract.

## Real deployment prototype (Mininet + local stack)

See [deployment/README.md](deployment/README.md) and [deployment/API_CONTRACT.md](deployment/API_CONTRACT.md).

### Build service binaries

```bash
cargo build --release --bins --target-dir target
```

### Local stack (origin + 3 edges + controller)

```bash
bash deployment/local/start_stack.sh
target/release/workload_replay --edges http://127.0.0.1:7001,http://127.0.0.1:7002,http://127.0.0.1:7003 --requests 2000 --csv-out results_real/replay.csv
curl -s http://127.0.0.1:7001/metrics
bash deployment/local/stop_stack.sh
```

### Protocol comparison (HTTP vs gRPC upstream)

```bash
bash deployment/experiments/protocol_compare.sh
```

Outputs:

- `results_real/protocol_compare/replay_http.csv`
- `results_real/protocol_compare/replay_grpc.csv`
- `results_real/protocol_compare/protocol_compare_summary.md`

### Sim-vs-real validation sweep

```bash
bash deployment/experiments/run_validation_sweeps.sh
```

Outputs:

- `results_real/validation/sim_unit_sweep.csv`
- `results_real/validation/real_unit_sweep.csv`
- `results_real/validation/sim_real_validation.md`
