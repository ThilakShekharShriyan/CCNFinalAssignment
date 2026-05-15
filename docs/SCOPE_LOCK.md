# Scope lock (frozen for the semester)

This document freezes **baselines**, **metrics**, and **independent variables** so the research plan, simulator, slides, and final report stay aligned. Changes require an explicit note in the project log (one sentence: what changed and why).

## Project category

- **Category 2 — Simulation project** (imitate CLoSER-style collaborative edge caching; **new** tail-aware chunk policy + evaluation).

## Baselines (implement all four before tuning the proposed policy)

1. **LRU** — per-edge chunk cache, standard least-recently-used eviction.  
2. **LFU / static popularity** — eviction by lowest historical frequency (Zipf-weighted synthetic demand is acceptable).  
3. **CLoSER-like collaborative** — multi-edge cluster; on miss, fetch from **nearest neighbour** that holds the chunk, else **origin**; placement/replacement optimises **mean** delay or hit rate **without** a p99 term (greedy or periodic refresh is enough—does not need to reproduce CLoSER’s full NP-hard solver).  
4. **Proposed — p99-aware collaborative chunk** — same topology and admission paths as (3), but replacement/admission uses the **tail-aware score** from the research plan (rolling window, weighted hit / p99 / remote bytes).

## Primary and secondary metrics

| Priority | Metric | Definition (simulator) |
|----------|--------|-------------------------|
| **Primary** | **p99 startup delay** | 99th percentile of per-request time until **first playback chunk** is available at the serving edge (includes local hit, neighbour transfer, or origin). |
| **Primary** | **p95 startup delay** | Same, 95th percentile. |
| Secondary | Mean startup delay | Average of the same per-request latency. |
| Secondary | Cache hit ratio | Fraction of requests served with **first chunk** already at the attached edge (local hit). |
| Secondary | Remote bytes | Total bytes pulled from **origin** (not neighbour). |
| Secondary | Fairness | **Jain’s fairness index** on per-user **mean** startup delay (or per-user p95 if sample size allows). |

## Independent variables (exactly four sweeps)

1. **Cache budget** — total chunks storable per edge, or uniform `C` per node; sweep e.g. `{small, medium, large}` as fractions of catalogue size.  
2. **Backhaul heterogeneity** — fraction of edges or edges with **slow** edge-to-origin or inter-edge links (latency × factor, bandwidth ÷ factor).  
3. **Burstiness** — request rate multiplier in **on** state vs **off** state, or burst probability per time slot (document the generator parameters in the repo `configs/`).  
4. **Prediction error** — noise on predicted popularity (Gaussian or uniform perturbation) and/or a **single step change** in hot set mid-simulation.

## Out of scope (do not implement unless baselines finish early)

- Full PHY or ns-3 integration.  
- Training **full DRL** (PRIME/DECC-scale neural nets).  
- GPU clusters, NCCL, real distributed training.  
- Overlapping artefact with **CS 297 / CS 298**.

## Reproducibility

- Fixed RNG seeds in config.  
- One command line entry point documented in the simulator crate (when implemented).  
- All figures traceable to a saved CSV + config hash.
