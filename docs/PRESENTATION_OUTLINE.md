# Presentation outline (upload by **April 29, 2026, 8:30 AM**)

Same deck for whichever presentation day you are called. If you have a partner, **both upload identical PPT/PDF** per instructor instructions.

**Deck length (adjust to slot):** aim **10–12 slides** for ~10–15 minutes + backup slides.

---

## Slide 1 — Title

- Title: *p99-Aware Collaborative Chunk Caching for Short-Video Delivery in Wireless Edge Networks*  
- Names, course, category **Simulation (Cat 2)**  
- One line: “Rust discrete-event simulator on macOS”

## Slide 2 — Motivation

- Short-video traffic: bursty, skewed, tail-sensitive QoE  
- Edge caches + **local sharing** between small cells / APs  
- Bullet: industry/MEC trend (no deep standards dive)

## Slide 3 — Problem

- Collaborative caching reduces **mean** delay and backhaul  
- **Gap:** p95/p99 startup delay under bursts often **not** the optimisation target in prior work  
- Research question (one sentence from research plan)

## Slide 4 — Base paper (CLoSER)

- What CLoSER does: SBS cooperation, traces, playout delay  
- Figure: simple **topology** (3–5 edge nodes + origin) — reuse from plan/architecture  
- “We imitate the **model**, not every optimisation detail”

## Slide 5 — Your contribution

- Chunk / **first-chunk** abstraction (startup = first segment latency)  
- **p99-aware** admission/replacement score (high-level formula, no code)  
- Optional: **prediction noise** ablation (one bullet)

## Slide 6 — System architecture

- Mermaid or block diagram: Trace → DES → cluster → policies → metrics  
- Event types: `request_arrival`, `local_hit`, `neighbor_fetch`, `origin_fetch`, `evict`

## Slide 7 — Baselines

- Table: LRU | LFU | CLoSER-like | **Ours (p99-aware)**  
- One column: “what objective each implicitly optimises”

## Slide 8 — Evaluation setup

- Four **sweeps**: cache budget, heterogeneity, burstiness, forecast error ([SCOPE_LOCK.md](SCOPE_LOCK.md))  
- Metrics row: **p99 primary**, mean + hit + remote bytes + Jain fairness

## Slide 9 — Expected results (placeholders OK for Apr 29)

- Sketch **p99 vs cache size** curves (4 policies)  
- Sketch **CDF** of startup delay: baseline vs proposed  
- Caption: “hypothesis—final numbers after experiments”

## Slide 10 — Demo script (live or recorded)

1. Same seed, same trace, two configs side-by-side.  
2. Show rolling **p99** or **CDF** update.  
3. One sentence takeaway.

## Slide 11 — Timeline & milestones

- Gantt or table: now → Apr 29 (first results + deck) → May (sweeps) → May 15 (plan + final deck per feedback)

## Slide 12 — Conclusion

- Expected insight in one sentence  
- Limitations: synthetic workload first; no PHY

---

## Backup slides (hidden after slide 12)

- B1: Related work table (PRIME, DECC, COCAM—one line each)  
- B2: References (top 5)  
- B3: Risk: noisy p99 → many seeds + CI  
- B4: **No overlap** with CS 297/298 statement

---

## Alignment checklist (vs research plan)

- [ ] Problem, gap, contribution, method, metrics, demo all appear  
- [ ] CLoSER cited on slide 4  
- [ ] p99 explicitly on slides 3, 5, 8, 9  
- [ ] File ready for **Apr 29** upload; partner copy identical
