//! Synthetic Zipf object demand + on/off burstiness + user→edge attachment.

use crate::config::SimConfig;
use crate::types::ChunkId;
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn zipf_weights(num_objects: u32, s: f64) -> Vec<f64> {
    (1..=num_objects)
        .map(|i| 1.0 / (i as f64).powf(s))
        .collect()
}

pub struct Workload {
    pub user_edge: Vec<u32>,
    pub stream: RequestStream,
}

pub struct RequestStream {
    rng: StdRng,
    object_dist: WeightedIndex<f64>,
    catalog_objects: u32,
    chunks_per_object: u8,
    chunk_non_head_prob: f64,
    num_users: u32,
    burst_high: f64,
    burst_low: f64,
    burst_toggle_mean: u64,
    requests_until_toggle: u64,
    burst_on: bool,
    unit_interval: f64,
}

impl Workload {
    pub fn new(cfg: &SimConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(cfg.seed);
        let mut user_edge = vec![0u32; cfg.num_users as usize];
        for u in &mut user_edge {
            *u = rng.gen_range(0..cfg.num_edge_nodes);
        }
        Self {
            user_edge,
            stream: RequestStream::new(cfg, rng),
        }
    }

    pub fn user_home_edge(&self, user: u32) -> usize {
        self.user_edge[user as usize] as usize
    }
}

impl RequestStream {
    fn new(cfg: &SimConfig, rng: StdRng) -> Self {
        let w = zipf_weights(cfg.catalog_objects, cfg.zipf_skew);
        let object_dist = WeightedIndex::new(&w).expect("positive weights");
        Self {
            rng,
            object_dist,
            catalog_objects: cfg.catalog_objects,
            chunks_per_object: cfg.chunks_per_object,
            chunk_non_head_prob: cfg.chunk_non_head_prob,
            num_users: cfg.num_users,
            burst_high: cfg.burst_high_rate_mult,
            burst_low: cfg.burst_low_rate_mult,
            burst_toggle_mean: cfg.burst_toggle_mean_requests.max(1),
            requests_until_toggle: cfg.burst_toggle_mean_requests.max(1),
            burst_on: true,
            unit_interval: cfg.request_unit_interval,
        }
    }

    fn reschedule_toggle(&mut self) {
        let mean = self.burst_toggle_mean as f64;
        let u: f64 = self.rng.gen();
        let steps = (-u.ln() * mean).round() as u64;
        self.requests_until_toggle = steps.max(1);
    }

    /// Next request as (user, chunk, inter_arrival_time).
    pub fn next(&mut self) -> (u32, ChunkId, f64) {
        if self.requests_until_toggle == 0 {
            self.burst_on = !self.burst_on;
            self.reschedule_toggle();
        } else {
            self.requests_until_toggle -= 1;
        }

        let interval = self.unit_interval
            / if self.burst_on {
                self.burst_high
            } else {
                self.burst_low
            };

        let user = self.rng.gen_range(0..self.num_users);
        let idx = self.object_dist.sample(&mut self.rng) as u32;
        let object = idx.min(self.catalog_objects.saturating_sub(1));
        let k = self.chunks_per_object.max(1);
        let chunk_idx = if k <= 1 {
            0u8
        } else if self.rng.gen::<f64>() < self.chunk_non_head_prob {
            self.rng.gen_range(1..k as u32) as u8
        } else {
            0u8
        };
        let chunk = ChunkId {
            object,
            chunk: chunk_idx,
        };
        (user, chunk, interval)
    }
}
