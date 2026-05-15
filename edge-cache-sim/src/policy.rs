//! Cache replacement policies: LRU, LFU, CLoSER-style, P99-aware.

use crate::cache::NodeChunkStore;
use crate::config::{PolicyName, SimConfig};
use crate::types::ChunkId;
use rand::distributions::Distribution;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::Normal;
use std::collections::{HashMap, VecDeque};

const LOCAL_HIT_MS: f64 = 0.15;

/// Recent request records for tail-aware eviction (post-warmup recommended).
#[derive(Clone, Debug)]
pub struct RecentRecord {
    pub object: u32,
    pub delay_ms: f64,
    pub origin: bool,
}

pub struct PolicyEngine {
    pub kind: PolicyName,
    /// Per-edge LRU order: front = LRU victim candidate.
    lru: Vec<VecDeque<ChunkId>>,
    /// Per-edge LFU frequencies (updated on each access at that edge).
    lfu: Vec<HashMap<ChunkId, u64>>,
    /// Static Zipf prior 1/(rank^s) for `Closer` tie-breaking.
    zipf_prior: Vec<f64>,
    pub recent: VecDeque<RecentRecord>,
    pub recent_cap: usize,
    pub p99_w_hit: f64,
    pub p99_w_tail: f64,
    pub p99_w_remote: f64,
    pub prediction_noise_sigma: f64,
    noise_rng: StdRng,
}

impl PolicyEngine {
    pub fn new(cfg: &SimConfig, num_edges: usize) -> Self {
        let zipf_prior: Vec<f64> = (1..=cfg.catalog_objects)
            .map(|i| 1.0 / (i as f64).powf(cfg.zipf_skew))
            .collect();
        Self {
            kind: cfg.policy,
            lru: vec![VecDeque::new(); num_edges],
            lfu: vec![HashMap::new(); num_edges],
            zipf_prior,
            recent: VecDeque::new(),
            recent_cap: cfg.p99_window_requests.max(100),
            p99_w_hit: cfg.p99_w_hit,
            p99_w_tail: cfg.p99_w_tail,
            p99_w_remote: cfg.p99_w_remote,
            prediction_noise_sigma: cfg.prediction_noise_sigma,
            noise_rng: StdRng::seed_from_u64(cfg.seed ^ 0x51ED_DEAD_BEEF),
        }
    }

    pub fn local_hit_delay_ms() -> f64 {
        LOCAL_HIT_MS
    }

    /// Called after each request completes at `edge` for `chunk` (hit or after admission).
    pub fn on_access(&mut self, edge: usize, chunk: ChunkId) {
        match self.kind {
            PolicyName::Lru | PolicyName::Closer | PolicyName::P99Aware => {
                let q = &mut self.lru[edge];
                if let Some(pos) = q.iter().position(|&c| c == chunk) {
                    q.remove(pos);
                }
                q.push_back(chunk);
            }
            PolicyName::Lfu => {}
        }
        let f = self.lfu[edge].entry(chunk).or_insert(0);
        *f += 1;
    }

    pub fn push_recent(&mut self, rec: RecentRecord) {
        self.recent.push_back(rec);
        while self.recent.len() > self.recent_cap {
            self.recent.pop_front();
        }
    }

    /// Insert `chunk` at `edge` and evict if over capacity. Assumes `chunk` not present.
    pub fn admit(&mut self, edge: usize, chunk: ChunkId, store: &mut NodeChunkStore) {
        if store.contains(chunk) {
            return;
        }
        if store.is_full() {
            if let Some(victim) = self.pick_victim(edge, store) {
                self.remove_tracking(edge, &victim);
                store.remove(&victim);
            }
        }
        store.insert(chunk);
        self.on_access(edge, chunk);
    }

    fn remove_tracking(&mut self, edge: usize, chunk: &ChunkId) {
        self.lfu[edge].remove(chunk);
        if let Some(pos) = self.lru[edge].iter().position(|c| c == chunk) {
            self.lru[edge].remove(pos);
        }
    }

    fn pick_victim(&mut self, edge: usize, store: &NodeChunkStore) -> Option<ChunkId> {
        match self.kind {
            PolicyName::Lru => self.lru[edge].front().copied(),
            PolicyName::Lfu => {
                let mut best: Option<(ChunkId, u64)> = None;
                for c in store.iter() {
                    let f = *self.lfu[edge].get(&c).unwrap_or(&0);
                    match best {
                        None => best = Some((c, f)),
                        Some((bc, bf)) => {
                            if f < bf || (f == bf && c < bc) {
                                best = Some((c, f));
                            }
                        }
                    }
                }
                best.map(|(c, _)| c)
            }
            PolicyName::Closer => {
                // Keep first chunks of popular objects longer: higher eviction score = more likely to evict.
                let mut worst: Option<(ChunkId, f64)> = None;
                for c in store.iter() {
                    let f = *self.lfu[edge].get(&c).unwrap_or(&0) as f64;
                    let z = self.zipf_prior.get(c.object as usize).copied().unwrap_or(1e-9);
                    let first_boost = if c.chunk == 0 { 2.5 } else { 1.0 };
                    // Importance to keep ~ f * z * first_boost ; evict lowest importance.
                    let keep_score = f * z * first_boost;
                    match worst {
                        None => worst = Some((c, keep_score)),
                        Some((bc, bs)) => {
                            if keep_score < bs || (keep_score == bs && c < bc) {
                                worst = Some((c, keep_score));
                            }
                        }
                    }
                }
                worst.map(|(c, _)| c)
            }
            PolicyName::P99Aware => self.pick_victim_p99(edge, store),
        }
    }

    fn pick_victim_p99(&mut self, _edge: usize, store: &NodeChunkStore) -> Option<ChunkId> {
        // Build per-object aggregates from sliding window.
        let mut hits: HashMap<u32, u64> = HashMap::new();
        let mut delays: HashMap<u32, Vec<f64>> = HashMap::new();
        let mut origins: HashMap<u32, u64> = HashMap::new();
        for r in &self.recent {
            *hits.entry(r.object).or_insert(0) += 1;
            delays.entry(r.object).or_default().push(r.delay_ms);
            if r.origin {
                *origins.entry(r.object).or_insert(0) += 1;
            }
        }

        let mut worst: Option<(ChunkId, f64)> = None;
        for c in store.iter() {
            let o = c.object;
            let h = *hits.get(&o).unwrap_or(&0) as f64;
            let mut v = delays.get(&o).cloned().unwrap_or_default();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let tail = percentile(&v, 0.95).unwrap_or(0.0);
            let orig = *origins.get(&o).unwrap_or(&0) as f64;
            let mut score = self.p99_w_hit * h - self.p99_w_tail * tail - self.p99_w_remote * orig;
            if self.prediction_noise_sigma > 0.0 {
                let noise = Normal::new(0.0, self.prediction_noise_sigma).unwrap();
                score += noise.sample(&mut self.noise_rng);
            }
            // Evict smallest score (least valuable to keep for tail/mean balance).
            match worst {
                None => worst = Some((c, score)),
                Some((bc, bs)) => {
                    if score < bs || (score == bs && c < bc) {
                        worst = Some((c, score));
                    }
                }
            }
        }
        worst.map(|(c, _)| c)
    }
}

fn percentile(sorted: &[f64], q: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let pos = (sorted.len() as f64 - 1.0) * q;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return Some(sorted[lo]);
    }
    let w = pos - lo as f64;
    Some(sorted[lo] * (1.0 - w) + sorted[hi] * w)
}
