use anyhow::Context;
use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};
use clap::Parser;
use edge_cache_sim::proto::chunk_service_server::{ChunkService, ChunkServiceServer};
use edge_cache_sim::proto::{ChunkReply, ChunkRequest};
use edge_cache_sim::real_types::ChunkFetchResponse;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::info;

type ChunkMap = Arc<RwLock<HashMap<(u32, u32), Vec<u8>>>>;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "127.0.0.1:7000")]
    http_addr: SocketAddr,
    #[arg(long, default_value = "127.0.0.1:7100")]
    grpc_addr: SocketAddr,
    #[arg(long, default_value_t = 1024)]
    objects: u32,
    #[arg(long, default_value_t = 4)]
    chunks_per_object: u32,
    #[arg(long, default_value_t = 64)]
    chunk_size_kib: usize,
}

#[derive(Clone)]
struct GrpcSvc {
    chunks: ChunkMap,
}

#[tonic::async_trait]
impl ChunkService for GrpcSvc {
    async fn get_chunk(&self, req: Request<ChunkRequest>) -> Result<Response<ChunkReply>, Status> {
        let req = req.into_inner();
        let key = (req.object, req.chunk);
        let map = self.chunks.read().await;
        if let Some(data) = map.get(&key) {
            Ok(Response::new(ChunkReply {
                data: data.clone(),
                found: true,
                source: "origin_grpc".to_string(),
            }))
        } else {
            Ok(Response::new(ChunkReply {
                data: Vec::new(),
                found: false,
                source: "miss".to_string(),
            }))
        }
    }
}

async fn http_get_chunk(Path((object, chunk)): Path<(u32, u32)>, state: axum::extract::State<ChunkMap>) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let key = (object, chunk);
    let map = state.read().await;
    let (found, size_bytes) = match map.get(&key) {
        Some(bytes) => (true, bytes.len()),
        None => (false, 0),
    };
    Json(ChunkFetchResponse {
        object,
        chunk,
        found,
        source: "origin_http".to_string(),
        size_bytes,
        latency_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

fn build_chunks(objects: u32, chunks_per_object: u32, chunk_size_bytes: usize) -> HashMap<(u32, u32), Vec<u8>> {
    let mut map = HashMap::new();
    for o in 0..objects {
        for c in 0..chunks_per_object {
            let fill = ((o + c) % 251) as u8;
            map.insert((o, c), vec![fill; chunk_size_bytes]);
        }
    }
    map
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let args = Args::parse();
    let chunk_size = args.chunk_size_kib * 1024;
    let chunks: ChunkMap = Arc::new(RwLock::new(build_chunks(
        args.objects,
        args.chunks_per_object,
        chunk_size,
    )));

    let http_app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/chunk/:object/:chunk", get(http_get_chunk))
        .with_state(chunks.clone());

    let grpc_svc = GrpcSvc { chunks: chunks.clone() };
    let http_addr = args.http_addr;
    let grpc_addr = args.grpc_addr;
    info!("origin http={} grpc={}", http_addr, grpc_addr);

    let http_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(http_addr).await.context("bind origin http")?;
        axum::serve(listener, http_app).await.context("serve origin http")
    });
    let grpc_task = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(ChunkServiceServer::new(grpc_svc))
            .serve(grpc_addr)
            .await
            .context("serve origin grpc")
    });

    tokio::select! {
        r = http_task => r??,
        r = grpc_task => r??,
        _ = tokio::signal::ctrl_c() => {}
    }
    Ok(())
}
