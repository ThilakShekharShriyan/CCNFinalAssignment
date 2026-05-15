//! Latency/bandwidth graph among edge nodes plus origin (cloud).

use crate::config::SimConfig;

#[derive(Clone, Debug)]
pub struct Link {
    pub to: usize,
    pub latency_ms: f64,
    pub bw_mbps: f64,
}

#[derive(Clone, Debug)]
pub struct Network {
    /// Vertex count = num_edge_nodes + 1 (last index is origin).
    pub origin: usize,
    pub adj: Vec<Vec<Link>>,
}

impl Network {
    pub fn from_config(cfg: &SimConfig) -> Self {
        let n = cfg.num_edge_nodes as usize;
        let origin = n;
        let mut adj = vec![Vec::new(); n + 1];

        // Full mesh among edge nodes (symmetric).
        for i in 0..n {
            for j in (i + 1)..n {
                adj[i].push(Link {
                    to: j,
                    latency_ms: cfg.local_latency_ms,
                    bw_mbps: cfg.local_bw_mbps,
                });
                adj[j].push(Link {
                    to: i,
                    latency_ms: cfg.local_latency_ms,
                    bw_mbps: cfg.local_bw_mbps,
                });
            }
        }

        // Each edge node connects to origin (backhaul).
        for i in 0..n {
            let (lat, bw) = if cfg.heterogeneous_backhaul && i >= n / 2 {
                (
                    cfg.origin_latency_ms * cfg.slow_origin_latency_mult,
                    cfg.origin_bw_mbps * cfg.slow_origin_bw_mult,
                )
            } else {
                (cfg.origin_latency_ms, cfg.origin_bw_mbps)
            };
            adj[i].push(Link {
                to: origin,
                latency_ms: lat,
                bw_mbps: bw,
            });
            adj[origin].push(Link {
                to: i,
                latency_ms: lat,
                bw_mbps: bw,
            });
        }

        Self { origin, adj }
    }

    /// Shortest path by **latency only**; returns vertex sequence `src -> ... -> dst`.
    pub fn shortest_latency_path(&self, src: usize, dst: usize) -> Option<Vec<usize>> {
        if src == dst {
            return Some(vec![src]);
        }
        let n = self.adj.len();
        let mut dist = vec![f64::INFINITY; n];
        let mut parent = vec![None; n];
        dist[src] = 0.0;

        let mut visited = vec![false; n];
        for _ in 0..n {
            let u = (0..n)
                .filter(|&i| !visited[i])
                .min_by(|&a, &b| dist[a].partial_cmp(&dist[b]).unwrap())?;
            if dist[u].is_infinite() {
                break;
            }
            visited[u] = true;
            for e in &self.adj[u] {
                let nd = dist[u] + e.latency_ms;
                if nd < dist[e.to] {
                    dist[e.to] = nd;
                    parent[e.to] = Some(u);
                }
            }
        }

        if parent[dst].is_none() && src != dst {
            return None;
        }

        let mut path = vec![dst];
        let mut cur = dst;
        while cur != src {
            cur = parent[cur]?;
            path.push(cur);
        }
        path.reverse();
        Some(path)
    }

    /// Propagation latency sum + transfer time for `bytes` at bottleneck link capacity along `path`.
    pub fn path_transfer_delay_ms(&self, path: &[usize], bytes: f64) -> f64 {
        if path.len() < 2 {
            return 0.0;
        }
        let mut sum_lat = 0.0;
        let mut min_bw = f64::INFINITY;
        for w in path.windows(2) {
            let a = w[0];
            let b = w[1];
            let link = self
                .adj[a]
                .iter()
                .find(|l| l.to == b)
                .expect("path edge must exist");
            sum_lat += link.latency_ms;
            min_bw = min_bw.min(link.bw_mbps);
        }
        let transfer_ms = if min_bw.is_finite() && min_bw > 0.0 {
            // Mbps = megabits/s; bytes * 8 / (bw * 1e6) = seconds
            (bytes * 8.0) / (min_bw * 1_000_000.0) * 1000.0
        } else {
            f64::INFINITY
        };
        sum_lat + transfer_ms
    }
}
