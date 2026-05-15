use anyhow::Context;
use clap::Parser;
use edge_cache_sim::real_types::ChunkFetchResponse;
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::io::Write;
use std::time::Instant;
use tracing::info;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "http://127.0.0.1:7001,http://127.0.0.1:7002,http://127.0.0.1:7003")]
    edges: String,
    #[arg(long, default_value_t = 10_000)]
    requests: u64,
    #[arg(long, default_value_t = 400)]
    catalog_objects: u32,
    #[arg(long, default_value_t = 4)]
    chunks_per_object: u8,
    #[arg(long, default_value_t = 0.18)]
    chunk_non_head_prob: f64,
    #[arg(long, default_value_t = 1.15)]
    zipf_skew: f64,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 0.0)]
    request_interval_ms: f64,
    #[arg(long, default_value = "results_real/replay_requests.csv")]
    csv_out: String,
}

fn zipf_weights(num_objects: u32, s: f64) -> Vec<f64> {
    (1..=num_objects)
        .map(|i| 1.0 / (i as f64).powf(s))
        .collect()
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
    anyhow::ensure!(!edges.is_empty(), "at least one edge endpoint is required");

    if let Some(parent) = std::path::Path::new(&args.csv_out).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out = std::fs::File::create(&args.csv_out)?;
    writeln!(out, "req_id,edge,object,chunk,source,found,latency_ms,size_bytes")?;

    let weights = zipf_weights(args.catalog_objects, args.zipf_skew);
    let dist = WeightedIndex::new(&weights).context("build zipf dist")?;
    let mut rng = StdRng::seed_from_u64(args.seed);
    let http = reqwest::Client::new();
    let mut success = 0u64;

    for i in 0..args.requests {
        let edge = &edges[rng.gen_range(0..edges.len())];
        let object = dist.sample(&mut rng) as u32;
        let chunk = if args.chunks_per_object <= 1 {
            0u32
        } else if rng.gen::<f64>() < args.chunk_non_head_prob {
            rng.gen_range(1..args.chunks_per_object as u32)
        } else {
            0u32
        };

        let url = format!("{edge}/chunk/{object}/{chunk}");
        let t0 = Instant::now();
        let resp = http.get(url).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let mut body: ChunkFetchResponse = r.json().await?;
                if body.latency_ms <= 0.0 {
                    body.latency_ms = t0.elapsed().as_secs_f64() * 1000.0;
                }
                if body.found {
                    success += 1;
                }
                writeln!(
                    out,
                    "{},{},{},{},{},{},{:.6},{}",
                    i, edge, object, chunk, body.source, body.found, body.latency_ms, body.size_bytes
                )?;
            }
            Ok(r) => {
                writeln!(out, "{},{},{},{},error_status_{},false,0.0,0", i, edge, object, chunk, r.status())?;
            }
            Err(e) => {
                writeln!(out, "{},{},{},{},error_req_{},false,0.0,0", i, edge, object, chunk, e)?;
            }
        }
        if args.request_interval_ms > 0.0 {
            tokio::time::sleep(std::time::Duration::from_secs_f64(
                args.request_interval_ms / 1000.0,
            ))
            .await;
        }
    }

    info!(
        "replay complete requests={} success_rate={:.4} output={}",
        args.requests,
        success as f64 / args.requests.max(1) as f64,
        args.csv_out
    );
    Ok(())
}
