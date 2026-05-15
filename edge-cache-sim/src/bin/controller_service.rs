use anyhow::Context;
use clap::Parser;
use edge_cache_sim::real_types::{MetricsSnapshot, PolicyConfig};
use std::time::Duration;
use tracing::{info, warn};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "http://127.0.0.1:7001,http://127.0.0.1:7002,http://127.0.0.1:7003")]
    edges: String,
    #[arg(long, default_value_t = 5)]
    interval_seconds: u64,
}

fn decide_policy(m: &MetricsSnapshot) -> PolicyConfig {
    if m.p99_ms > 1_000.0 {
        PolicyConfig {
            policy: "p99aware".into(),
            p99_w_hit: 1.0,
            p99_w_tail: 0.12,
            p99_w_remote: 0.03,
        }
    } else if m.remote_ratio > 0.35 {
        PolicyConfig {
            policy: "closer".into(),
            p99_w_hit: 1.0,
            p99_w_tail: 0.08,
            p99_w_remote: 0.02,
        }
    } else {
        PolicyConfig {
            policy: "lru".into(),
            p99_w_hit: 1.0,
            p99_w_tail: 0.08,
            p99_w_remote: 0.02,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let args = Args::parse();
    let edges = args
        .edges
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let http = reqwest::Client::new();
    info!("controller watching {} edges", edges.len());

    loop {
        for e in &edges {
            let metrics_url = format!("{e}/metrics");
            match http.get(&metrics_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let snapshot: MetricsSnapshot = resp.json().await.context("decode metrics")?;
                    let next = decide_policy(&snapshot);
                    let set_url = format!("{e}/policy/config");
                    if let Err(err) = http.post(set_url).json(&next).send().await {
                        warn!("failed policy update edge={} err={}", e, err);
                    } else {
                        info!(
                            "edge={} policy={} p99={:.2} remote_ratio={:.3}",
                            snapshot.node, next.policy, snapshot.p99_ms, snapshot.remote_ratio
                        );
                    }
                }
                Ok(resp) => warn!("metrics request failed edge={} status={}", e, resp.status()),
                Err(err) => warn!("metrics request error edge={} err={}", e, err),
            }
        }
        tokio::time::sleep(Duration::from_secs(args.interval_seconds)).await;
    }
}
