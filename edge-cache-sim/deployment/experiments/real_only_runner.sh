#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LATEST_DIR="${ROOT_DIR}/results_real/latest"
ARCHIVE_ROOT="${ROOT_DIR}/results_real/archive"
TS="$(date +%Y%m%d_%H%M%S)"
ARCHIVE_DIR="${ARCHIVE_ROOT}/${TS}"

PROTOCOL_DIR="${LATEST_DIR}/protocol_compare"
SWEEP_DIR="${LATEST_DIR}/policy_load_sweep"
SUMMARY_DIR="${LATEST_DIR}/summary"

mkdir -p "${PROTOCOL_DIR}" "${SWEEP_DIR}" "${SUMMARY_DIR}" "${ARCHIVE_DIR}"

cd "${ROOT_DIR}"
echo "[real-only] building binaries..."
cargo build --release --bins --target-dir "${ROOT_DIR}/target" -q

echo "[real-only] protocol comparison (http vs grpc)..."
bash "${ROOT_DIR}/deployment/experiments/protocol_compare.sh"
cp -f "${ROOT_DIR}/results_real/protocol_compare/replay_http.csv" "${PROTOCOL_DIR}/replay_http.csv"
cp -f "${ROOT_DIR}/results_real/protocol_compare/replay_grpc.csv" "${PROTOCOL_DIR}/replay_grpc.csv"
cp -f "${ROOT_DIR}/results_real/protocol_compare/protocol_compare_summary.md" "${PROTOCOL_DIR}/protocol_compare_summary.md"

echo "[real-only] policy/load sweep..."
REAL_CSV="${SWEEP_DIR}/real_policy_load_sweep.csv"
echo "policy,request_interval_ms,mean_ms,p95_ms,p99_ms,hit_rate,remote_ratio,node_count" > "${REAL_CSV}"

for policy in lru lfu closer p99aware; do
  for replay_ms in 1.5 3.0 4.5; do
    echo "[real-only] run policy=${policy} interval_ms=${replay_ms}"
    START_CONTROLLER=0 SKIP_BUILD=1 "${ROOT_DIR}/deployment/local/start_stack.sh" >/dev/null
    sleep 2

    curl -s -X POST http://127.0.0.1:7001/policy/config -H 'content-type: application/json' -d "{\"policy\":\"${policy}\",\"p99_w_hit\":1.0,\"p99_w_tail\":0.08,\"p99_w_remote\":0.02}" >/dev/null
    curl -s -X POST http://127.0.0.1:7002/policy/config -H 'content-type: application/json' -d "{\"policy\":\"${policy}\",\"p99_w_hit\":1.0,\"p99_w_tail\":0.08,\"p99_w_remote\":0.02}" >/dev/null
    curl -s -X POST http://127.0.0.1:7003/policy/config -H 'content-type: application/json' -d "{\"policy\":\"${policy}\",\"p99_w_hit\":1.0,\"p99_w_tail\":0.08,\"p99_w_remote\":0.02}" >/dev/null

    "${ROOT_DIR}/target/release/workload_replay" \
      --edges http://127.0.0.1:7001,http://127.0.0.1:7002,http://127.0.0.1:7003 \
      --requests 900 \
      --request-interval-ms "${replay_ms}" \
      --seed 42 \
      --csv-out "${SWEEP_DIR}/replay_${policy}_${replay_ms}.csv" >/dev/null

    curl -s http://127.0.0.1:7001/metrics > "${SWEEP_DIR}/m1_${policy}_${replay_ms}.json"
    curl -s http://127.0.0.1:7002/metrics > "${SWEEP_DIR}/m2_${policy}_${replay_ms}.json"
    curl -s http://127.0.0.1:7003/metrics > "${SWEEP_DIR}/m3_${policy}_${replay_ms}.json"

    python3 - <<PY
import json, pathlib
policy="${policy}"
replay_ms="${replay_ms}"
paths=[
  pathlib.Path("${SWEEP_DIR}/m1_${policy}_${replay_ms}.json"),
  pathlib.Path("${SWEEP_DIR}/m2_${policy}_${replay_ms}.json"),
  pathlib.Path("${SWEEP_DIR}/m3_${policy}_${replay_ms}.json"),
]
rows=[json.loads(p.read_text()) for p in paths]
mean_ms=sum(r["counters"]["total_latency_ms"]/max(1,r["counters"]["total_requests"]) for r in rows)/len(rows)
p95_ms=sum(r["p95_ms"] for r in rows)/len(rows)
p99_ms=sum(r["p99_ms"] for r in rows)/len(rows)
hit_rate=sum(r["hit_rate"] for r in rows)/len(rows)
remote_ratio=sum(r["remote_ratio"] for r in rows)/len(rows)
out=pathlib.Path("${REAL_CSV}")
with out.open("a") as f:
    f.write(f"{policy},{replay_ms},{mean_ms:.6f},{p95_ms:.6f},{p99_ms:.6f},{hit_rate:.6f},{remote_ratio:.6f},{len(rows)}\\n")
PY

    "${ROOT_DIR}/deployment/local/stop_stack.sh" >/dev/null
  done
done

python3 "${ROOT_DIR}/deployment/experiments/real_policy_load_sweep.py" \
  --csv "${REAL_CSV}" \
  --out "${SWEEP_DIR}/real_policy_load_summary.md"

python3 "${ROOT_DIR}/scripts/plot_sweep.py" "${REAL_CSV}" --x request_interval_ms --metric mean_ms --out "${SWEEP_DIR}/mean_vs_request_interval.png"
python3 "${ROOT_DIR}/scripts/plot_sweep.py" "${REAL_CSV}" --x request_interval_ms --metric p95_ms --out "${SWEEP_DIR}/p95_vs_request_interval.png"
python3 "${ROOT_DIR}/scripts/plot_sweep.py" "${REAL_CSV}" --x request_interval_ms --metric hit_rate --out "${SWEEP_DIR}/hit_vs_request_interval.png"

echo "[real-only] writing manifest and run notes..."
GIT_SHA="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
cat > "${SUMMARY_DIR}/MANIFEST.md" <<EOF
# Real Results Manifest

- Timestamp: ${TS}
- Git commit: ${GIT_SHA}
- Runner: deployment/experiments/real_only_runner.sh
- Protocol compare script: deployment/experiments/protocol_compare.sh
- Policy/load summary script: deployment/experiments/real_policy_load_sweep.py
- Replay requests per run: 900
- Replay seed: 42
- Replay intervals (ms): 1.5, 3.0, 4.5
- Policies: lru, lfu, closer, p99aware
EOF

cat > "${SUMMARY_DIR}/RUN_NOTES.md" <<EOF
# Run Notes (Real Data Only)

This pack intentionally excludes simulator CSVs from final evidence.

Primary evidence:
- protocol_compare/*
- policy_load_sweep/*

Quality checks:
- protocol replay CSVs generated
- policy/load sweep CSV has all policy x interval rows
- plots generated and non-empty
EOF

echo "[real-only] archiving latest -> ${ARCHIVE_DIR}"
mkdir -p "${ARCHIVE_DIR}"
cp -R "${LATEST_DIR}/." "${ARCHIVE_DIR}/"

echo "[real-only] done"
echo "latest:  ${LATEST_DIR}"
echo "archive: ${ARCHIVE_DIR}"
