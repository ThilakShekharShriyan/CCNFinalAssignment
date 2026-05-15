#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN_DIR="${ROOT_DIR}/target/release"
OUT_DIR="${ROOT_DIR}/results_real/protocol_compare"
mkdir -p "${OUT_DIR}"

cleanup() {
  for f in "${OUT_DIR}"/*.pid; do
    [[ -e "$f" ]] || continue
    pid="$(cat "$f")"
    kill "$pid" 2>/dev/null || true
    rm -f "$f"
  done
}
trap cleanup EXIT

start_bg() {
  local name="$1"
  shift
  nohup "$@" >"${OUT_DIR}/${name}.log" 2>&1 &
  echo $! >"${OUT_DIR}/${name}.pid"
}

run_case() {
  local proto="$1"
  local out_csv="${OUT_DIR}/replay_${proto}.csv"
  echo "[protocol] running ${proto}"
  cleanup

  start_bg origin \
    "${BIN_DIR}/origin_server" \
    --http-addr 127.0.0.1:7000 \
    --grpc-addr 127.0.0.1:7100 \
    --objects 1024 \
    --chunks-per-object 4 \
    --chunk-size-kib 64
  sleep 1
  start_bg edge1 \
    "${BIN_DIR}/edge_server" \
    --node-name edge1 \
    --http-addr 127.0.0.1:7001 \
    --origin-http http://127.0.0.1:7000 \
    --origin-grpc http://127.0.0.1:7100 \
    --neighbors "" \
    --origin-protocol "${proto}"
  sleep 1

  "${BIN_DIR}/workload_replay" \
    --edges http://127.0.0.1:7001 \
    --requests 4000 \
    --seed 42 \
    --csv-out "${out_csv}"
}

cd "${ROOT_DIR}"
cargo build --release --bins --target-dir "${ROOT_DIR}/target" -q
run_case http
run_case grpc
cleanup

python3 "${ROOT_DIR}/deployment/experiments/protocol_compare_summary.py" \
  --http "${OUT_DIR}/replay_http.csv" \
  --grpc "${OUT_DIR}/replay_grpc.csv" \
  --out "${OUT_DIR}/protocol_compare_summary.md"

echo "[protocol] done. outputs in ${OUT_DIR}"
