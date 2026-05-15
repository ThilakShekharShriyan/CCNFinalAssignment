# Real Deployment Toolkit

This folder contains scripts and specs for the real-network prototype.

## Contents

- [API_CONTRACT.md](API_CONTRACT.md): REST/gRPC contract and metric schema.
- `local/`:
  - `start_stack.sh`: start origin + 3 edges + controller on localhost.
  - `stop_stack.sh`: stop all local stack processes.
- `mininet/`:
  - `topology.py`: Mininet topology + service launcher.
  - `README.md`: Mininet setup and usage.
- `experiments/`:
  - `protocol_compare.sh`: run HTTP vs gRPC comparison.
  - `run_validation_sweeps.sh`: run simulator + real sweeps and produce sim-vs-real summary.

## Quick local run

```bash
bash deployment/local/start_stack.sh
target/release/workload_replay --edges http://127.0.0.1:7001,http://127.0.0.1:7002,http://127.0.0.1:7003 --requests 2000 --csv-out results_real/replay.csv
curl -s http://127.0.0.1:7001/metrics
bash deployment/local/stop_stack.sh
```

## Real-data-only research pipeline

Use this for final research artifacts (excludes simulator outputs from final evidence pack):

```bash
bash deployment/experiments/real_only_runner.sh
```

Outputs:

- `results_real/latest/protocol_compare/*`
- `results_real/latest/policy_load_sweep/*`
- `results_real/latest/summary/*`
- archive copy: `results_real/archive/<timestamp>/*`
