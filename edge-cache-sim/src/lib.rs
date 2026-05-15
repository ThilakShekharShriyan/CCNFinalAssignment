//! Edge chunk caching simulator for collaborative small-cell style scenarios.
pub mod cache;
pub mod config;
pub mod metrics;
pub mod network;
pub mod parse;
pub mod policy;
pub mod real_types;
pub mod sim;
pub mod types;
pub mod workload;

pub mod proto {
    tonic::include_proto!("edgecache");
}

pub use config::{PolicyName, SimConfig};
pub use metrics::RunMetrics;
pub use parse::{comma_separated_f64, comma_separated_usize};
pub use sim::{simulate, sweep_all_policies};
