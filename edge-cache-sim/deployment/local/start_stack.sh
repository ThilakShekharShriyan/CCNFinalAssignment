#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN_DIR="${ROOT_DIR}/target/release"
RUN_DIR="${ROOT_DIR}/results_real/local_stack"
mkdir -p "${RUN_DIR}"
START_CONTROLLER="${START_CONTROLLER:-1}"
SKIP_BUILD="${SKIP_BUILD:-0}"

cd "${ROOT_DIR}"
if [[ "${SKIP_BUILD}" != "1" ]]; then
  echo "[stack] building release binaries..."
  cargo build --release --bins --target-dir "${ROOT_DIR}/target" -q
fi

start_proc() {
  local name="$1"
  shift
  echo "[stack] starting ${name}"
  nohup "$@" >"${RUN_DIR}/${name}.log" 2>&1 &
  echo $! >"${RUN_DIR}/${name}.pid"
}

start_proc origin \
  "${BIN_DIR}/origin_server" \
  --http-addr 127.0.0.1:7000 \
  --grpc-addr 127.0.0.1:7100 \
  --objects 1024 \
  --chunks-per-object 4 \
  --chunk-size-kib 64

sleep 1
start_proc edge1 \
  "${BIN_DIR}/edge_server" \
  --node-name edge1 \
  --http-addr 127.0.0.1:7001 \
  --origin-http http://127.0.0.1:7000 \
  --origin-grpc http://127.0.0.1:7100 \
  --neighbors http://127.0.0.1:7002,http://127.0.0.1:7003 \
  --origin-protocol http

start_proc edge2 \
  "${BIN_DIR}/edge_server" \
  --node-name edge2 \
  --http-addr 127.0.0.1:7002 \
  --origin-http http://127.0.0.1:7000 \
  --origin-grpc http://127.0.0.1:7100 \
  --neighbors http://127.0.0.1:7001,http://127.0.0.1:7003 \
  --origin-protocol http

start_proc edge3 \
  "${BIN_DIR}/edge_server" \
  --node-name edge3 \
  --http-addr 127.0.0.1:7003 \
  --origin-http http://127.0.0.1:7000 \
  --origin-grpc http://127.0.0.1:7100 \
  --neighbors http://127.0.0.1:7001,http://127.0.0.1:7002 \
  --origin-protocol http

if [[ "${START_CONTROLLER}" == "1" ]]; then
  start_proc controller \
    "${BIN_DIR}/controller_service" \
    --edges http://127.0.0.1:7001,http://127.0.0.1:7002,http://127.0.0.1:7003 \
    --interval-seconds 5
fi

echo "[stack] started. logs in ${RUN_DIR}"
echo "[stack] run replay:"
echo "  ${BIN_DIR}/workload_replay --edges http://127.0.0.1:7001,http://127.0.0.1:7002,http://127.0.0.1:7003 --requests 5000 --csv-out ${ROOT_DIR}/results_real/replay.csv"
