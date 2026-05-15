#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${ROOT_DIR}/results_real/validation"
mkdir -p "${OUT_DIR}"

cd "${ROOT_DIR}"
echo "[validate] building binaries..."
cargo build --release --bins --target-dir "${ROOT_DIR}/target" -q

echo "[validate] simulator unit-interval sweep..."
cargo run --release --quiet --bin edge-cache-sim -- \
  --config configs/phase3_base.toml \
  --sweep-policies \
  --sweep-unit-intervals 60,120,180 \
  --csv-out "${OUT_DIR}/sim_unit_sweep.csv"

echo "policy,request_interval_ms,mean_ms,p95_ms,p99_ms,hit_rate,remote_ratio,node_count" > "${OUT_DIR}/real_unit_sweep.csv"

for policy in lru lfu closer p99aware; do
for unit in 60 120 180; do
  echo "[validate] real run policy=${policy} unit=${unit}"
  START_CONTROLLER=0 SKIP_BUILD=1 "${ROOT_DIR}/deployment/local/start_stack.sh" >/dev/null
  sleep 2

  # Map sim unit interval to replay pacing (ms between requests).
  if [[ "${unit}" == "60" ]]; then
    replay_ms="1.5"
  elif [[ "${unit}" == "120" ]]; then
    replay_ms="3.0"
  else
    replay_ms="4.5"
  fi

  curl -s -X POST http://127.0.0.1:7001/policy/config -H 'content-type: application/json' -d "{\"policy\":\"${policy}\",\"p99_w_hit\":1.0,\"p99_w_tail\":0.08,\"p99_w_remote\":0.02}" >/dev/null
  curl -s -X POST http://127.0.0.1:7002/policy/config -H 'content-type: application/json' -d "{\"policy\":\"${policy}\",\"p99_w_hit\":1.0,\"p99_w_tail\":0.08,\"p99_w_remote\":0.02}" >/dev/null
  curl -s -X POST http://127.0.0.1:7003/policy/config -H 'content-type: application/json' -d "{\"policy\":\"${policy}\",\"p99_w_hit\":1.0,\"p99_w_tail\":0.08,\"p99_w_remote\":0.02}" >/dev/null

  "${ROOT_DIR}/target/release/workload_replay" \
    --edges http://127.0.0.1:7001,http://127.0.0.1:7002,http://127.0.0.1:7003 \
    --requests 900 \
    --request-interval-ms "${replay_ms}" \
    --seed 42 \
    --csv-out "${OUT_DIR}/replay_${policy}_unit_${unit}.csv" >/dev/null

  curl -s http://127.0.0.1:7001/metrics > "${OUT_DIR}/m1_${unit}.json"
  curl -s http://127.0.0.1:7002/metrics > "${OUT_DIR}/m2_${unit}.json"
  curl -s http://127.0.0.1:7003/metrics > "${OUT_DIR}/m3_${unit}.json"

  python3 - <<PY
import json, pathlib
unit = "${unit}"
policy = "${policy}"
replay_ms = "${replay_ms}"
paths = [pathlib.Path("${OUT_DIR}/m1_${unit}.json"), pathlib.Path("${OUT_DIR}/m2_${unit}.json"), pathlib.Path("${OUT_DIR}/m3_${unit}.json")]
rows = [json.loads(p.read_text()) for p in paths]
mean_ms = sum(r["counters"]["total_latency_ms"]/max(1,r["counters"]["total_requests"]) for r in rows)/len(rows)
p95_ms = sum(r["p95_ms"] for r in rows)/len(rows)
p99_ms = sum(r["p99_ms"] for r in rows)/len(rows)
hit_rate = sum(r["hit_rate"] for r in rows)/len(rows)
remote_ratio = sum(r["remote_ratio"] for r in rows)/len(rows)
out = pathlib.Path("${OUT_DIR}/real_unit_sweep.csv")
with out.open("a") as f:
    f.write(f"{policy},{replay_ms},{mean_ms:.6f},{p95_ms:.6f},{p99_ms:.6f},{hit_rate:.6f},{remote_ratio:.6f},{len(rows)}\\n")
PY

  "${ROOT_DIR}/deployment/local/stop_stack.sh" >/dev/null
done
done

python3 "${ROOT_DIR}/deployment/experiments/sim_real_compare.py" \
  --sim "${OUT_DIR}/sim_unit_sweep.csv" \
  --real "${OUT_DIR}/real_unit_sweep.csv" \
  --out "${OUT_DIR}/sim_real_validation.md"

echo "[validate] done. outputs in ${OUT_DIR}"
