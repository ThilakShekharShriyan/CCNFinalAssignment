//! Integration tests: load configs and run short simulations.

use edge_cache_sim::{simulate, sweep_all_policies, SimConfig};
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn load_default_toml() {
    let p = manifest_dir().join("configs/default.toml");
    SimConfig::load(&p).expect("default.toml parses");
}

#[test]
fn simulate_smoke_config_deterministic() {
    let p = manifest_dir().join("configs/smoke.toml");
    let cfg = SimConfig::load(&p).unwrap();
    let a = simulate(&cfg);
    let b = simulate(&cfg);
    assert_eq!(a.count, b.count);
    assert!(a.count > 0);
    assert!(a.mean_ms.is_finite());
    assert!(a.p95_ms.is_finite());
    assert!(a.p99_ms.is_finite());
    assert!(a.local_hit_rate >= 0.0 && a.local_hit_rate <= 1.0);
}

#[test]
fn sweep_matches_single_policy_lru() {
    let p = manifest_dir().join("configs/smoke.toml");
    let cfg = SimConfig::load(&p).unwrap();
    let mut c = cfg.clone();
    c.policy = edge_cache_sim::PolicyName::Lru;
    let one = simulate(&c);
    let sweep = sweep_all_policies(&cfg);
    let from_sweep = sweep
        .iter()
        .find(|(p, _)| *p == edge_cache_sim::PolicyName::Lru)
        .unwrap()
        .1
        .clone();
    assert_eq!(one.count, from_sweep.count);
    assert!((one.mean_ms - from_sweep.mean_ms).abs() < 1e-6);
}

#[test]
fn all_policies_run() {
    let base = manifest_dir().join("configs/smoke.toml");
    let raw = std::fs::read_to_string(&base).unwrap();
    for pol in ["lru", "lfu", "closer", "p99aware"] {
        let toml = raw.replace("policy = \"lru\"", &format!("policy = \"{pol}\""));
        let cfg: SimConfig = toml::from_str(&toml).unwrap();
        let m = simulate(&cfg);
        assert!(m.count > 0, "policy {pol} produced metrics");
    }
}
