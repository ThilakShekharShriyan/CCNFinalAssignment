use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunkFetchResponse {
    pub object: u32,
    pub chunk: u32,
    pub found: bool,
    pub source: String,
    #[serde(default)]
    pub size_bytes: usize,
    #[serde(default)]
    pub latency_ms: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheAdmitRequest {
    pub object: u32,
    pub chunk: u32,
    pub ttl_seconds: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub policy: String,
    pub p99_w_hit: f64,
    pub p99_w_tail: f64,
    pub p99_w_remote: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RuntimeCounters {
    pub total_requests: u64,
    pub local_hits: u64,
    pub neighbor_hits: u64,
    pub origin_hits: u64,
    pub total_latency_ms: f64,
    pub remote_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub node: String,
    pub policy: String,
    pub counters: RuntimeCounters,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub hit_rate: f64,
    pub remote_ratio: f64,
    pub timestamp_unix_ms: u128,
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl MetricsSnapshot {
    pub fn summarize(node: String, policy: String, counters: RuntimeCounters, delays: &[f64]) -> Self {
        let total = counters.total_requests.max(1);
        let hit_rate = counters.local_hits as f64 / total as f64;
        let remote_ratio = counters.origin_hits as f64 / total as f64;
        let mut sorted = delays.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = percentile(&sorted, 0.95);
        let p99 = percentile(&sorted, 0.99);
        Self {
            node,
            policy,
            counters,
            p95_ms: p95,
            p99_ms: p99,
            hit_rate,
            remote_ratio,
            timestamp_unix_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            tags: HashMap::new(),
        }
    }
}

pub fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = (sorted.len() as f64 - 1.0) * q;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    let w = pos - lo as f64;
    sorted[lo] * (1.0 - w) + sorted[hi] * w
}
