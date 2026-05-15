#!/usr/bin/env bash
set -euo pipefail

# One-command demo pack generator:
# - runs key sweeps
# - generates plots
# - writes a markdown summary
#
# Usage:
#   bash scripts/demo_run.sh
#   bash scripts/demo_run.sh "results/demo_pack_custom"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

STAMP="$(date +%Y%m%d_%H%M%S)"
OUT_DIR="${1:-results/demo_pack_${STAMP}}"
mkdir -p "$OUT_DIR"

echo "[demo] building release binary..."
cargo build --release -q --bin edge-cache-sim

echo "[demo] running cache sweep (policy x cache size)..."
cargo run --release --quiet --bin edge-cache-sim -- \
  --config configs/phase3_base.toml \
  --sweep-policies \
  --sweep-cache-sizes 36,54,72,90 \
  --csv-out "${OUT_DIR}/cache_sweep.csv"

echo "[demo] running burst sweep (policy x burst high multiplier)..."
cargo run --release --quiet --bin edge-cache-sim -- \
  --config configs/phase3_base.toml \
  --sweep-policies \
  --sweep-burst-high-mults 1,2,4,6 \
  --csv-out "${OUT_DIR}/burst_sweep.csv"

echo "[demo] running hetero sweep (policy x homogeneous/heterogeneous)..."
cargo run --release --quiet --bin edge-cache-sim -- \
  --config configs/phase3_base.toml \
  --sweep-policies \
  --sweep-heterogeneous \
  --csv-out "${OUT_DIR}/hetero_sweep.csv"

echo "[demo] running unit-interval calibration sweep (policy x load level)..."
cargo run --release --quiet --bin edge-cache-sim -- \
  --config configs/phase3_base.toml \
  --sweep-policies \
  --sweep-unit-intervals 60,120,180 \
  --csv-out "${OUT_DIR}/unit_interval_sweep.csv"

if python3 -c "import pandas, matplotlib" >/dev/null 2>&1; then
  echo "[demo] plotting..."
  python3 scripts/plot_sweep.py "${OUT_DIR}/cache_sweep.csv" --x cache_capacity_chunks --metric mean_ms --het any --out "${OUT_DIR}/cache_mean.png"
  python3 scripts/plot_sweep.py "${OUT_DIR}/cache_sweep.csv" --x cache_capacity_chunks --metric hit_rate --het any --out "${OUT_DIR}/cache_hit.png"
  python3 scripts/plot_sweep.py "${OUT_DIR}/burst_sweep.csv" --x burst_high_rate_mult --metric mean_ms --het any --out "${OUT_DIR}/burst_mean.png"
  python3 scripts/plot_sweep.py "${OUT_DIR}/unit_interval_sweep.csv" --x request_unit_interval --metric mean_ms --het any --out "${OUT_DIR}/unit_interval_mean.png"
else
  echo "[demo] pandas/matplotlib not found; skipping plot generation."
  echo "       install with: pip install -r scripts/requirements-plot.txt"
fi

echo "[demo] writing summary markdown..."
python3 scripts/demo_summary.py \
  --cache "${OUT_DIR}/cache_sweep.csv" \
  --burst "${OUT_DIR}/burst_sweep.csv" \
  --hetero "${OUT_DIR}/hetero_sweep.csv" \
  --unit "${OUT_DIR}/unit_interval_sweep.csv" \
  --out "${OUT_DIR}/DEMO_SUMMARY.md"

cat <<EOF
[demo] done.
[demo] output folder: ${OUT_DIR}
[demo] open these first:
  - ${OUT_DIR}/DEMO_SUMMARY.md
  - ${OUT_DIR}/cache_mean.png
  - ${OUT_DIR}/cache_hit.png
  - ${OUT_DIR}/burst_mean.png
  - ${OUT_DIR}/unit_interval_mean.png
EOF
