//! CLI: `cargo run --release -- --config configs/default.toml`
//! Policy sweep: `cargo run --release -- --config configs/phase2.toml --sweep-policies --csv-out results/phase2.csv`
//! Phase-3 sweeps: `--sweep-cache-sizes` and/or `--sweep-heterogeneous`
//! Phase-4: `--sweep-noise-sigmas` (prediction noise for P99Aware eviction)
//! Phase-5: `--sweep-burst-high-mults` (burstiness stress)
//! Phase-6: `--sweep-unit-intervals` (offered-load calibration).

use anyhow::Context;
use clap::Parser;
use edge_cache_sim::{
    comma_separated_f64, comma_separated_usize, simulate, sweep_all_policies, RunMetrics, SimConfig,
};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "edge-cache-sim")]
struct Args {
    /// Path to TOML config (see `configs/`).
    #[arg(long, default_value = "configs/default.toml")]
    config: PathBuf,

    /// Run `Lru`, `Lfu`, `Closer`, and `P99Aware` with identical parameters (ignores `policy` in TOML).
    #[arg(long, default_value_t = false)]
    sweep_policies: bool,

    /// Repeat runs for each cache capacity, e.g. `40,60,90` (independent variable: cache budget).
    #[arg(long, value_name = "N,N,N")]
    sweep_cache_sizes: Option<String>,

    /// Run each scenario with homogeneous backhaul, then heterogeneous (half slow origin links).
    #[arg(long, default_value_t = false)]
    sweep_heterogeneous: bool,

    /// Repeat runs for each `prediction_noise_sigma` (affects P99Aware eviction scores only).
    #[arg(long, value_name = "σ,σ,σ")]
    sweep_noise_sigmas: Option<String>,

    /// Repeat runs for each `burst_high_rate_mult` (burstiness independent variable).
    #[arg(long, value_name = "b,b,b")]
    sweep_burst_high_mults: Option<String>,

    /// Repeat runs for each base inter-arrival interval (larger => lower load).
    #[arg(long, value_name = "u,u,u")]
    sweep_unit_intervals: Option<String>,

    /// Output CSV (required for any batch sweep; optional for a single run — appends one row).
    #[arg(long)]
    csv_out: Option<PathBuf>,
}

fn csv_header() -> &'static str {
    "policy,chunks_per_object,chunk_non_head_prob,cache_capacity_chunks,heterogeneous_backhaul,prediction_noise_sigma,burst_high_rate_mult,burst_low_rate_mult,burst_toggle_mean_requests,request_unit_interval,seed,num_requests,warmup_requests,mean_ms,p95_ms,p99_ms,hit_rate,remote_mb,jain_fairness,count"
}

fn write_csv_row(
    f: &mut dyn Write,
    cfg: &SimConfig,
    policy_label: &str,
    m: &RunMetrics,
) -> anyhow::Result<()> {
    writeln!(
        f,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{}",
        policy_label,
        cfg.chunks_per_object,
        cfg.chunk_non_head_prob,
        cfg.cache_capacity_chunks,
        cfg.heterogeneous_backhaul,
        cfg.prediction_noise_sigma,
        cfg.burst_high_rate_mult,
        cfg.burst_low_rate_mult,
        cfg.burst_toggle_mean_requests,
        cfg.request_unit_interval,
        cfg.seed,
        cfg.num_requests,
        cfg.warmup_requests,
        m.mean_ms,
        m.p95_ms,
        m.p99_ms,
        m.local_hit_rate,
        m.remote_bytes as f64 / (1024.0 * 1024.0),
        m.jain_fairness,
        m.count
    )?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let cfg_path = &args.config;
    let base_cfg = SimConfig::load(cfg_path)
        .with_context(|| format!("failed to load config {}", cfg_path.display()))?;

    let cache_list: Vec<usize> = match &args.sweep_cache_sizes {
        Some(s) => comma_separated_usize(s)?,
        None => vec![base_cfg.cache_capacity_chunks],
    };
    for c in &cache_list {
        anyhow::ensure!(*c >= 1, "cache capacity must be >= 1, got {c}");
    }

    let het_list: Vec<bool> = if args.sweep_heterogeneous {
        vec![false, true]
    } else {
        vec![base_cfg.heterogeneous_backhaul]
    };

    let noise_list: Vec<f64> = match &args.sweep_noise_sigmas {
        Some(s) => comma_separated_f64(s)?,
        None => vec![base_cfg.prediction_noise_sigma],
    };
    for n in &noise_list {
        anyhow::ensure!(*n >= 0.0, "prediction_noise_sigma must be >= 0, got {n}");
    }

    let burst_list: Vec<f64> = match &args.sweep_burst_high_mults {
        Some(s) => comma_separated_f64(s)?,
        None => vec![base_cfg.burst_high_rate_mult],
    };
    for b in &burst_list {
        anyhow::ensure!(*b > 0.0, "burst_high_rate_mult must be > 0, got {b}");
    }

    let unit_interval_list: Vec<f64> = match &args.sweep_unit_intervals {
        Some(s) => comma_separated_f64(s)?,
        None => vec![base_cfg.request_unit_interval],
    };
    for u in &unit_interval_list {
        anyhow::ensure!(*u > 0.0, "request_unit_interval must be > 0, got {u}");
    }

    let is_batch = args.sweep_policies
        || args.sweep_cache_sizes.is_some()
        || args.sweep_heterogeneous
        || args.sweep_noise_sigmas.is_some()
        || args.sweep_burst_high_mults.is_some()
        || args.sweep_unit_intervals.is_some();

    if is_batch {
        let csv_path = args
            .csv_out
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!(
                "batch mode requires --csv-out <path> (use --sweep-policies, --sweep-cache-sizes, --sweep-heterogeneous, --sweep-noise-sigmas, --sweep-burst-high-mults, and/or --sweep-unit-intervals)"
            ))?;

        if let Some(parent) = csv_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
        }
        let mut f = std::fs::File::create(csv_path)
            .with_context(|| format!("create {}", csv_path.display()))?;
        writeln!(f, "{}", csv_header())?;

        let mut rows = 0u64;
        for unit_i in &unit_interval_list {
            for burst in &burst_list {
                for noise in &noise_list {
                    for het in &het_list {
                        for cap in &cache_list {
                            let mut cfg = base_cfg.clone();
                            cfg.request_unit_interval = *unit_i;
                            cfg.burst_high_rate_mult = *burst;
                            cfg.prediction_noise_sigma = *noise;
                            cfg.heterogeneous_backhaul = *het;
                            cfg.cache_capacity_chunks = *cap;

                            if args.sweep_policies {
                                for (p, m) in sweep_all_policies(&cfg) {
                                    write_csv_row(&mut f, &cfg, p.as_str(), &m)?;
                                    rows += 1;
                                }
                            } else {
                                let m = simulate(&cfg);
                                write_csv_row(&mut f, &cfg, cfg.policy.as_str(), &m)?;
                                rows += 1;
                            }
                        }
                    }
                }
            }
        }

        eprintln!("Wrote {rows} rows to {}", csv_path.display());
        return Ok(());
    }

    let m = simulate(&base_cfg);

    println!(
        "policy={:?} count={} mean_ms={:.3} p95_ms={:.3} p99_ms={:.3} hit={:.4} remote_mb={:.3} jain={:.4}",
        base_cfg.policy,
        m.count,
        m.mean_ms,
        m.p95_ms,
        m.p99_ms,
        m.local_hit_rate,
        m.remote_bytes as f64 / (1024.0 * 1024.0),
        m.jain_fairness
    );

    if let Some(csv) = args.csv_out {
        let new = !csv.exists();
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&csv)
            .with_context(|| format!("open {}", csv.display()))?;
        if new {
            writeln!(f, "{}", csv_header())?;
        }
        write_csv_row(&mut f, &base_cfg, base_cfg.policy.as_str(), &m)?;
    }

    Ok(())
}
