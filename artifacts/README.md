# Submission artifacts

Curated outputs for the CCN final project. Regenerate locally; do not edit by hand.

## Simulator (`simulator-demo/`)

Discrete-event sweep results and figures from:

```bash
cd edge-cache-sim
bash scripts/demo_run.sh ../artifacts/simulator-demo
```

| File | Description |
|------|-------------|
| `DEMO_SUMMARY.md` | Auto-generated talking points |
| `*_sweep.csv` | Policy sweeps (cache, burst, heterogeneity, load) |
| `*.png` | Mean latency / hit-rate plots |

## Real deployment (`real-deployment/`)

Local stack validation (optional; not required for the core simulation grade). After building binaries:

```bash
cd edge-cache-sim
cargo build --release --bins
bash deployment/experiments/real_only_runner.sh
```

Copy summaries and aggregate CSVs from `results_real/latest/` into this folder before submitting evidence from the live prototype.
