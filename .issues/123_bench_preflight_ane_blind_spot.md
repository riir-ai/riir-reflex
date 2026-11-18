# Issue 123 — bench_preflight ANE blind spot: Neural Engine activity moves GPU numbers and is invisible to every current detector

**Status:** filed 2026-11-18 (from riir-refine Research 421 — the anemll-forge distill verdict round 2, Claude AGREE session c676122d; unclaimed — no lane assigned)
**Claimed:** zed-33a4c7a2@amo, 1795011642, expires 2026-11-19T14:00:00Z
  Lease scope: P0 both halves — the unsudo'd ANE-activity probe in `scripts/bench_preflight.sh` (aned CPU-delta two-sample proxy + the sudo -n powermetrics arm when passwordless sudo exists; every unreadable signal a LOUD ane=UNKNOWN disclosure, never a silent pass) + the same probe in `src/harness/box_state.rs::capture()` (macOS arm; Windows/4090 discloses n/a — no Apple ANE axis there). P1's reflex-side half (the AGENTS.md box-state/league note) rides the same landing; the 4090 bout-runbook row is riir-ai's file — routed, not written here.

## The finding

`scripts/bench_preflight.sh` checks battery, Low Power Mode (`pmset powermode`), settle time, and
load average — but **Apple Neural Engine activity moves GPU-side numbers while surfacing in NONE of
those detectors**, and ANE thermal does not appear in `pmset` output at all.

Measured basis (anemll-forge @ `912925f7f0579c0918786267a7457f7abb36e597`, MIT, M6/macOS 27.3,
`research/v8_prefill/PREFILL_RESEARCH.md` §"The coupling", read 2026-11-18):

- "The GPU attention call at 60K takes 18.7 ms alone and 26.8-27.1 ms (**+43-45%**) under an ANE
  load. The ANE FFN slows 3 to 20% under GPU load."
- Not bandwidth: "A 153 GB/s GPU memory stream costs the ANE only 7% and ALU FMA 3%, but the int8
  DeltaNet loop costs 20%." Likely cause: "a shared power or clock limit on the matrix units (not
  measured: no power or clock samples yet; shared system-cache contention for matrix work would fit
  the same tests)."
- Even wake work interferes: a GPU warm-up kick helps GPU-only (0.25→0.09 ms) but "with the ANE
  running, the GPU waking mid-call makes the ANE call ~1.3 ms longer. A spinning CPU core does the
  same (+1.5-2 ms)."
- Thermal: "both units slow under sustained load **with no pmset warning**: the ANE FFN pair call
  goes from 2.6 to 3.0-3.7 ms and the GPU mixer from 3.19 to 3.38 ms. A 5 min idle restores it."
  Configs "compared only interleaved (A B A B)".

## Why it matters here

Every latency bar this repo publishes (GOAT G2 cells, the league rows, harness suites) assumes the
GPU number measures GPU work. A concurrent ANE consumer — CoreML background work, dictation,
Spotlight indexing, a dev's own ANE lane (clef-ANE, the riir-infer audio lanes) — would inflate GPU
timings 3-45% class with zero GPU-side signal and zero pmset signal. The GPU-exclusivity check
(`ps aux | grep -E "cargo|riir|katgpt"`, riir-ai AGENTS.md) sees GPU/CPU compute consumers only;
ANE clients (CoreML/`aned`) may not match those patterns at all. The 09-30 anemll lane record
(`riir-ai/.research/392`) already priced ANE/GPU co-execution interference for the laya lane; this
issue is the bench-hygiene half.

## Proposed work (P0 cheap, P1 best-effort)

- [ ] P0: extend `bench_preflight.sh` (and `box_state::capture()` so 4090-hosted runs and Rust
      callers get it too) with an ANE-activity probe: cheapest candidate signals —
      `powermetrics -i 100 -n 1 --samplers ane` (needs sudo; document rather than run by default),
      `ioreg`/`sysdiagnose`-free proxies (ANE power via `powermetrics` is the only documented
      per-process ANE monitor; an unsudo'd proxy may be process-count-of-`aned`-clients or
      powermode-adjacent heuristics — measure what is honest and disclose what is not detectable).
      Every ANE signal that cannot be read unsudo'd becomes a LOUD "ANE state UNKNOWN" disclosure
      line, never a silent pass.
- [ ] P1: a documented ANE-quiet procedure for league/GOAT cells (the bout runbook gains an
      ANE-exclusivity row beside the GPU-exclusivity row), plus the interleaved-A/B note — anemll's
      thermal datum (5-10% drift within minutes, no OS warning) corroborates bench_preflight's
      existing settle/load checks and adds the "first rep after cool-down ~5% fastest" posture.

## Scope guard

No timing bars move until a probe exists; the issue is detection-only. Numbers above are M6
provenance (Research 421's league law — never headroom); our own M3/M5 silicon needs its own
measurements if a gate is ever pinned on this.

Cross-refs: riir-refine `.research/421_anemll_forge_ane_hybrid_prefill_distill.md` (F3/F4 faces) ·
`riir-ai/.research/392_Anemll_Forge_ANE_GDN_Numerics_Distill.md` (the 09-30 lane record) ·
anemll-forge @ `912925f7` (the measured source).
