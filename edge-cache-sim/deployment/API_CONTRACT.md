# Real Prototype API Contract

This document defines the shared schema used by `origin_server`, `edge_server`, `controller_service`, and `workload_replay`.

## Endpoints

### Origin service

- `GET /health`
- `GET /chunk/{object}/{chunk}`
- gRPC `edgecache.ChunkService/GetChunk`

### Edge service

- `GET /health`
- `GET /chunk/{object}/{chunk}?cache=true|false`
  - Returns `ChunkFetchResponse`
  - Lookup order: local -> neighbors -> origin
- `POST /cache/admit`
  - Request: `CacheAdmitRequest`
- `GET /metrics`
  - Returns `MetricsSnapshot`
- `GET /policy/config`
  - Returns `PolicyConfig`
- `POST /policy/config`
  - Request: `PolicyConfig`

### Controller service

- Polls `GET /metrics` from all edges.
- Pushes `POST /policy/config` to each edge.

### Workload replay

- Sends HTTP requests to edge `/chunk` endpoint.
- Writes per-request rows to CSV for analysis.

## Schemas

All schemas are implemented in [src/real_types.rs](../src/real_types.rs).

### `ChunkFetchResponse`

```json
{
  "object": 10,
  "chunk": 0,
  "found": true,
  "source": "local|neighbor|origin_http|origin_grpc|error",
  "size_bytes": 65536,
  "latency_ms": 3.14
}
```

### `CacheAdmitRequest`

```json
{
  "object": 10,
  "chunk": 0,
  "ttl_seconds": 300
}
```

### `PolicyConfig`

```json
{
  "policy": "lru|lfu|closer|p99aware",
  "p99_w_hit": 1.0,
  "p99_w_tail": 0.08,
  "p99_w_remote": 0.02
}
```

### `MetricsSnapshot`

- `node`: edge node name
- `policy`: currently active policy
- `counters`:
  - `total_requests`
  - `local_hits`
  - `neighbor_hits`
  - `origin_hits`
  - `total_latency_ms`
  - `remote_bytes`
- `p95_ms`, `p99_ms`
- `hit_rate`, `remote_ratio`
- `timestamp_unix_ms`

## Protocol comparison path

Origin currently supports:

- HTTP endpoint `/chunk/{object}/{chunk}`
- gRPC endpoint `ChunkService/GetChunk`

Edge chooses upstream protocol via `--origin-protocol http|grpc`.
