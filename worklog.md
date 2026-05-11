
---
Task ID: 1
Agent: main
Task: Implement ARC 3 — Performance Maturity

Work Log:
- Read all source files to understand post-ARC2 codebase state
- Analyzed roadmap ARC 3 items: 3.1 Hot Path Profiling, 3.2 Terminal IO Optimization, 3.3 SIMD Opportunity Audit, 3.4 Benchmark Credibility
- Delegated implementation to subagent
- Subagent implemented all 4 ARC 3 items across 11 files
- Ran ./build.sh check-all — all 6 tests pass, clippy clean, formatting correct
- Committed as 21e9418

Stage Summary:
- 11 files changed, 125 insertions(+), 45 deletions(-)
- ARC 3.1: #[inline] on 15+ hot methods, #[cold] on shutdown paths
- ARC 3.2: BufWriter<Stdout> 64 KiB, 256-byte run buffer, cursor skip optimization
- ARC 3.3: color_to_rgb Reset fast-path, pre-built blank cell in phosphor decay
- ARC 3.4: 2s warmup, 1% outlier trimming, configurable via COSMOSTRIX_BENCH_WARMUP_SECS
