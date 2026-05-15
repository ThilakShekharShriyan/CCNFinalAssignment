# Rubric crosswalk (self-check)

Your course may publish a **detailed rubric per submission** on Canvas. This file is a **generic systems/networking research-plan checklist** so you can paste matching sentences into the template or appendix. When the official rubric is released, **add a column** “Official rubric item → evidence in our submission” and tick every box.

## Problem and motivation

- [ ] States a **clear networking problem** (wireless edge content delivery, cooperative caching, latency/QoE).  
- [ ] Explains **why it matters** (short-video traffic, edge/MEC, backhaul).  
- [ ] Avoids overlap with **CS 297 / CS 298** (one explicit sentence).

## Related work and base paper

- [ ] Names a **primary anchor paper** (CLoSER) with full citation.  
- [ ] States what will be **imitated** (system model: multi-edge, local sharing, origin fallback) vs what will **not** be replicated (full solver complexity).  
- [ ] Lists **2–4 supporting** references (short-video, predictive MEC, optional cooperative multicast).

## Research gap and novelty

- [ ] Gap is **one paragraph**, not a vague “more research needed.”  
- [ ] Novelty is **measurable**: chunk-level + **p95/p99 startup** + optional **forecast error**—not “we use AI.”

## Methodology

- [ ] **Simulation** approach is explicit (discrete-event, laptop-feasible).  
- [ ] **Components** named: workload, topology, caches, policies, metrics export.  
- [ ] **Baselines** listed and match [SCOPE_LOCK.md](SCOPE_LOCK.md).

## Evaluation

- [ ] **Primary metrics** are p95/p99 startup delay.  
- [ ] **Independent variables** are listed (4 sweeps locked in scope doc).  
- [ ] **Hypotheses** stated in falsifiable form (optional but strong for grading).  
- [ ] **Reproducibility**: seeds, configs, command to rerun.

## Feasibility and ethics

- [ ] Timeline reaches **May 15, 2026** research plan and **Apr 29, 2026** presentation upload.  
- [ ] Scope is **bounded** (no full 5G stack, no GPU training requirement).  
- [ ] Data: **synthetic or public aggregated** traces; no personal data.

## Deliverables alignment

- [ ] **Demo story** referenced (live run or pre-recorded CDF / p99 vs cache size).  
- [ ] **Team**: if two members, note **identical file uploads** where required.

## Presentation-specific (Apr 29 deck)

- [ ] One slide: **architecture diagram**.  
- [ ] One slide: **expected results** (even placeholder curves).  
- [ ] One slide: **limitations** (honest).

---

**How to use:** When Canvas rubric rows appear, duplicate each row here and fill “Where in RESEARCH_PLAN_SUBMISSION.md or slides” in the last column.
