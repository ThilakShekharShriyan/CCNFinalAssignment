# CCN final — research plan + simulator

## Simulator (Rust)

| Path | Purpose |
|------|---------|
| [edge-cache-sim/](edge-cache-sim/) | `cargo run` discrete-event style edge caching study |
| [edge-cache-sim/configs/default.toml](edge-cache-sim/configs/default.toml) | Baseline parameters |
| [edge-cache-sim/README.md](edge-cache-sim/README.md) | Run / policy sweep / Phase-3 cache & het sweeps / optional plots |

```bash
cd edge-cache-sim && cargo run --release -- --config configs/default.toml
```

## Research plan docs

| File | Purpose |
|------|---------|
| [docs/RESEARCH_PLAN_SUBMISSION.md](docs/RESEARCH_PLAN_SUBMISSION.md) | Paste-ready research plan body |
| [docs/CANVAS_TEMPLATE_MAPPING.md](docs/CANVAS_TEMPLATE_MAPPING.md) | Map Canvas template headings to sections |
| [docs/SCOPE_LOCK.md](docs/SCOPE_LOCK.md) | Frozen baselines, metrics, experiment variables |
| [docs/REFERENCES.md](docs/REFERENCES.md) | Verified bibliography + PDF links |
| [docs/RUBRIC_CROSSWALK.md](docs/RUBRIC_CROSSWALK.md) | Self-check vs typical rubric criteria |
| [docs/PRESENTATION_OUTLINE.md](docs/PRESENTATION_OUTLINE.md) | Apr 29 deck structure |

Do not edit the Cursor plan file at `~/.cursor/plans/` unless you intend to; graded artefacts live here and on Canvas.
