// Copyright (c) 2026 rezky_nightky

//! Headless benchmark runners for Cosmostrix.
//!
//! Two modes:
//! - `--bench-frames N`: CI/regression benchmark, prints legacy `BENCH:` format.
//! - `--benchmark`: Premium user-facing 5-second benchmark with Report engine output.

use std::env;
use std::time::{Duration, Instant};

use crate::constants::{
    BENCH_ELAPSED_MIN_S, DENSITY_AUTO_DEFAULT_COLS, DENSITY_AUTO_DEFAULT_LINES, MAX_TERMINAL_COLS,
    MAX_TERMINAL_LINES,
};
use crate::diagnostics;
use crate::frame::Frame;
use crate::renderer_info;
use crate::report::Report;

use super::{effective_density, CloudConfig};

/// Duration of the premium benchmark in seconds.
const BENCHMARK_DURATION_SECS: u64 = 5;

/// Warmup duration for the premium benchmark in seconds.
const BENCHMARK_WARMUP_SECS: u64 = 1;

/// Number of frame time samples for percentile calculations.
const FRAME_TIME_SAMPLES: usize = 10_000;

/// Legacy CI benchmark: run N frames and print results in the original format.
/// Output format is preserved for backwards compatibility.
pub fn run_benchmark(cfg: &CloudConfig) -> std::io::Result<()> {
    let bench_frames = cfg.bench_frames.expect("bench_frames must be set");

    let (w, h) = bench_dimensions();

    let density = effective_density(cfg.base_density, w, h, cfg.fullwidth, cfg.density_auto);

    let mut cloud = cfg.create_cloud(density);
    cloud.reset(w, h);

    let mut frame = Frame::new(w, h, cloud.palette.bg);

    let target_period = Duration::from_secs_f64(1.0 / cfg.target_fps);
    cloud.set_max_sim_delta(target_period);

    let warmup_frames = (bench_frames / 10).clamp(10, 200);
    let mut sim_now = Instant::now();

    for _ in 0..warmup_frames {
        sim_now += target_period;
        cloud.rain_at(&mut frame, sim_now);
        frame.clear_dirty();
    }

    let start = Instant::now();
    for _ in 0..bench_frames {
        sim_now += target_period;
        cloud.rain_at(&mut frame, sim_now);
        frame.clear_dirty();
    }
    let elapsed_s = start.elapsed().as_secs_f64().max(BENCH_ELAPSED_MIN_S);
    let fps = (bench_frames as f64) / elapsed_s;

    println!("BENCH:");
    println!("  cols: {}", w);
    println!("  lines: {}", h);
    println!("  frames: {}", bench_frames);
    println!("  elapsed_s: {:.6}", elapsed_s);
    println!("  frames_per_s: {:.3}", fps);
    Ok(())
}

/// Premium user-facing benchmark: runs for 5 seconds with enhanced metrics.
pub fn run_premium_benchmark(cfg: &CloudConfig) -> std::io::Result<()> {
    let (w, h) = bench_dimensions();
    let density = effective_density(cfg.base_density, w, h, cfg.fullwidth, cfg.density_auto);

    let mut cloud = cfg.create_cloud(density);
    cloud.reset(w, h);

    let mut frame = Frame::new(w, h, cloud.palette.bg);

    let target_period = Duration::from_secs_f64(1.0 / cfg.target_fps);
    cloud.set_max_sim_delta(target_period);

    // --- Warmup phase ---
    let warmup_end = Instant::now() + Duration::from_secs(BENCHMARK_WARMUP_SECS);
    let mut sim_now = Instant::now();
    while Instant::now() < warmup_end {
        sim_now += target_period;
        cloud.rain_at(&mut frame, sim_now);
        frame.clear_dirty();
    }

    // --- Measurement phase ---
    let mut frame_times: [f64; FRAME_TIME_SAMPLES] = [0.0; FRAME_TIME_SAMPLES];
    let mut ft_index: usize = 0;
    let mut total_frames: u64 = 0;
    let mut drawn_frames: u64 = 0;
    let mut total_ansi_bytes: u64 = 0;
    let mut active_streams_sum: u64 = 0;

    let start = Instant::now();
    let bench_end = start + Duration::from_secs(BENCHMARK_DURATION_SECS);

    while Instant::now() < bench_end {
        sim_now += target_period;

        let frame_start = Instant::now();
        cloud.rain_at(&mut frame, sim_now);

        let did_draw = frame.is_dirty_all() || !frame.dirty_indices().is_empty();
        if did_draw {
            drawn_frames += 1;
            // Estimate ANSI bytes for dirty cells (rough: ~20 bytes per cell
            // for SGR + char + reset)
            let dirty_count = if frame.is_dirty_all() {
                (w as usize) * (h as usize)
            } else {
                frame.dirty_indices().len()
            };
            total_ansi_bytes += (dirty_count as u64) * 20;
        }

        frame.clear_dirty();

        let frame_time_ms = frame_start.elapsed().as_secs_f64() * 1000.0;
        if ft_index < FRAME_TIME_SAMPLES {
            frame_times[ft_index] = frame_time_ms;
            ft_index += 1;
        }
        total_frames += 1;
        active_streams_sum += cloud.droplet_count() as u64;
    }

    let elapsed = start.elapsed();
    let elapsed_s = elapsed.as_secs_f64().max(BENCH_ELAPSED_MIN_S);

    // --- Compute metrics ---
    let avg_fps = (total_frames as f64) / elapsed_s;
    let peak_fps = 1000.0
        / frame_times[..ft_index]
            .iter()
            .copied()
            .fold(f64::MAX, f64::min);
    let avg_frame_time = frame_times[..ft_index].iter().sum::<f64>() / (ft_index as f64).max(1.0);

    // p99 frame time
    let mut sorted_ft: Vec<f64> = frame_times[..ft_index].to_vec();
    sorted_ft.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p99_frame_time = if ft_index > 0 {
        sorted_ft[(((ft_index as f64) * 0.99) as usize).min(ft_index - 1)]
    } else {
        0.0
    };

    // Frame jitter: standard deviation of frame times
    let variance: f64 = if ft_index > 1 {
        let mean = avg_frame_time;
        frame_times[..ft_index]
            .iter()
            .map(|&t| (t - mean) * (t - mean))
            .sum::<f64>()
            / (ft_index - 1) as f64
    } else {
        0.0
    };
    let jitter_std = variance.sqrt();
    let jitter_classification = if jitter_std < 0.5 {
        "low"
    } else if jitter_std < 2.0 {
        "medium"
    } else {
        "high"
    };

    // Glyphs per second: drawn_frames * average dirty cells per drawn frame / elapsed
    let total_cells = (w as u64) * (h as u64);
    let glyphs_per_second = if drawn_frames > 0 {
        ((drawn_frames * total_cells) as f64 / elapsed_s).round() as u64
    } else {
        0
    };

    let ansi_bytes_per_second = (total_ansi_bytes as f64 / elapsed_s).round() as u64;
    let active_streams_avg = active_streams_sum / total_frames.max(1);

    let draw_ratio = if total_frames > 0 {
        (drawn_frames as f64) / (total_frames as f64) * 100.0
    } else {
        0.0
    };

    // --- Build report ---
    let cpu = diagnostics::detect_cpu_info();
    let ri = renderer_info::renderer_info(cfg.color_mode);

    let mut r = Report::new("COSMOSTRIX BENCHMARK");

    {
        let s = r.section("SYSTEM");
        s.field("variant", cpu.variant);
        s.field("optimization", &diagnostics::feature_string(&cpu.features));
        s.field("build", cpu.build_variant);
    }

    {
        let s = r.section("RENDERER");
        s.field("backend", ri.backend);
        s.field("pacing", ri.pacing);
        s.field("frame_strategy", ri.frame_strategy);
        s.field("color_depth", ri.color_depth);
    }

    {
        let s = r.section("CONFIG");
        s.field("cols", &w.to_string());
        s.field("lines", &h.to_string());
        s.field("target_fps", &format!("{:.1}", cfg.target_fps));
        s.field("density", &format!("{:.2}", cfg.density));
    }

    {
        let s = r.section("PERFORMANCE");
        s.field("avg_fps", &format!("{:.1}", avg_fps));
        s.field("peak_fps", &format!("{:.1}", peak_fps));
        s.field("avg_frame_time", &format!("{:.3}ms", avg_frame_time));
        s.field("p99_frame_time", &format!("{:.3}ms", p99_frame_time));
        s.field("frame_jitter", jitter_classification);
        s.field("draw_ratio", &format!("{:.1}%", draw_ratio));
    }

    {
        let s = r.section("THROUGHPUT");
        s.field("glyphs_per_second", &glyphs_per_second.to_string());
        s.field("ansi_bytes_per_second", &ansi_bytes_per_second.to_string());
        s.field("active_streams_avg", &active_streams_avg.to_string());
    }

    {
        let s = r.section("TIMING");
        s.field("elapsed", &format!("{:.3}s", elapsed_s));
        s.field("total_frames", &total_frames.to_string());
        s.field("drawn_frames", &drawn_frames.to_string());
    }

    r.print();
    Ok(())
}

/// Read benchmark dimensions from environment or use defaults.
fn bench_dimensions() -> (u16, u16) {
    let w = env::var("COSMOSTRIX_BENCH_COLS")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(DENSITY_AUTO_DEFAULT_COLS)
        .min(MAX_TERMINAL_COLS);
    let h = env::var("COSMOSTRIX_BENCH_LINES")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(DENSITY_AUTO_DEFAULT_LINES)
        .min(MAX_TERMINAL_LINES);
    (w, h)
}
