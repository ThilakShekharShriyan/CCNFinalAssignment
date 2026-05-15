#!/usr/bin/env python3
import argparse
import csv
import json
from pathlib import Path
from typing import List


def parse_csv_list(v: str) -> List[str]:
    return [x.strip() for x in v.split(",") if x.strip()]


def summarize_rows(rows):
    if not rows:
        return {"count": 0, "mean_ms": 0.0, "p95_ms": 0.0, "p99_ms": 0.0, "hit_rate": 0.0, "remote_ratio": 0.0}
    n = len(rows)
    return {
        "count": n,
        "mean_ms": sum(float(r["mean_ms"]) for r in rows) / n,
        "p95_ms": sum(float(r["p95_ms"]) for r in rows) / n,
        "p99_ms": sum(float(r["p99_ms"]) for r in rows) / n,
        "hit_rate": sum(float(r["hit_rate"]) for r in rows) / n,
        "remote_ratio": sum(float(r["remote_ratio"]) for r in rows) / n,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--csv", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    rows = list(csv.DictReader(Path(args.csv).open()))
    by_policy = {}
    for r in rows:
        by_policy.setdefault(r["policy"], []).append(r)

    lines = [
        "# Real Policy Load Sweep Summary",
        "",
        f"- Source CSV: `{args.csv}`",
        "",
        "| Policy | Rows | Mean (ms) | P95 (ms) | P99 (ms) | Hit rate | Remote ratio |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]
    for p in sorted(by_policy):
        s = summarize_rows(by_policy[p])
        lines.append(
            f"| {p} | {s['count']} | {s['mean_ms']:.3f} | {s['p95_ms']:.3f} | {s['p99_ms']:.3f} | {s['hit_rate']:.4f} | {s['remote_ratio']:.4f} |"
        )

    Path(args.out).write_text("\n".join(lines))
    print(f"Wrote {args.out}")


if __name__ == "__main__":
    main()
