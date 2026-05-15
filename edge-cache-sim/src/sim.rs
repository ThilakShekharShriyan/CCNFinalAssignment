//! Main simulation loop: requests, routing, admission, metrics.

use crate::cache::NodeChunkStore;
use crate::config::{PolicyName, SimConfig};
use crate::metrics::RunMetrics;
use crate::network::Network;
use crate::policy::{PolicyEngine, RecentRecord};
use crate::types::ChunkId;
use crate::workload::Workload;

#[derive(Clone, Debug)]
struct Resolved {
    delay_ms: f64,
    origin: bool,
    local_hit: bool,
}

fn resolve_fetch(
    net: &Network,
    home: usize,
    chunk: ChunkId,
    chunk_bytes: f64,
    stores: &[NodeChunkStore],
) -> Resolved {
    if stores[home].contains(chunk) {
        return Resolved {
            delay_ms: PolicyEngine::local_hit_delay_ms(),
            origin: false,
            local_hit: true,
        };
    }

    let n = stores.len();
    let origin = net.origin;
    let mut best_d = f64::INFINITY;
    let mut best_origin = true;

    for v in 0..n {
        if v == home {
            continue;
        }
        if stores[v].contains(chunk) {
            if let Some(path) = net.shortest_latency_path(home, v) {
                let d = net.path_transfer_delay_ms(&path, chunk_bytes);
                if d < best_d {
                    best_d = d;
                    best_origin = false;
                }
            }
        }
    }

    if let Some(path) = net.shortest_latency_path(home, origin) {
        let d = net.path_transfer_delay_ms(&path, chunk_bytes);
        if d < best_d {
            best_d = d;
            best_origin = true;
        }
    }

    if !best_d.is_finite() {
        best_d = 0.0;
    }

    Resolved {
        delay_ms: best_d,
        origin: best_origin,
        local_hit: false,
    }
}

/// Run the same scenario for all four policies (ignores `cfg.policy`).
pub fn sweep_all_policies(cfg: &SimConfig) -> Vec<(PolicyName, RunMetrics)> {
    PolicyName::ALL
        .into_iter()
        .map(|p| {
            let mut c = cfg.clone();
            c.policy = p;
            let m = simulate(&c);
            (p, m)
        })
        .collect()
}

pub fn simulate(cfg: &SimConfig) -> RunMetrics {
    let n = cfg.num_edge_nodes as usize;
    let net = Network::from_config(cfg);
    let mut stores: Vec<NodeChunkStore> =
        (0..n).map(|_| NodeChunkStore::new(cfg.cache_capacity_chunks)).collect();
    let mut policy = PolicyEngine::new(cfg, n);
    let mut wl = Workload::new(cfg);
    let chunk_bytes = cfg.chunk_size_kib as f64 * 1024.0;

    let mut delays = Vec::new();
    let mut local_hits = 0u64;
    let mut remote_bytes = 0u64;
    let nu = cfg.num_users as usize;
    let mut per_user_sum = vec![0.0f64; nu];
    let mut per_user_cnt = vec![0u64; nu];
    let mut now = 0.0f64;
    // Simple per-edge FIFO queue abstraction to make burstiness matter.
    let mut edge_busy_until = vec![0.0f64; n];

    for i in 0..cfg.num_requests {
        let (user, chunk, inter_arrival) = wl.stream.next();
        now += inter_arrival;
        let edge = wl.user_home_edge(user);
        let mut res = resolve_fetch(&net, edge, chunk, chunk_bytes, &stores);
        let start = now.max(edge_busy_until[edge]);
        let queue_wait = (start - now).max(0.0);
        edge_busy_until[edge] = start + res.delay_ms;
        res.delay_ms += queue_wait;

        if stores[edge].contains(chunk) {
            policy.on_access(edge, chunk);
        } else {
            policy.admit(edge, chunk, &mut stores[edge]);
        }

        policy.push_recent(RecentRecord {
            object: chunk.object,
            delay_ms: res.delay_ms,
            origin: res.origin,
        });

        if i >= cfg.warmup_requests {
            delays.push(res.delay_ms);
            if res.local_hit {
                local_hits += 1;
            }
            if res.origin {
                remote_bytes += cfg.chunk_size_kib as u64 * 1024;
            }
            let u = user as usize;
            per_user_sum[u] += res.delay_ms;
            per_user_cnt[u] += 1;
        }
    }

    let measured = cfg.num_requests - cfg.warmup_requests;
    RunMetrics::from_samples(
        &delays,
        local_hits,
        measured,
        remote_bytes,
        &per_user_sum,
        &per_user_cnt,
    )
}
