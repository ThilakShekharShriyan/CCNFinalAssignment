#!/usr/bin/env python3
"""
Plot p99 (or mean) vs cache_capacity_chunks from a sweep CSV produced by edge-cache-sim.

  pip install pandas matplotlib
  python3 scripts/plot_sweep.py results/phase3.csv --metric p99_ms --out results/p99_vs_cache.png

Requires CSV with columns: policy, cache_capacity_chunks, mean_ms, p95_ms, p99_ms, ...
"""

from __future__ import annotations

import argparse
import sys


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("csv", help="CSV from --csv-out")
    p.add_argument(
        "--metric",
        default="p99_ms",
        choices=("mean_ms", "p95_ms", "p99_ms", "hit_rate", "remote_mb", "jain_fairness"),
    )
    p.add_argument("--out", default="plot.png", help="Output PNG path")
    p.add_argument(
        "--x",
        default="cache_capacity_chunks",
        help="X-axis column (e.g. cache_capacity_chunks, prediction_noise_sigma)",
    )
    p.add_argument(
        "--het",
        choices=("any", "true", "false"),
        default="any",
        help="Filter rows by heterogeneous_backhaul column",
    )
    args = p.parse_args()

    try:
        import pandas as pd
        import matplotlib.pyplot as plt
    except ImportError:
        print("Install dependencies: pip install pandas matplotlib", file=sys.stderr)
        sys.exit(1)

    df = pd.read_csv(args.csv)
    if args.het != "any":
        want = args.het == "true"
        df = df[df["heterogeneous_backhaul"].astype(bool) == want]
    if df.empty:
        print("No rows after filter.", file=sys.stderr)
        sys.exit(2)

    if args.x not in df.columns:
        print(f"Missing column {args.x!r}. Have: {list(df.columns)}", file=sys.stderr)
        sys.exit(3)

    plt.figure(figsize=(8, 5))
    for pol in sorted(df["policy"].unique()):
        sub = df[df["policy"] == pol].sort_values(args.x)
        plt.plot(
            sub[args.x],
            sub[args.metric],
            marker="o",
            label=pol,
        )
    plt.xlabel(args.x)
    plt.ylabel(args.metric)
    plt.title(f"{args.metric} vs {args.x}")
    plt.legend()
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    plt.savefig(args.out, dpi=150)
    print(f"Wrote {args.out}")


if __name__ == "__main__":
    main()
