# Real-deployment artifacts (optional)

This directory holds **summaries and figures** from the localhost edge stack (`deployment/local/`). Per-run replay traces stay under `edge-cache-sim/results_real/` (gitignored).

## Regenerate

```bash
cd edge-cache-sim
cargo build --release --bins
bash deployment/experiments/real_only_runner.sh
```

Then copy into this folder:

- `results_real/latest/protocol_compare/protocol_compare_summary.md`
- `results_real/latest/policy_load_sweep/real_policy_load_summary.md`
- `results_real/latest/policy_load_sweep/real_policy_load_sweep.csv`
- `results_real/latest/policy_load_sweep/*.png`
- `results_real/latest/summary/MANIFEST.md`

See [deployment/README.md](../edge-cache-sim/deployment/README.md) for stack and Mininet details.
