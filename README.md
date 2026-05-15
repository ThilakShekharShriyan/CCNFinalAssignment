# p99-Aware Collaborative Edge Chunk Caching (CCN Final)

**Course:** CS 258 — Computer Communication Networks  
**Repository:** https://github.com/ThilakShekharShriyan/CCNFinalAssignment

Discrete-event simulation of collaborative edge chunk caching with tail-aware (**P99Aware**) replacement, plus an optional localhost deployment prototype.

## Repository layout

| Path | Contents |
|------|----------|
| [docs/RESEARCH_PLAN_SUBMISSION.md](docs/RESEARCH_PLAN_SUBMISSION.md) | Research plan (submission draft) |
| [docs/SCOPE_LOCK.md](docs/SCOPE_LOCK.md) | Frozen baselines, metrics, experiment variables |
| [docs/REFERENCES.md](docs/REFERENCES.md) | Bibliography |
| [docs/PRESENTATION_OUTLINE.md](docs/PRESENTATION_OUTLINE.md) | Presentation structure |
| [edge-cache-sim/](edge-cache-sim/) | Rust simulator and deployment toolkit |
| [artifacts/](artifacts/) | Curated CSVs, plots, and summaries for review |

## Quick start (simulator)

```bash
cd edge-cache-sim
cargo test
cargo run --release -- --config configs/default.toml
```

One-command demo pack (CSVs + plots + summary):

```bash
bash scripts/demo_run.sh
```

Pre-generated demo outputs: [artifacts/simulator-demo/](artifacts/simulator-demo/).

## Reproducibility

- Configurations: `edge-cache-sim/configs/`
- Experiment contract: [docs/SCOPE_LOCK.md](docs/SCOPE_LOCK.md)
- Optional real stack: [edge-cache-sim/deployment/README.md](edge-cache-sim/deployment/README.md)

Build artifacts (`target/`) and scratch run directories (`results/`, `results_real/`) are gitignored; regenerate with the commands above.
