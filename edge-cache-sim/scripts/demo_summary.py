#!/usr/bin/env python3
"""
Create a concise markdown summary from demo sweep CSV files.
No third-party dependencies required.
"""

from __future__ import annotations

import argparse
import csv
from pathlib import Path


def read_rows(path: Path):
    with path.open(newline="") as f:
        return list(csv.DictReader(f))


def to_float(v: str) -> float:
    return float(v)


def best_by(rows, metric: str, minimize: bool = True):
    key = (lambda r: to_float(r[metric]))
    return min(rows, key=key) if minimize else max(rows, key=key)


def section_cache(rows):
    best_mean = best_by(rows, "mean_ms", minimize=True)
    best_hit = best_by(rows, "hit_rate", minimize=False)
    return [
        "## Cache Sweep (policy x cache size)",
        f"- Best mean latency: `{best_mean['policy']}` at cache `{best_mean['cache_capacity_chunks']}` with mean `{float(best_mean['mean_ms']):.3f} ms`.",
        f"- Best hit rate: `{best_hit['policy']}` at cache `{best_hit['cache_capacity_chunks']}` with hit `{float(best_hit['hit_rate']):.4f}`.",
    ]


def section_burst(rows):
    by_policy = {}
    for r in rows:
        by_policy.setdefault(r["policy"], []).append(r)
    lines = ["## Burst Sweep (policy x burst_high_rate_mult)"]
    for pol, rs in sorted(by_policy.items()):
        rs = sorted(rs, key=lambda r: float(r["burst_high_rate_mult"]))
        lo = float(rs[0]["mean_ms"])
        hi = float(rs[-1]["mean_ms"])
        lines.append(
            f"- `{pol}` mean latency from burst `{rs[0]['burst_high_rate_mult']}` to `{rs[-1]['burst_high_rate_mult']}`: `{lo:.3f}` -> `{hi:.3f}` ms."
        )
    return lines


def section_hetero(rows):
    by = {}
    for r in rows:
        by[(r["policy"], str(r["heterogeneous_backhaul"]).strip().lower())] = float(r["mean_ms"])
    lines = ["## Heterogeneous Backhaul Impact"]
    policies = sorted({r["policy"] for r in rows})
    for p in policies:
        h0 = by.get((p, "false"))
        h1 = by.get((p, "true"))
        if h0 is None or h1 is None:
            continue
        delta = h1 - h0
        lines.append(f"- `{p}` mean latency delta (hetero - homo): `{delta:.3f}` ms.")
    return lines


def section_unit(rows):
    by_policy = {}
    for r in rows:
        by_policy.setdefault(r["policy"], []).append(r)
    lines = ["## Unit-Interval Calibration (offered load)"]
    for pol, rs in sorted(by_policy.items()):
        rs = sorted(rs, key=lambda r: float(r["request_unit_interval"]))
        lo = float(rs[0]["mean_ms"])
        hi = float(rs[-1]["mean_ms"])
        lines.append(
            f"- `{pol}` mean latency from interval `{rs[0]['request_unit_interval']}` to `{rs[-1]['request_unit_interval']}`: `{lo:.3f}` -> `{hi:.3f}` ms."
        )
    return lines


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cache", required=True)
    ap.add_argument("--burst", required=True)
    ap.add_argument("--hetero", required=True)
    ap.add_argument("--unit", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    cache_rows = read_rows(Path(args.cache))
    burst_rows = read_rows(Path(args.burst))
    hetero_rows = read_rows(Path(args.hetero))
    unit_rows = read_rows(Path(args.unit))

    lines = [
        "# Demo Summary",
        "",
        "This file is auto-generated from demo sweeps.",
        "",
        f"- Cache CSV: `{args.cache}`",
        f"- Burst CSV: `{args.burst}`",
        f"- Hetero CSV: `{args.hetero}`",
        f"- Unit-interval CSV: `{args.unit}`",
        "",
    ]
    lines += section_cache(cache_rows) + [""]
    lines += section_burst(burst_rows) + [""]
    lines += section_hetero(hetero_rows) + [""]
    lines += section_unit(unit_rows) + [""]
    lines += [
        "## Suggested live demo flow",
        "- Start with cache sweep plots (`cache_mean.png`, `cache_hit.png`) to establish baseline trade-off.",
        "- Show burst sweep (`burst_mean.png`) to discuss load sensitivity.",
        "- Show unit-interval calibration (`unit_interval_mean.png`) to explain queue pressure control.",
    ]

    out = Path(args.out)
    out.write_text("\n".join(lines))
    print(f"Wrote {out}")


if __name__ == "__main__":
    main()
