#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUN_DIR="${ROOT_DIR}/results_real/local_stack"

if [[ ! -d "${RUN_DIR}" ]]; then
  echo "[stack] no local stack run directory at ${RUN_DIR}"
  exit 0
fi

for pidfile in "${RUN_DIR}"/*.pid; do
  [[ -e "${pidfile}" ]] || continue
  pid="$(cat "${pidfile}")"
  if kill -0 "${pid}" 2>/dev/null; then
    echo "[stack] stopping pid ${pid}"
    kill "${pid}" || true
  fi
  rm -f "${pidfile}"
done

echo "[stack] stopped."
