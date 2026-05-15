use anyhow::Context;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use edge_cache_sim::proto::chunk_service_client::ChunkServiceClient;
use edge_cache_sim::proto::ChunkRequest;
use edge_cache_sim::real_types::{
    CacheAdmitRequest, ChunkFetchResponse, MetricsSnapshot, PolicyConfig, RuntimeCounters,
};
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "edge-a")]
    node_name: String,
    #[arg(long, default_value = "127.0.0.1:7001")]
    http_addr: SocketAddr,
    #[arg(long, default_value = "http://127.0.0.1:7000")]
    origin_http: String,
    #[arg(long, default_value = "http://127.0.0.1:7100")]
    origin_grpc: String,
    #[arg(long, default_value = "http")]
    origin_protocol: String,
    #[arg(long, default_value = "")]
    neighbors: String,
    #[arg(long, default_value_t = 512)]
    cache_capacity_chunks: usize,
}

#[derive(Clone, Debug)]
enum UpstreamProtocol {
    Http,
    Grpc,
}

#[derive(Clone)]
struct EdgeState {
    node_name: String,
    origin_http: String,
    origin_grpc: String,
    origin_protocol: UpstreamProtocol,
    neighbors: Vec<String>,
    cache_capacity: usize,
    cache: Arc<RwLock<HashMap<(u32, u32), Vec<u8>>>>,
    lru: Arc<RwLock<VecDeque<(u32, u32)>>>,
    counters: Arc<RwLock<RuntimeCounters>>,
    delays: Arc<RwLock<Vec<f64>>>,
    policy: Arc<RwLock<PolicyConfig>>,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct ChunkQuery {
    cache: Option<bool>,
}

async fn health() -> &'static str {
    "ok"
}

fn default_policy() -> PolicyConfig {
    PolicyConfig {
        policy: "lru".to_string(),
        p99_w_hit: 1.0,
        p99_w_tail: 0.08,
        p99_w_remote: 0.02,
    }
}

async fn insert_cache(state: &EdgeState, key: (u32, u32), data: Vec<u8>) {
    let mut cache = state.cache.write().await;
    let mut lru = state.lru.write().await;
    if cache.contains_key(&key) {
        if let Some(pos) = lru.iter().position(|k| *k == key) {
            lru.remove(pos);
        }
        lru.push_back(key);
        return;
    }
    if cache.len() >= state.cache_capacity {
        if let Some(victim) = lru.pop_front() {
            cache.remove(&victim);
        }
    }
    cache.insert(key, data);
    lru.push_back(key);
}

async fn try_neighbor(
    state: &EdgeState,
    object: u32,
    chunk: u32,
) -> anyhow::Result<Option<ChunkFetchResponse>> {
    for n in &state.neighbors {
        let url = format!("{n}/chunk/{object}/{chunk}?cache=false");
        let resp = state.client.get(url).send().await;
        if let Ok(resp) = resp {
            if resp.status().is_success() {
                let body = resp.json::<ChunkFetchResponse>().await?;
                if body.found {
                    return Ok(Some(ChunkFetchResponse {
                        source: "neighbor".to_string(),
                        ..body
                    }));
                }
            }
        }
    }
    Ok(None)
}

async fn fetch_origin(state: &EdgeState, object: u32, chunk: u32) -> anyhow::Result<ChunkFetchResponse> {
    match state.origin_protocol {
        UpstreamProtocol::Http => {
            let url = format!("{}/chunk/{object}/{chunk}", state.origin_http);
            let body = state.client.get(url).send().await?.json::<ChunkFetchResponse>().await?;
            Ok(ChunkFetchResponse {
                source: "origin_http".to_string(),
                ..body
            })
        }
        UpstreamProtocol::Grpc => {
            let mut client = ChunkServiceClient::connect(state.origin_grpc.clone()).await?;
            let resp = client
                .get_chunk(ChunkRequest {
                    object,
                    chunk,
                })
                .await?
                .into_inner();
            Ok(ChunkFetchResponse {
                object,
                chunk,
                found: resp.found,
                source: "origin_grpc".to_string(),
                size_bytes: resp.data.len(),
                latency_ms: 0.0,
            })
        }
    }
}

async fn get_chunk(
    Path((object, chunk)): Path<(u32, u32)>,
    State(state): State<EdgeState>,
    axum::extract::Query(q): axum::extract::Query<ChunkQuery>,
) -> Json<ChunkFetchResponse> {
    let cache_enabled = q.cache.unwrap_or(true);
    let started = std::time::Instant::now();
    let key = (object, chunk);

    if let Some(bytes) = state.cache.read().await.get(&key).cloned() {
        let latency_ms = started.elapsed().as_secs_f64() * 1000.0;
        {
            let mut c = state.counters.write().await;
            c.total_requests += 1;
            c.local_hits += 1;
            c.total_latency_ms += latency_ms;
        }
        state.delays.write().await.push(latency_ms);
        return Json(ChunkFetchResponse {
            object,
            chunk,
            found: true,
            source: "local".to_string(),
            size_bytes: bytes.len(),
            latency_ms,
        });
    }

    // Neighbor probe mode: only check local cache and return miss immediately.
    if !cache_enabled {
        let latency_ms = started.elapsed().as_secs_f64() * 1000.0;
        return Json(ChunkFetchResponse {
            object,
            chunk,
            found: false,
            source: "neighbor_probe_miss".to_string(),
            size_bytes: 0,
            latency_ms,
        });
    }

    if let Ok(Some(mut n)) = try_neighbor(&state, object, chunk).await {
        if cache_enabled && n.found {
            insert_cache(&state, key, vec![1; n.size_bytes.max(1)]).await;
        }
        n.latency_ms = started.elapsed().as_secs_f64() * 1000.0;
        {
            let mut c = state.counters.write().await;
            c.total_requests += 1;
            c.neighbor_hits += 1;
            c.total_latency_ms += n.latency_ms;
        }
        state.delays.write().await.push(n.latency_ms);
        return Json(n);
    }

    match fetch_origin(&state, object, chunk).await {
        Ok(mut o) => {
            if cache_enabled && o.found {
                insert_cache(&state, key, vec![1; o.size_bytes.max(1)]).await;
            }
            o.latency_ms = started.elapsed().as_secs_f64() * 1000.0;
            {
                let mut c = state.counters.write().await;
                c.total_requests += 1;
                c.origin_hits += 1;
                c.remote_bytes += o.size_bytes as u64;
                c.total_latency_ms += o.latency_ms;
            }
            state.delays.write().await.push(o.latency_ms);
            Json(o)
        }
        Err(_) => Json(ChunkFetchResponse {
            object,
            chunk,
            found: false,
            source: "error".to_string(),
            size_bytes: 0,
            latency_ms: started.elapsed().as_secs_f64() * 1000.0,
        }),
    }
}

async fn cache_admit(State(state): State<EdgeState>, Json(req): Json<CacheAdmitRequest>) -> Json<ChunkFetchResponse> {
    let key = (req.object, req.chunk);
    let up = fetch_origin(&state, req.object, req.chunk).await.ok();
    if let Some(resp) = up {
        insert_cache(&state, key, vec![1; resp.size_bytes.max(1)]).await;
        Json(resp)
    } else {
        Json(ChunkFetchResponse {
            object: req.object,
            chunk: req.chunk,
            found: false,
            source: "admit_failed".to_string(),
            size_bytes: 0,
            latency_ms: 0.0,
        })
    }
}

async fn metrics(State(state): State<EdgeState>) -> Json<MetricsSnapshot> {
    let counters = state.counters.read().await.clone();
    let policy = state.policy.read().await.policy.clone();
    let delays = state.delays.read().await.clone();
    Json(MetricsSnapshot::summarize(
        state.node_name.clone(),
        policy,
        counters,
        &delays,
    ))
}

async fn policy_get(State(state): State<EdgeState>) -> Json<PolicyConfig> {
    Json(state.policy.read().await.clone())
}

async fn policy_set(State(state): State<EdgeState>, Json(cfg): Json<PolicyConfig>) -> Json<PolicyConfig> {
    *state.policy.write().await = cfg.clone();
    Json(cfg)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let args = Args::parse();
    let neighbors = args
        .neighbors
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let origin_protocol = match args.origin_protocol.to_lowercase().as_str() {
        "grpc" => UpstreamProtocol::Grpc,
        _ => UpstreamProtocol::Http,
    };

    let state = EdgeState {
        node_name: args.node_name.clone(),
        origin_http: args.origin_http,
        origin_grpc: args.origin_grpc,
        origin_protocol,
        neighbors,
        cache_capacity: args.cache_capacity_chunks,
        cache: Arc::new(RwLock::new(HashMap::new())),
        lru: Arc::new(RwLock::new(VecDeque::new())),
        counters: Arc::new(RwLock::new(RuntimeCounters::default())),
        delays: Arc::new(RwLock::new(Vec::new())),
        policy: Arc::new(RwLock::new(default_policy())),
        client: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/chunk/:object/:chunk", get(get_chunk))
        .route("/cache/admit", post(cache_admit))
        .route("/metrics", get(metrics))
        .route("/policy/config", get(policy_get).post(policy_set))
        .with_state(state);

    info!("edge node={} listening={}", args.node_name, args.http_addr);
    let listener = tokio::net::TcpListener::bind(args.http_addr)
        .await
        .context("bind edge http")?;
    axum::serve(listener, app).await.context("serve edge")?;
    Ok(())
}
