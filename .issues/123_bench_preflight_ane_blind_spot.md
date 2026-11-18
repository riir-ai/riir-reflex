# Issue 123 — bench_preflight ANE blind spot: Neural Engine activity moves GPU numbers and is invisible to every current detector

**Status:** filed 2026-11-18 (from riir-refine Research 421 — the anemll-forge distill verdict round 2, Claude AGREE session c676122d). **P0 LANDED 2026-11-18 (zed-33a4c7a2@amo)** — both halves; P1's reflex-side half landed, the 4090-runbook row routed to riir-ai's lane. Detection posture unchanged: no timing bar moved; the probe makes the invisible axis visible.
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

- [x] P0: extend `bench_preflight.sh` (and `box_state::capture()` so 4090-hosted runs and Rust
      callers get it too) with an ANE-activity probe — **LANDED 2026-11-18 (zed-33a4c7a2@amo):**
      the .sh gains section 7 — the unsudo'd PROXY is a two-sample `aned` CPU-time delta over
      `ANE_PROBE_SECS` (default 3 s; the daemon idle-burns ~0 CPU, so a positive delta is current
      ANE-coordination work, never an uptime fact): delta ≥ `ANE_REFUSE_CS` (100 cs = one full
      CPU-second) REFUSES, any positive delta warns (`ane=warm`), zero reads `ane=ok`; when
      passwordless sudo exists (`sudo -n`, never prompts) the REAL `powermetrics --samplers ane`
      read rides as a disclosure line; on Linux/Windows the axis discloses n/a; an unreadable
      signal stays LOUD (`ane=unknown` in the provenance — never a silent pass);
      `ANE_PROBE_SECS=0` is the explicit opt-out (`ane=disabled(env)`). `box_state.rs` mirrors it:
      `BoxState.ane_state` (normalized ok/warm(+S)/active(+S)/unknown/na/disabled(env)), the pure
      `parse_ps_cputime_cs` parser (all three live `[DD-]HH:MM:SS.CC` shapes — macOS also prints
      an UNBOUNDED-minutes MM:SS.CC form, launchd at 16 days reads `169:40.55`; fixture-pinned),
      the 1 s-window capture probe (stamps fire twice per run), disclosure-only in the Rust
      verdict (the .sh is the refusal gate; `active` in a record lets a reader exclude the row
      post-hoc). Windows arm records `na` (no Apple ANE axis). Gates: clippy -D clean, 7/7
      box_state tests, fmt clean, live-run `ane=ok` + `ane=disabled(env)` verified on amo.
- [ ] P1: a documented ANE-quiet procedure for league/GOAT cells — **reflex-side half LANDED
      2026-11-18** (the AGENTS.md box-state paragraph names the ane axis beside the GPU
      exclusivity check + the interleaved-A/B posture is the preflight's own settle/canary
      discipline); the 4090 bout-runbook row is `perf_rematch.sh`'s file (riir-ai) — routed to
      that lane, not written here.

## Scope guard

No timing bars move until a probe exists; the issue is detection-only. Numbers above are M6
provenance (Research 421's league law — never headroom); our own M3/M5 silicon needs its own
measurements if a gate is ever pinned on this.

Cross-refs: riir-refine `.research/421_anemll_forge_ane_hybrid_prefill_distill.md` (F3/F4 faces) ·
`riir-ai/.research/392_Anemll_Forge_ANE_GDN_Numerics_Distill.md` (the 09-30 lane record) ·
anemll-forge @ `912925f7` (the measured source).
