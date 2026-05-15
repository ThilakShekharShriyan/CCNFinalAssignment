#!/usr/bin/env python3
import argparse
import csv
from collections import defaultdict
from pathlib import Path


def read_csv(path):
    with open(path, newline="") as f:
        return list(csv.DictReader(f))


def monotonic_nonincreasing(values):
    return all(values[i + 1] <= values[i] + 1e-9 for i in range(len(values) - 1))


def collect_sim(rows):
    by = defaultdict(list)
    for r in rows:
        by[r["policy"]].append((float(r["request_unit_interval"]), float(r["mean_ms"])))
    out = {}
    for p, vals in by.items():
        vals = sorted(vals)
        out[p] = {"x": [x for x, _ in vals], "y": [y for _, y in vals], "mono": monotonic_nonincreasing([y for _, y in vals])}
    return out


def collect_real(rows):
    by = defaultdict(list)
    for r in rows:
        by[r["policy"]].append((float(r["request_interval_ms"]), float(r["mean_ms"])))
    out = {}
    for p, vals in by.items():
        vals = sorted(vals)
        out[p] = {"x": [x for x, _ in vals], "y": [y for _, y in vals], "mono": monotonic_nonincreasing([y for _, y in vals])}
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sim", required=True)
    ap.add_argument("--real", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    sim = collect_sim(read_csv(args.sim))
    real = collect_real(read_csv(args.real))

    lines = [
        "# Simulator vs Real Validation Summary",
        "",
        f"- Simulator CSV: `{args.sim}`",
        f"- Real CSV: `{args.real}`",
        "",
        "| Policy | Sim mean monotonic (higher interval -> lower mean) | Real mean monotonic (higher interval -> lower mean) | Trend aligned |",
        "|---|---|---|---|",
    ]
    for p in sorted(set(sim.keys()) | set(real.keys())):
        sm = sim.get(p, {}).get("mono", False)
        rm = real.get(p, {}).get("mono", False)
        lines.append(f"| {p} | {sm} | {rm} | {sm and rm} |")

    lines += [
        "",
        "## Notes",
        "- Real runs use replay pacing (`request_interval_ms`) as the offered-load control.",
        "- Simulator runs use `request_unit_interval`.",
        "- Validation focuses on directional consistency, not exact latency matching.",
    ]
    Path(args.out).write_text("\n".join(lines))
    print(f"Wrote {args.out}")


if __name__ == "__main__":
    main()
