use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyName {
    Lru,
    Lfu,
    Closer,
    P99Aware,
}

impl PolicyName {
    pub const ALL: [PolicyName; 4] = [Self::Lru, Self::Lfu, Self::Closer, Self::P99Aware];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lru => "lru",
            Self::Lfu => "lfu",
            Self::Closer => "closer",
            Self::P99Aware => "p99aware",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SimConfig {
    pub seed: u64,
    pub num_requests: u64,
    pub warmup_requests: u64,

    pub num_edge_nodes: u32,
    pub local_latency_ms: f64,
    pub local_bw_mbps: f64,
    pub origin_latency_ms: f64,
    pub origin_bw_mbps: f64,

    pub heterogeneous_backhaul: bool,
    pub slow_origin_latency_mult: f64,
    pub slow_origin_bw_mult: f64,

    pub catalog_objects: u32,
    pub chunks_per_object: u8,
    pub chunk_size_kib: u32,
    /// Fraction of requests that target a non-first chunk (1..chunks-1) to stress non-head caching.
    #[serde(default)]
    pub chunk_non_head_prob: f64,

    pub zipf_skew: f64,
    pub num_users: u32,

    pub burst_high_rate_mult: f64,
    pub burst_low_rate_mult: f64,
    pub burst_toggle_mean_requests: u64,
    /// Base inter-arrival interval in abstract time units (larger => lower offered load).
    #[serde(default = "default_request_unit_interval")]
    pub request_unit_interval: f64,

    pub cache_capacity_chunks: usize,

    pub policy: PolicyName,

    pub p99_window_requests: usize,
    pub p99_w_hit: f64,
    pub p99_w_tail: f64,
    pub p99_w_remote: f64,

    pub prediction_noise_sigma: f64,
}

impl SimConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let cfg: SimConfig = toml::from_str(&raw)?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.num_edge_nodes >= 2, "num_edge_nodes must be >= 2");
        anyhow::ensure!(self.catalog_objects >= 1, "catalog_objects must be >= 1");
        anyhow::ensure!(self.chunks_per_object >= 1, "chunks_per_object must be >= 1");
        anyhow::ensure!(self.cache_capacity_chunks >= 1, "cache_capacity_chunks must be >= 1");
        anyhow::ensure!(self.num_requests > self.warmup_requests, "num_requests must exceed warmup");
        anyhow::ensure!(
            (0.0..=1.0).contains(&self.chunk_non_head_prob),
            "chunk_non_head_prob must be in [0,1]"
        );
        anyhow::ensure!(
            self.request_unit_interval > 0.0,
            "request_unit_interval must be > 0"
        );
        Ok(())
    }
}

fn default_request_unit_interval() -> f64 {
    1.0
}