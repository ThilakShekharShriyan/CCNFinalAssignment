# Mininet Lab

This folder contains scripts to run the real edge-cache prototype inside Mininet.

## Prereqs

- Linux environment with Mininet installed.
- Build Rust binaries first:

```bash
cargo build --release
```

## Launch

```bash
sudo python3 deployment/mininet/topology.py --binary-dir "$(pwd)/target/release"
```

Optional one-shot replay:

```bash
sudo python3 deployment/mininet/topology.py \
  --binary-dir "$(pwd)/target/release" \
  --run-replay --replay-requests 3000
```

## Useful checks in Mininet CLI

```bash
edge1 curl -s http://10.0.0.11:7001/health
edge1 curl -s http://10.0.0.11:7001/metrics
edge2 curl -s http://10.0.0.12:7002/policy/config
origin curl -s http://10.0.0.20:7000/chunk/10/0
```

Logs are written per host under `/tmp/<host>.log` within the Mininet runtime environment.
