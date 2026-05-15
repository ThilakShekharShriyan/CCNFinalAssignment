# Demo Summary

This file is auto-generated from demo sweeps.

- Cache CSV: `../artifacts/simulator-demo/cache_sweep.csv`
- Burst CSV: `../artifacts/simulator-demo/burst_sweep.csv`
- Hetero CSV: `../artifacts/simulator-demo/hetero_sweep.csv`
- Unit-interval CSV: `../artifacts/simulator-demo/unit_interval_sweep.csv`

## Cache Sweep (policy x cache size)
- Best mean latency: `lru` at cache `90` with mean `65.142 ms`.
- Best hit rate: `closer` at cache `90` with hit `0.7498`.

## Burst Sweep (policy x burst_high_rate_mult)
- `closer` mean latency from burst `1` to `6`: `54.245` -> `141.793` ms.
- `lfu` mean latency from burst `1` to `6`: `62.395` -> `187.672` ms.
- `lru` mean latency from burst `1` to `6`: `49.288` -> `117.524` ms.
- `p99aware` mean latency from burst `1` to `6`: `74.376` -> `2658.233` ms.

## Heterogeneous Backhaul Impact
- `closer` mean latency delta (hetero - homo): `0.841` ms.
- `lfu` mean latency delta (hetero - homo): `1.054` ms.
- `lru` mean latency delta (hetero - homo): `0.705` ms.
- `p99aware` mean latency delta (hetero - homo): `-0.098` ms.

## Unit-Interval Calibration (offered load)
- `closer` mean latency from interval `60` to `180`: `312.568` -> `69.036` ms.
- `lfu` mean latency from interval `60` to `180`: `579.059` -> `79.803` ms.
- `lru` mean latency from interval `60` to `180`: `224.607` -> `61.918` ms.
- `p99aware` mean latency from interval `60` to `180`: `6350.057` -> `95.196` ms.

## Suggested live demo flow
- Start with cache sweep plots (`cache_mean.png`, `cache_hit.png`) to establish baseline trade-off.
- Show burst sweep (`burst_mean.png`) to discuss load sensitivity.
- Show unit-interval calibration (`unit_interval_mean.png`) to explain queue pressure control.