//! Aggregate run metrics (mean / p95 / p99, hit rate, remote bytes, Jain fairness).

#[derive(Clone, Debug, Default)]
pub struct RunMetrics {
    pub count: u64,
    pub mean_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub local_hit_rate: f64,
    pub remote_bytes: u64,
    pub jain_fairness: f64,
}

impl RunMetrics {
    pub fn from_samples(
        delays_ms: &[f64],
        local_hits: u64,
        total_reqs: u64,
        remote_bytes: u64,
        per_user_sum: &[f64],
        per_user_cnt: &[u64],
    ) -> Self {
        if delays_ms.is_empty() || total_reqs == 0 {
            return Self::default();
        }
        let mut sorted = delays_ms.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = sorted.len();
        let mean = sorted.iter().sum::<f64>() / n as f64;
        let p95 = percentile_sorted(&sorted, 0.95);
        let p99 = percentile_sorted(&sorted, 0.99);
        let jain = jain_on_means(per_user_sum, per_user_cnt);
        Self {
            count: n as u64,
            mean_ms: mean,
            p95_ms: p95,
            p99_ms: p99,
            local_hit_rate: local_hits as f64 / total_reqs as f64,
            remote_bytes,
            jain_fairness: jain,
        }
    }
}

fn percentile_sorted(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let pos = (n as f64 - 1.0) * q;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    let w = pos - lo as f64;
    sorted[lo] * (1.0 - w) + sorted[hi] * w
}

#[cfg(test)]
mod tests {
    #[test]
    fn percentile_sorted_basic() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 100.0];
        assert!((super::percentile_sorted(&v, 0.0) - 1.0).abs() < 1e-9);
        assert!((super::percentile_sorted(&v, 1.0) - 100.0).abs() < 1e-9);
        let p50 = super::percentile_sorted(&v, 0.5);
        assert!(p50 >= 2.0 && p50 <= 4.0);
    }

    #[test]
    fn jain_fairness_range() {
        let sums = vec![10.0, 10.0, 10.0];
        let cnts = vec![2u64, 2, 2];
        let j = super::jain_on_means(&sums, &cnts);
        assert!((j - 1.0).abs() < 1e-9);
    }
}

fn jain_on_means(per_user_sum: &[f64], per_user_cnt: &[u64]) -> f64 {
    let mut xs = Vec::new();
    for (s, c) in per_user_sum.iter().zip(per_user_cnt.iter()) {
        if *c > 0 {
            xs.push(*s / *c as f64);
        }
    }
    if xs.is_empty() {
        return 1.0;
    }
    let n = xs.len() as f64;
    let sum: f64 = xs.iter().sum();
    let sum_sq: f64 = xs.iter().map(|x| x * x).sum();
    if sum_sq <= 0.0 {
        return 1.0;
    }
    (sum * sum) / (n * sum_sq)
}
