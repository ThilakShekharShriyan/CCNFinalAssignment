#!/usr/bin/env python3
import argparse
import csv
from pathlib import Path


def stats(path: Path):
    rows = list(csv.DictReader(path.open()))
    lats = [float(r["latency_ms"]) for r in rows]
    found = [r["found"] == "true" or r["found"] == "True" for r in rows]
    mean = sum(lats) / len(lats) if lats else 0.0
    hit = sum(found) / len(found) if found else 0.0
    lats_sorted = sorted(lats)

    def pct(q):
        if not lats_sorted:
            return 0.0
        pos = (len(lats_sorted) - 1) * q
        lo = int(pos)
        hi = min(len(lats_sorted) - 1, lo + 1)
        w = pos - lo
        return lats_sorted[lo] * (1 - w) + lats_sorted[hi] * w

    return {"count": len(rows), "mean": mean, "p95": pct(0.95), "p99": pct(0.99), "hit": hit}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--http", required=True)
    ap.add_argument("--grpc", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    http = stats(Path(args.http))
    grpc = stats(Path(args.grpc))

    lines = [
        "# Protocol Comparison Summary",
        "",
        f"- HTTP replay: `{args.http}`",
        f"- gRPC replay: `{args.grpc}`",
        "",
        "| Protocol | Requests | Mean (ms) | P95 (ms) | P99 (ms) | Found rate |",
        "|---|---:|---:|---:|---:|---:|",
        f"| HTTP | {http['count']} | {http['mean']:.3f} | {http['p95']:.3f} | {http['p99']:.3f} | {http['hit']:.4f} |",
        f"| gRPC | {grpc['count']} | {grpc['mean']:.3f} | {grpc['p95']:.3f} | {grpc['p99']:.3f} | {grpc['hit']:.4f} |",
        "",
    ]
    Path(args.out).write_text("\n".join(lines))
    print(f"Wrote {args.out}")


if __name__ == "__main__":
    main()
