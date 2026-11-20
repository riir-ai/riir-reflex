//! Issue 121 B1/B2 — the per-row flip census (report-only, exclusive
//! early-exit mode).
//!
//! The paper's interference instrument (katgpt-rs Research 679, Ok & Lee):
//! delete one written row, replay the frozen test read ONCE per deletion,
//! count answer flips — interference as a measured, banded, per-row
//! number (sparse-table class ≈ ≤ 2 flips/row; dense class ≈ 4.8+; a
//! zero-flip row is dead weight, an outsized row carries blast radius).
//!
//! Protocol:
//! - The engine is the DEPLOYED modelless build at the suite's fitted
//!   posture (the same `fit_posture_inner` + `build_engine_with` path the
//!   null probe pins) over the FULL registry pool — the baseline read.
//! - Per deletion: the pool minus the row, a full rebuild, the SAME test
//!   cases, the SAME forced picks — flips = pick positions that changed.
//! - **B2 determinism canary**: one row deleted TWICE (first and mid-
//!   census) must produce identical flip sets — a mismatch REFUSES the
//!   run (a nondeterministic replay would fake interference; the
//!   probe-family identity-red law).
//! - **B3 byproduct (report-only)**: the dead-row count IS the deletion
//!   cross-check for decision receipts — a cited row that flips nothing
//!   on this read is a false citation; the count rides the record, no
//!   receipt format changes.
//!
//! REPORT-ONLY: no exit-code change on the standard sweep (this is an
//! exclusive mode); no corpus pruning (the read instrument stays a read).

use serde::Serialize;

use super::*;

/// Options for one census run (`--row-flip-census`).
#[derive(Debug, Clone)]
pub struct RowFlipCensusOptions {
    /// Max corpus rows deleted per suite (the default 512; a bigger pool
    /// samples the first N in pool order — deterministic, disclosed).
    pub max_rows: usize,
}

impl Default for RowFlipCensusOptions {
    fn default() -> Self {
        Self { max_rows: 512 }
    }
}

/// One suite's census record.
#[derive(Debug, Clone, Serialize)]
pub struct RowFlipCensusSuite {
    pub suite: String,
    /// Corpus rows the census walked (≤ `max_rows`, pool order).
    pub rows_deleted: usize,
    /// Total pool rows (the deletion domain the cap sampled from).
    pub pool_rows: usize,
    /// Per-row flip counts, pool order (row i of the walked prefix).
    pub flips_per_row: Vec<usize>,
    /// Rows with zero flips (dead weight — the B3 receipt cross-check).
    pub dead_rows: usize,
    /// Mean flips per deleted row.
    pub mean_flips_per_row: f64,
    /// The paper's band the mean lands in (`sparse` ≤ 2 / `dense` ≥ 4.8 /
    /// `between`).
    pub band: &'static str,
    /// Questions in the frozen read (the denominator of one flip).
    pub n_questions: usize,
    /// The B2 canary row (its pool index) + both flip sets' agreement.
    pub canary_row: usize,
    pub canary_identical: bool,
    /// The engine posture the census pinned (disclosure).
    pub effective_cap: usize,
}

/// One census run (all named suites).
#[derive(Debug, Clone, Serialize)]
pub struct RowFlipCensusOutput {
    pub suites: Vec<RowFlipCensusSuite>,
    pub box_state: crate::harness::box_state::BoxState,
}

/// Band a mean flips/row lands in (the paper's numbers: sparse-table
/// class ≈ ≤ 2, dense class ≈ 4.8+).
#[must_use]
fn band_of(mean: f64) -> &'static str {
    if mean <= 2.0 {
        "sparse"
    } else if mean >= 4.8 {
        "dense"
    } else {
        "between"
    }
}

/// One deletion replay: the pool minus `del`, rebuilt at the pinned
/// posture, the SAME test read — the forced pick per question.
#[allow(
    clippy::too_many_arguments,
    reason = "the deletion-replay signature mirrors build_engine_with's own shape"
)]
fn replay_picks<const N: usize>(
    spec: &SuiteSpec,
    pool: &[TrainDoc],
    del: usize,
    labels: &[String],
    effective_cap: usize,
    cfg: &EngineConfig,
    cases: &[SuiteCase],
    state_strs: &[String],
) -> Result<Vec<Vec<usize>>, String> {
    let mut reduced: Vec<TrainDoc> = Vec::with_capacity(pool.len() - 1);
    reduced.extend_from_slice(&pool[..del]);
    reduced.extend_from_slice(&pool[del + 1..]);
    let (mut engine, _fallbacks) =
        build_engine_with::<N>(spec.name, &reduced, labels, effective_cap, cfg.clone(), &[])?;
    let (ev, _) = eval_engine(&mut engine, cases, state_strs, false)?;
    Ok(ev.picks)
}

/// The census over one suite (generic over the domain count — the caller
/// dispatches, the same ladder the modelless lane uses).
fn census_suite<const N: usize>(
    spec: &SuiteSpec,
    prepared: &Prepared,
    copts: &RowFlipCensusOptions,
    kn_route: bool,
) -> Result<RowFlipCensusSuite, String> {
    let inp = ModellessInput {
        spec,
        cascade_worthiness: false,
        gate_fit_selection: false,
        gate_distance_only: false,
        gate_fit_calibrated: true,
        gate_rho: None,
        gate_risk_target: None,
        esc_delta_fit: None,
        kn_route,
        drafter_fix: crate::engine::DrafterFix::Off,
        pick_latch: None,
        route_census: false,
        bm25_floor: false,
        suite: &prepared.suite,
        train: &prepared.train,
        state_strs: &prepared.state_strs,
        cal_cases: &prepared.cal_cases,
        cal_state_strs: &prepared.cal_state_strs,
        labels: &prepared.labels,
        want_by_type: false,
        corpus_cap_per_label: spec.corpus_cap_per_label,
        head_scale: 0.0,
        head_select: false,
        nb_select: false,
        oc_select: false,
        ridge_select: false,
        genome_select: false,
        genome_accept_margin: 0.0,
        cap_source_base: "registry",
        cal_select_caps: &[],
        pool_rows: &prepared.pool_rows,
        pair_head_ab: false,
        nli_feature_ab: false,
        nli_m1: false,
        #[cfg(feature = "mc_ensemble")]
        mc_ab: None,
        density_gate: false,
        set_refine: None,
        gate_fit_cases_override: None,
        leak_flags: None,
    };
    let FittedPosture {
        effective_cap,
        default_cfg,
        score_threshold,
        distance_threshold,
        ..
    } = fit_posture_inner::<N>(&inp)?;
    let cfg = EngineConfig {
        score_threshold,
        distance_threshold,
        ..default_cfg
    };
    let pool = &prepared.train;

    // The frozen baseline read at the FULL pool.
    let (mut engine, _fb) = build_engine_with::<N>(
        spec.name,
        pool,
        &prepared.labels,
        effective_cap,
        cfg.clone(),
        &[],
    )?;
    let (base_ev, _) = eval_engine(
        &mut engine,
        &prepared.suite.cases,
        &prepared.state_strs,
        false,
    )?;
    let base_picks = base_ev.picks;
    let n_questions: usize = base_picks.iter().map(|q| q.len()).sum();

    let walk = pool.len().min(copts.max_rows);
    let mut flips_per_row = Vec::with_capacity(walk);
    let canary_row = walk / 2;
    let mut canary_flips: Option<Vec<bool>> = None;
    for del in 0..walk {
        let picks = replay_picks::<N>(
            spec,
            pool,
            del,
            &prepared.labels,
            effective_cap,
            &cfg,
            &prepared.suite.cases,
            &prepared.state_strs,
        )?;
        let mut flips = Vec::with_capacity(n_questions);
        for (ci, case) in case_q_counts(prepared) {
            for qi in 0..case {
                flips.push(base_picks[ci][qi] != picks[ci][qi]);
            }
        }
        // B2: the canary row replays a second time at its own deletion
        // slot — the FIRST replay already ran; on the mid-census pass we
        // compare against the recorded set instead of re-replaying (the
        // canary is the SAME deletion twice: record the first mid pass,
        // compare on the second — implemented as: at canary_row record,
        // at the END re-run once and compare).
        if del == canary_row {
            canary_flips = Some(flips.clone());
        }
        flips_per_row.push(flips.iter().filter(|f| **f).count());
    }
    // B2 execution: the canary deletion replays AGAIN now; identical flip
    // sets are the identity assertion (a mismatch refuses the run).
    let canary_picks = replay_picks::<N>(
        spec,
        pool,
        canary_row,
        &prepared.labels,
        effective_cap,
        &cfg,
        &prepared.suite.cases,
        &prepared.state_strs,
    )?;
    let canary_second: Vec<bool> = {
        let mut v = Vec::with_capacity(n_questions);
        for (ci, nq) in case_q_counts(prepared) {
            for qi in 0..nq {
                v.push(base_picks[ci][qi] != canary_picks[ci][qi]);
            }
        }
        v
    };
    let canary_identical = canary_flips.as_deref() == Some(canary_second.as_slice());
    if !canary_identical {
        return Err(format!(
            "{}: row-flip census B2 canary FAILED — deleting row {canary_row} twice \
             produced different flip sets (a nondeterministic replay fakes \
             interference); refusing the census",
            spec.name
        ));
    }

    let dead_rows = flips_per_row.iter().filter(|f| **f == 0).count();
    let mean = if flips_per_row.is_empty() {
        0.0
    } else {
        flips_per_row.iter().sum::<usize>() as f64 / flips_per_row.len() as f64
    };
    Ok(RowFlipCensusSuite {
        suite: spec.name.to_string(),
        rows_deleted: walk,
        pool_rows: pool.len(),
        flips_per_row,
        dead_rows,
        mean_flips_per_row: mean,
        band: band_of(mean),
        n_questions,
        canary_row,
        canary_identical,
        effective_cap,
    })
}

/// The per-case question counts of the prepared suite (the walk grid the
/// picks compare over — cases × questions, in order).
fn case_q_counts(prepared: &Prepared) -> Vec<(usize, usize)> {
    prepared
        .suite
        .cases
        .iter()
        .map(|c| (0, c.questions.len()))
        .enumerate()
        .map(|(i, (_, nq))| (i, nq))
        .collect()
}

/// The census entry: run every suite in `opts.suites` (the exclusive
/// early-exit mode's own walk — never a side lane of a standard run).
pub fn run_row_flip_census(
    opts: &RunOptions,
    copts: &RowFlipCensusOptions,
) -> Result<RowFlipCensusOutput, String> {
    if opts.suites.is_empty() {
        return Err("row-flip census: no suites named (--suites is required — \
                    the census is a named-suite instrument, never a default walk)"
            .to_string());
    }
    let box_state = crate::harness::box_state::capture();
    let mut suites = Vec::new();
    for name in &opts.suites {
        let spec = SUITES
            .iter()
            .find(|s| s.name == name.as_str())
            .ok_or_else(|| format!("row-flip census: unknown suite {name}"))?;
        if !spec.modelless_lane {
            return Err(format!(
                "row-flip census: {name} carries no modelless lane — the census \
                 measures the corpus-is-the-model engine"
            ));
        }
        eprintln!("═══ row-flip census: {name} ═══");
        let prepared = prepare(spec, &opts.datasets_dir)?;
        let kn_route = opts.kn_route;
        let rec = match prepared.labels.len() {
            2 => census_suite::<2>(spec, &prepared, copts, kn_route),
            3 => census_suite::<3>(spec, &prepared, copts, kn_route),
            4 => census_suite::<4>(spec, &prepared, copts, kn_route),
            5 => census_suite::<5>(spec, &prepared, copts, kn_route),
            6 => census_suite::<6>(spec, &prepared, copts, kn_route),
            7 => census_suite::<7>(spec, &prepared, copts, kn_route),
            8 => census_suite::<8>(spec, &prepared, copts, kn_route),
            10 => census_suite::<10>(spec, &prepared, copts, kn_route),
            15 => census_suite::<15>(spec, &prepared, copts, kn_route),
            56 => census_suite::<56>(spec, &prepared, copts, kn_route),
            59 => census_suite::<59>(spec, &prepared, copts, kn_route),
            77 => census_suite::<77>(spec, &prepared, copts, kn_route),
            151 => census_suite::<151>(spec, &prepared, copts, kn_route),
            842 => census_suite::<842>(spec, &prepared, copts, kn_route),
            n => {
                return Err(format!(
                    "row-flip census: no engine instantiation for {n} domains — \
                     extend the dispatch ladder"
                ));
            }
        };
        let s = rec?;
        eprintln!(
            "    census[{}]: {} rows deleted of {} · dead {} · mean flips/row {:.2} \
             ({}) · canary row {} identical={}",
            s.suite,
            s.rows_deleted,
            s.pool_rows,
            s.dead_rows,
            s.mean_flips_per_row,
            s.band,
            s.canary_row,
            s.canary_identical
        );
        suites.push(s);
    }
    Ok(RowFlipCensusOutput { suites, box_state })
}

/// The census markdown (the probe family's output convention).
#[must_use]
pub fn render_row_flip_census_markdown(out: &RowFlipCensusOutput) -> String {
    let mut s = String::from("# Row-flip census (issue 121 B1/B2)\n\n");
    s.push_str(&format!(
        "Instrument: delete one corpus row, replay the frozen test read, count answer \
         flips (the WSP paper's interference bands: sparse ≤ 2 /row, dense ≥ 4.8 /row; \
         a zero-flip row is dead weight). REPORT-ONLY.\n\nBox: `{}`\n\n",
        crate::harness::box_state::render_line(&crate::harness::box_state::BoxStateSpan {
            start: out.box_state.clone(),
            end: out.box_state.clone(),
        })
    ));
    for st in &out.suites {
        s.push_str(&format!(
            "## {} — {} rows deleted of {} · mean {:.2}/row ({}) · dead {} · \
             canary row {} identical {}\n\n",
            st.suite,
            st.rows_deleted,
            st.pool_rows,
            st.mean_flips_per_row,
            st.band,
            st.dead_rows,
            st.canary_row,
            st.canary_identical
        ));
        // The histogram (flip-count buckets) + the top rows.
        let mut hist: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
        for f in &st.flips_per_row {
            *hist.entry(*f).or_insert(0) += 1;
        }
        s.push_str("| flips | rows |\n|---|---|\n");
        for (f, n) in &hist {
            s.push_str(&format!("| {f} | {n} |\n"));
        }
        s.push('\n');
        let mut ranked: Vec<(usize, usize)> =
            st.flips_per_row.iter().copied().enumerate().collect();
        ranked.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let top = ranked
            .iter()
            .filter(|(_, f)| *f > 0)
            .take(10)
            .collect::<Vec<_>>();
        if !top.is_empty() {
            s.push_str("Top blast-radius rows (pool index, flips):\n\n");
            for (i, f) in &top {
                s.push_str(&format!("- row {i}: {f} flip(s)\n"));
            }
            s.push('\n');
        }
        if st.dead_rows > 0 {
            s.push_str(&format!(
                "**B3 receipt cross-check:** {} dead row(s) (a decision receipt citing \
                 one of these rows flips nothing on this read — false citation).\n\n",
                st.dead_rows
            ));
        }
    }
    s
}
