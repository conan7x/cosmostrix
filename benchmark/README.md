# Benchmark

This folder contains the benchmark script and reference results for Cosmostrix.

## v2.1.0 reference results (measured)

Real measurements from the premium benchmark (`--benchmark`) and legacy CI
benchmark (`--bench-frames`) on a cloud runner. These numbers are
**machine-dependent** — they depend on CPU, terminal size, density, color mode,
and OS kernel scheduler behavior. Treat them as a baseline example, not a
portable promise.

### Environment

| Item | Value |
|---|---|
| Cosmostrix | v2.1.0 |
| CPU | Intel Xeon (4 cores, 2.8 GHz), x86-64-v4 capable (AVX-512, AVX2, BMI2, FMA) |
| OS | Linux 5.10.134 (x86_64) |
| Rust | 1.96.0 |
| Terminal size | 120x40 (headless, `TERM=dumb`) |
| Target FPS | 60 |
| Density | 1.00 |

### Performance summary (premium benchmark, 5s + 2s warmup)

| Metric | `release` (x86-64 baseline) | `pro-linux-v3` (AVX2) |
|---|---|---|
| Avg FPS | 9,910 | 10,283 |
| Peak FPS | 10,154 | 10,396 |
| Avg frame time | 0.124 ms | 0.118 ms |
| P99 frame time | 0.167 ms | 0.142 ms |
| Frame jitter | low | low |
| Avg dirty cells/frame | 145 (3.02%) | 148 (3.08%) |
| Dirty glyphs/s | 1,437,190 | 1,520,820 |
| ANSI bytes/s | 27,306,601 | 28,895,579 |
| Active streams avg | 126 | 127 |
| Full redraw ratio | 0.0% | 0.0% |

### Legacy CI benchmark (`--bench-frames 10000`)

| Profile | Frames | Elapsed | FPS |
|---|---|---|---|
| `release` | 10,000 | 1.142s | 8,757 |
| `pro-linux-v3` | 10,000 | 1.155s | 8,660 |

### Interpretation

- **v2.1.0 prioritizes cinematic quality and terminal safety** over raw
  throughput. The renderer adds phosphor ghost character tracking, bottom-row
  residue cleanup, bracketed-paste burst suppression, and Tab/focus safety
  — all of which add per-frame work compared to earlier versions.
- **Performance remains well above the 60 FPS target.** Even in the
  worst-case headless benchmark, throughput exceeds 8,600 FPS, which is
  over 140x the 60 FPS target. Real terminal rendering is I/O-bound (ANSI
  escape sequence throughput to the terminal emulator), not simulation-bound.
- **Dirty cell ratio (~3%) is the key efficiency metric.** Cosmostrix uses
  differential (dirty-cell) rendering — only cells that changed since the last
  frame are redrawn. A 3% dirty ratio means 97% of the frame buffer is reused
  unchanged each frame. Full redraws (dirty cells >= 1/3 of total) are near
  zero (0.0%).
- **Terminal rendering benchmarks vary significantly by terminal emulator,
  OS, and font rendering pipeline.** A headless benchmark measures the
  simulation/draw-computation path without actual terminal I/O, which gives
  a stable throughput ceiling. Interactive FPS depends on terminal emulator
  speed, window compositor, and display refresh rate.
- **`pro-linux-v3` vs `release`**: The AVX2-optimized build is ~3-4% faster
  on the premium benchmark simulation path. The legacy CI benchmark shows
  comparable results between the two profiles, as its shorter warmup phase
  does not fully amortize the AVX2 codegen overhead. In interactive use the
  difference is usually imperceptible because terminal I/O dominates.

## How to reproduce

### Premium benchmark (recommended)

The premium benchmark runs for 5 seconds with a 2-second warmup and produces
a comprehensive report including FPS, frame time percentiles, dirty cell
ratios, and throughput metrics:

```bash
# Build an optimized profile
cargo pro-linux-v3

# Run the premium benchmark
COSMOSTRIX_BENCH_COLS=120 COSMOSTRIX_BENCH_LINES=40 \
  target/x86_64-unknown-linux-gnu/pro-linux-v3/cosmostrix --benchmark
```

### Legacy CI benchmark

The legacy benchmark runs a fixed number of headless frames and prints
machine-parseable output:

```bash
COSMOSTRIX_BENCH_COLS=120 COSMOSTRIX_BENCH_LINES=40 \
  target/release/cosmostrix --fps 60 --bench-frames 10000
```

### Full comparison script

The benchmark script builds both `release` and `pro-native`, calibrates a
repeatable frame count, and records FPS, frame pacing, and memory/profiling
data when optional tools (hyperfine, perf, valgrind) are installed:

```bash
bash benchmark/benchmark.sh
```

CI intentionally does not gate on benchmark numbers; they are measurement
aids, not stable pass/fail thresholds.

### Generated outputs

The script generates (in this folder, gitignored):

- `hyperfine.md` — release vs pro-native comparison table
- `time-*.txt` — `/usr/bin/time -v` output
- `perf-*.txt` — `perf stat` output
- `massif-*-*.out` — Valgrind heap profiles
